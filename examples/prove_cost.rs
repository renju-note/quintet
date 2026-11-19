//! What `Solver::prove` saves against `Solver::solve`: the walk through
//! the tables that recovers the winning line.
//!
//! For each proven VCT case, a df-pn solver is asked the question with
//! `solve` and with `prove`, on a fresh solver (the search is most of the
//! call) and again on the solver that has answered it (the proof is in its
//! tables, so the walk is most of what `solve` does). Times are the best of
//! a few runs, in microseconds.
//!
//! ```sh
//! cargo run --release --example prove_cost                 # cases tagged `quick`
//! cargo run --release --example prove_cost -- benches/cases/vct_black.txt ...
//! ```

use quintet::board::*;
use quintet::mate::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant};

struct Case {
    name: String,
    board: Board,
    attacker: Player,
    limit: u8,
    threat_limit: u8,
}

fn read_case(path: &PathBuf) -> Option<(Case, Vec<String>)> {
    let text = fs::read_to_string(path).ok()?;
    let (head, board) = text.split_once("board:")?;
    let field = |key: &str| {
        head.lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(k, _)| k.trim() == key)
            .map(|(_, v)| v.trim().to_string())
    };
    let tags = field("tags")
        .unwrap_or_default()
        .split_whitespace()
        .map(String::from)
        .collect();
    let case = Case {
        name: path.file_stem()?.to_string_lossy().into_owned(),
        board: board.trim().parse().ok()?,
        attacker: field("attacker")?.parse().ok()?,
        limit: field("limit")?.parse().ok()?,
        threat_limit: field("threat_limit").map_or(Some(1), |v| v.parse().ok())?,
    };
    Some((case, tags))
}

/// The best time of `runs` calls of `ask` on a solver made by `fresh`
/// (a new one for each call when `reuse` is false, one answered solver
/// otherwise), and the nodes of the last.
fn time<T>(
    case: &Case,
    runs: usize,
    reuse: bool,
    ask: impl Fn(&mut DFPNSVCTSolver, &mut VCTState, &mut NodeBudget) -> Outcome<T>,
) -> (Duration, u64) {
    let state = || VCTState::from_board(&case.board, case.attacker, case.limit, case.threat_limit);
    let fresh = || DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
    let mut solver = fresh();
    if reuse {
        ask(&mut solver, &mut state(), &mut NodeBudget::unlimited());
    }
    let mut best = (Duration::MAX, 0);
    for _ in 0..runs {
        if !reuse {
            solver = fresh();
        }
        let state = &mut state();
        let budget = &mut NodeBudget::unlimited();
        let t = Instant::now();
        let outcome = ask(&mut solver, state, budget);
        let elapsed = t.elapsed();
        assert!(outcome.is_proven(), "{} is not proven", case.name);
        best = (best.0.min(elapsed), budget.nodes());
    }
    best
}

fn us(d: Duration) -> f64 {
    d.as_secs_f64() * 1e6
}

fn main() {
    let args: Vec<PathBuf> = env::args().skip(1).map(PathBuf::from).collect();
    let named = !args.is_empty();
    let paths = if named {
        args
    } else {
        let mut paths: Vec<PathBuf> = fs::read_dir("benches/cases")
            .expect("run from the repository root")
            .map(|e| e.unwrap().path())
            .collect();
        paths.sort();
        paths
    };
    let wanted = |tags: &[String]| {
        let has = |t: &str| tags.iter().any(|x| x == t);
        has("vct") && has("proven") && (named || has("quick"))
    };

    println!(
        "case\tnodes\tsolve\tprove\tsaved\tagain_nodes\tagain_solve\tagain_prove\tagain_saved"
    );
    let mut sums = [0f64; 4];
    for path in paths {
        let Some((case, tags)) = read_case(&path) else {
            continue;
        };
        if !wanted(&tags) {
            continue;
        }
        let solve = |s: &mut DFPNSVCTSolver, st: &mut VCTState, b: &mut NodeBudget| s.solve(st, b);
        let prove = |s: &mut DFPNSVCTSolver, st: &mut VCTState, b: &mut NodeBudget| s.prove(st, b);
        let runs = if named { 1 } else { 3 };
        let (s1, nodes) = time(&case, runs, false, solve);
        let (p1, _) = time(&case, runs, false, prove);
        let (s2, again) = time(&case, 50, true, solve);
        let (p2, _) = time(&case, 50, true, prove);
        let saved = |s: Duration, p: Duration| 1.0 - us(p) / us(s);
        println!(
            "{}\t{}\t{:.0}\t{:.0}\t{:.3}\t{}\t{:.1}\t{:.1}\t{:.3}",
            case.name,
            nodes,
            us(s1),
            us(p1),
            saved(s1, p1),
            again,
            us(s2),
            us(p2),
            saved(s2, p2)
        );
        for (sum, x) in sums.iter_mut().zip([s1, p1, s2, p2]) {
            *sum += us(x);
        }
    }
    println!(
        "total\t\t{:.0}\t{:.0}\t{:.3}\t\t{:.1}\t{:.1}\t{:.3}",
        sums[0],
        sums[1],
        1.0 - sums[1] / sums[0],
        sums[2],
        sums[3],
        1.0 - sums[3] / sums[2]
    );
}
