//! What `mate::solve_line`'s options do to the lines of the benchmark's
//! proven cases, and what they cost.
//!
//! For each case (tagged `quick`, or every one but `heavy` with an
//! argument): the line with no option, with `shortest`, with
//! `drop_redundant` and with both, as `attacks/nodes/time`. Every line is
//! checked with fresh solvers: legal, and after each attack a win against
//! every defence within the limit less the attacks played; a line that
//! fails is marked `!INVALID`. `BUMP=n` asks every case with `n` attacks
//! more than its limit, which is where the options have lines to shorten.
//!
//! ```sh
//! cargo run --release --example line_options              # cases tagged `quick`
//! BUMP=4 cargo run --release --example line_options       # limits raised by 4
//! ```

use quintet::board::*;
use quintet::mate::*;
use std::fs;

fn field<'a>(head: &'a str, key: &str) -> Option<&'a str> {
    head.lines()
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim())
}

/// Independent check: after each attack, a fresh solver proves the attacker
/// wins against every defence with the attacks left; the line is legal.
fn valid(board: &Board, attacker: Player, m: &Mate, tl: u8, limit: u8) -> bool {
    let mut b = board.clone();
    for (i, &p) in m.path.iter().enumerate() {
        if b.stone(p).is_some() {
            return false;
        }
        let side = if i % 2 == 0 {
            attacker
        } else {
            attacker.opponent()
        };
        if side.is_black() && b.forbidden(p).is_some() {
            return false;
        }
        b.put(side, p);
        if i % 2 == 0 {
            let left = limit.saturating_sub(1 + (i / 2) as u8);
            let mut s = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
            let st = &mut VCTState::defending(&b, attacker, left, tl.max(1));
            if !s.search(st, &mut NodeBudget::unlimited()) {
                return false;
            }
        }
    }
    true
}

fn main() {
    let mut paths: Vec<_> = fs::read_dir("benches/cases")
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    let all = std::env::args().nth(1).is_some();
    let mut tot = [0u32; 4];
    for path in paths {
        let text = fs::read_to_string(&path).unwrap();
        let (head, board) = text.split_once("board:").unwrap();
        let tags = field(head, "tags").unwrap_or("");
        if !(tags.contains("proven") && (all || tags.contains("quick"))) || tags.contains("heavy") {
            continue;
        }
        let mode: SolveMode = field(head, "mode").unwrap().parse().unwrap();
        if mode == SolveMode::VCTAUTO {
            continue;
        }
        let board: Board = board.trim().parse().unwrap();
        let attacker: Player = field(head, "attacker").unwrap().parse().unwrap();
        let bump: u8 = std::env::var("BUMP").ok().map_or(0, |v| v.parse().unwrap());
        let limit: u8 = field(head, "limit")
            .unwrap()
            .parse::<u8>()
            .unwrap()
            .saturating_add(bump);
        let tl: u8 = field(head, "threat_limit").map_or(0, |v| v.parse().unwrap());
        let limits = SolveLimits::new(limit).with_threat_limit(tl);
        let mut row = vec![path.file_stem().unwrap().to_string_lossy().to_string()];
        for o in [
            LineOptions::new(),
            LineOptions::new().with_shortest(true),
            LineOptions::new().with_drop_redundant(true),
            LineOptions::new()
                .with_shortest(true)
                .with_drop_redundant(true),
        ] {
            let t = std::time::Instant::now();
            let (r, st) = solve_line(mode, &board, attacker, limits, o);
            let Some(m) = r.mate() else {
                row.push(format!("not proven {:?}", r.is_unknown()));
                continue;
            };
            let ok = m.path.len() % 2 == 1 && valid(&board, attacker, m, tl, limit);
            row.push(format!(
                "{}a/{}n/{:.0}ms{}",
                m.n_attacks(),
                st.nodes,
                t.elapsed().as_secs_f64() * 1e3,
                if ok { "" } else { "!INVALID" }
            ));
        }
        for (i, c) in row[1..].iter().enumerate() {
            tot[i] += c.split('a').next().unwrap().parse::<u32>().unwrap_or(0);
        }
        println!("{}", row.join("\t"));
    }
    println!("total attacks plain/shortest/drop/both: {:?}", tot);
}
