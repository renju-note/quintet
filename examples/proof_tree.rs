//! What `VCTSolver::proof_tree` and `VCTSolver::proves` cost on the
//! benchmark's proven VCT cases, against searching again.
//!
//! For each case: solve it, read the proof tree off the tables, then for
//! every point the defender may play, ask whether the proof survives a
//! stone there: free outside the tree's zone, `proves` inside it. The
//! points inside the zone are also searched again from scratch (the same
//! solver, so with its tables), which is what a caller would do without the
//! tree.
//!
//! ```sh
//! cargo run --release --example proof_tree                  # cases tagged `quick`
//! cargo run --release --example proof_tree -- benches/cases/vct_black.txt ...
//! ```

use quintet::board::*;
use quintet::mate::*;
use std::env;
use std::fs;
use std::path::PathBuf;

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

fn main() {
    let args: Vec<PathBuf> = env::args().skip(1).map(PathBuf::from).collect();
    let named = !args.is_empty();
    let paths = if !named {
        let mut paths: Vec<PathBuf> = fs::read_dir("benches/cases")
            .expect("run from the repository root")
            .map(|e| e.unwrap().path())
            .collect();
        paths.sort();
        paths
    } else {
        args
    };
    let wanted = |tags: &[String]| {
        let has = |t: &str| tags.iter().any(|x| x == t);
        has("vct") && has("proven") && (named || has("quick"))
    };

    println!("case\tsolve\ttree\tsize\tpoints\tzone\tholds\tproves\tresolve\tresolved");
    let mut total = [0u64; 9];
    for path in paths {
        let Some((case, tags)) = read_case(&path) else {
            continue;
        };
        if !wanted(&tags) {
            continue;
        }
        let solver = &mut DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let state =
            &mut VCTState::from_board(&case.board, case.attacker, case.limit, case.threat_limit);
        let budget = &mut NodeBudget::unlimited();
        if solver.solve(state, budget).is_none() {
            println!("{}\tnot proven", case.name);
            continue;
        }
        let solve_nodes = budget.nodes();
        let budget = &mut NodeBudget::unlimited();
        let Some(tree) = solver.proof_tree(state, budget) else {
            println!("{}\tno tree", case.name);
            continue;
        };
        let tree_nodes = budget.nodes();

        let defender = case.attacker.opponent();
        let points: Vec<Point> = case
            .board
            .empty_points()
            .filter(|&p| !(defender.is_black() && case.board.forbidden(p).is_some()))
            .collect();
        let inside: Vec<Point> = points
            .iter()
            .copied()
            .filter(|&p| tree.zone().contains(p))
            .collect();
        let (mut holds, mut proves_nodes) = (0, 0);
        for &p in &inside {
            let board = case.board.with_stone(defender, p);
            let state =
                &mut VCTState::from_board(&board, case.attacker, case.limit, case.threat_limit);
            let budget = &mut NodeBudget::unlimited();
            if solver.proves(state, &tree, budget) {
                holds += 1;
            }
            proves_nodes += budget.nodes();
        }
        let (mut resolved, mut resolve_nodes) = (0, 0);
        for &p in &inside {
            let board = case.board.with_stone(defender, p);
            let state =
                &mut VCTState::from_board(&board, case.attacker, case.limit, case.threat_limit);
            let budget = &mut NodeBudget::unlimited();
            if solver.solve(state, budget).is_some() {
                resolved += 1;
            }
            resolve_nodes += budget.nodes();
        }
        let row = [
            solve_nodes,
            tree_nodes,
            tree.size() as u64,
            points.len() as u64,
            inside.len() as u64,
            holds,
            proves_nodes,
            resolve_nodes,
            resolved,
        ];
        println!("{}\t{}", case.name, row.map(|x| x.to_string()).join("\t"));
        for (t, x) in total.iter_mut().zip(row) {
            *t += x;
        }
    }
    println!("total\t{}", total.map(|x| x.to_string()).join("\t"));
}
