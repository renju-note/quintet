use super::budget::NodeBudget;
use super::game::*;
use super::mate::Mate;
use super::outcome::Outcome;
use super::solve::{
    SolveLimits, SolveMode, SolveMode::*, SolveResult, SolveStats, solve_with_stats,
};
use super::solver::Solver;
use super::state::State;
use super::vcf::{DFSSolver, VCFState};
use super::vct::{
    DFPNSVCTSolver, DFSVCTSolver, PNSVCTSolver, ThresholdPolicy, VCTSolver, VCTState,
};
use crate::board::RowKind::*;
use crate::board::*;

/// How [`solve_path`] shapes the winning path it reports, for a reader
/// rather than for the search. The default changes nothing.
///
/// Both may be set: the path is made shortest first, then its redundant
/// moves are dropped.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct PathOptions {
    /// Report the mate proven at the smallest limit the solver proves one:
    /// the fewest attacks against the defender's best replies. Costs further
    /// searches, on the same budget.
    pub shortest: bool,
    /// Drop each attack, with the reply to it, that the path does not need:
    /// a four that only delays the win, a threat the rest of the path wins
    /// without. Checked by replaying the shorter path (see [`solve_path`]).
    pub drop_redundant: bool,
}

impl PathOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_shortest(self, shortest: bool) -> Self {
        Self { shortest, ..self }
    }

    pub fn with_drop_redundant(self, drop_redundant: bool) -> Self {
        Self {
            drop_redundant,
            ..self
        }
    }

    fn any(&self) -> bool {
        self.shortest || self.drop_redundant
    }
}

/// [`solve_with_stats`], with the winning path shaped by `options`.
///
/// The answer is the one `solve` gives: the options only change which path
/// a proof is reported with, and a path they leave is as much a win as the
/// one `solve` returns. The work they take is counted in the statistics and
/// spends the same budget; when the budget runs out they stop and the path
/// they have reached is reported, so a proof is never lost to them.
///
/// - `shortest` finds the smallest limit at which the solver proves a mate,
///   under the same `threat_limit`, and reports the path proven there: the
///   mate that is shortest against the defender's best replies, which is not
///   always the one with the shortest path. It bisects between 0 and the
///   limit, taking a proof to hold at every larger limit and a disproof at
///   every smaller one (exact from where decisions carry between limits,
///   docs/06-solver-vct §4); it asks the path's own length first.
/// - `drop_redundant` tries, from the first attack on, to drop each attack
///   with the defender's reply to it, and keeps every drop after which the
///   path still holds when replayed from the board:
///   - every forced move — the block of a four, of either side — is the one
///     the path plays, and no side to move has a five to make instead;
///   - after every attack that is not a four, the attacker is proven to win
///     against every defence within the limit the path was proven at, less
///     the attacks played (a VCT search, VCT modes only: in VCF mode every
///     attack must be a four);
///   - the last attack wins at once (fours the defender cannot both block,
///     or a block that is a forbidden move).
///
///   The defences shown are the ones the path had. After an attack that is
///   not a four they are one defence among those the proof answers.
///
/// `VCTAUTO` and the reserved modes ignore the options.
///
/// ```
/// use quintet::board::{Board, Player};
/// use quintet::mate::{PathOptions, SolveLimits, SolveMode, solve_path};
///
/// let board: Board = "H8,I9,J9,H7".parse().unwrap();
/// let limits = SolveLimits::new(3);
/// let options = PathOptions::new().with_shortest(true).with_drop_redundant(true);
/// let (result, _) = solve_path(SolveMode::VCFDFS, &board, Player::Black, limits, options);
/// assert!(result.is_disproven());
/// ```
pub fn solve_path(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
    options: PathOptions,
) -> (SolveResult, SolveStats) {
    if !options.any() || matches!(mode, VCFIDDFS | VCTIDDFS | VCTAUTO) {
        return solve_with_stats(mode, board, attacker, limits);
    }
    let budget = &mut limits.budget();
    let threat_limit = limits.threat_limit;
    let depth = limits.defender_vcf_depth;
    let vcf = |limit| VCFState::from_board(board, attacker, limit);
    let vct = |limit| VCTState::from_board(board, attacker, limit, threat_limit);
    // After an attack that is not a four: does the attacker win against
    // every defence, with `left` attacks more?
    fn vct_wins<P: ThresholdPolicy>(
        attacker: Player,
        threat_limit: u8,
    ) -> impl FnMut(&mut VCTSolver<P>, &Board, u8, &mut NodeBudget) -> bool {
        move |solver, board, left, budget| {
            let state = &mut VCTState::defending(board, attacker, left, threat_limit);
            solver.search(state, budget)
        }
    }
    let never = |_: &mut DFSSolver, _: &Board, _: u8, _: &mut NodeBudget| false;
    let (outcome, memo_len) = match mode {
        VCFDFS => shaped(
            DFSSolver::new(),
            limits.limit,
            vcf,
            options,
            false,
            never,
            budget,
        ),
        VCTDFS => {
            let wins = vct_wins(attacker, threat_limit);
            shaped(
                DFSVCTSolver::new(depth),
                limits.limit,
                vct,
                options,
                true,
                wins,
                budget,
            )
        }
        VCTPNS => {
            let wins = vct_wins(attacker, threat_limit);
            shaped(
                PNSVCTSolver::new(depth),
                limits.limit,
                vct,
                options,
                true,
                wins,
                budget,
            )
        }
        VCTDFPNS => {
            let wins = vct_wins(attacker, threat_limit);
            shaped(
                DFPNSVCTSolver::new(depth),
                limits.limit,
                vct,
                options,
                true,
                wins,
                budget,
            )
        }
        VCFIDDFS | VCTIDDFS | VCTAUTO => unreachable!("answered above"),
    };
    let stats = SolveStats {
        nodes: budget.nodes(),
        memo_len,
    };
    (outcome, stats)
}

/// The steps of [`solve_path`] on `solver`, whose root at each limit
/// `state` builds, and how much its memos hold afterwards. `threats` says
/// whether an attack may be other than a four, and `wins(solver, board,
/// left, budget)` is the proof check of `drop_redundant` after such an
/// attack.
fn shaped<S: Solver>(
    mut solver: S,
    limit: u8,
    state: impl Fn(u8) -> S::State,
    options: PathOptions,
    threats: bool,
    mut wins: impl FnMut(&mut S, &Board, u8, &mut NodeBudget) -> bool,
    budget: &mut NodeBudget,
) -> (Outcome<Mate>, usize) {
    let root = state(limit);
    let outcome = solver.solve(&mut state(limit), budget);
    let Outcome::Proven(mut mate) = outcome else {
        return (outcome, solver.memo_len());
    };
    let mut proven_at = limit;
    if options.shortest {
        (mate, proven_at) = shortest(&mut solver, mate, limit, &state, budget);
    }
    if options.drop_redundant {
        let board = root.game().board();
        mate = drop_redundant(
            board,
            root.attacker(),
            mate,
            proven_at,
            threats,
            |b, left| !budget.is_exhausted() && wins(&mut solver, b, left, budget),
        );
    }
    (Outcome::Proven(mate), solver.memo_len())
}

/// The smallest limit at which `solver` proves a mate, at most `limit`
/// where `mate` was proven, and the path it proves there, found by
/// bisection: a proof at one limit is taken to hold at every larger one, a
/// disproof at every smaller one. The first limit asked is the path's own
/// number of attacks, which is the answer more often than not. Stops where
/// it is when the budget runs out.
fn shortest<S: Solver>(
    solver: &mut S,
    mut mate: Mate,
    limit: u8,
    state: impl Fn(u8) -> S::State,
    budget: &mut NodeBudget,
) -> (Mate, u8) {
    // Proven at `hi`, not at `lo`.
    let (mut lo, mut hi) = (0, limit);
    let mut first = true;
    while hi - lo > 1 && !budget.is_exhausted() {
        let mid = if first {
            mate.n_attacks().clamp(lo + 1, hi - 1)
        } else {
            lo + (hi - lo) / 2
        };
        first = false;
        match solver.solve(&mut state(mid), budget) {
            Outcome::Proven(shorter) => (mate, hi) = (shorter, mid),
            Outcome::Disproven => lo = mid,
            Outcome::Unknown => break,
        }
    }
    (mate, hi)
}

/// `mate`, proven at `limit`, with every attack, and the reply to it, that
/// it does not need dropped, from the first on: see [`solve_path`].
/// `threats` says whether an attack may be other than a four, and
/// `wins(board, left)` whether the attacker, after such an attack on
/// `board`, wins against every defence with `left` attacks more.
fn drop_redundant(
    board: &Board,
    attacker: Player,
    mut mate: Mate,
    limit: u8,
    threats: bool,
    mut wins: impl FnMut(&Board, u8) -> bool,
) -> Mate {
    let mut i = 0;
    // The last attack is the one that wins; it is never dropped.
    while 2 * i + 1 < mate.path.len() {
        let mut path = mate.path.clone();
        path.drain(2 * i..2 * i + 2);
        match replay(board, attacker, &path, limit, threats, &mut wins) {
            Some(end) => mate = Mate::new(end, path),
            None => i += 1,
        }
    }
    mate
}

/// How `path` ends if it holds when played from `board` within
/// `limit` attacks (see [`solve_path`]), `None` if it does not.
fn replay(
    board: &Board,
    attacker: Player,
    path: &[Point],
    limit: u8,
    threats: bool,
    wins: &mut impl FnMut(&Board, u8) -> bool,
) -> Option<End> {
    let defender = attacker.opponent();
    let mut game = Game::new(board, attacker);
    for (i, &p) in path.iter().enumerate() {
        let attacking = i % 2 == 0;
        let side = if attacking { attacker } else { defender };
        if game.board().stone(p).is_some() || game.is_forbidden_move(p) {
            return None;
        }
        // The other side's four must be blocked, here or not at all.
        match game.check_event() {
            Some(Forced(q)) if q != p => return None,
            Some(Defeated(_)) => return None,
            _ => {}
        }
        // Nobody passes up a five.
        if game.board().rows(side, Four).next().is_some() {
            return None;
        }
        game.play(Some(p));
        if !attacking {
            continue;
        }
        let last = i + 1 == path.len();
        match game.check_event() {
            Some(Defeated(end)) => return last.then_some(end),
            _ if last => return None,
            Some(Forced(_)) => {}
            None => {
                // The other defences may need more attacks than the path
                // shows: as many as the proof had.
                let left = limit.saturating_sub(1 + (i / 2) as u8);
                if !threats || !wins(game.board(), left) {
                    return None;
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::{Black, White};
    use crate::mate::DEFAULT_DEFENDER_VCF_DEPTH;

    fn points(s: &str) -> Vec<Point> {
        s.parse::<Points>().unwrap().0
    }

    fn path(m: &Mate) -> String {
        Points(m.path.clone()).to_string()
    }

    /// An independent check of a path, with fresh solvers: every move is
    /// legal on an empty point, and after each attack the attacker wins
    /// against every defence within `limit` less the attacks played.
    fn holds(board: &Board, attacker: Player, m: &Mate, limit: u8, threat_limit: u8) -> bool {
        let mut b = board.clone();
        for (i, &p) in m.path.iter().enumerate() {
            let side = if i % 2 == 0 {
                attacker
            } else {
                attacker.opponent()
            };
            if b.stone(p).is_some() || (side.is_black() && b.forbidden(p).is_some()) {
                return false;
            }
            b.put(side, p);
            if i % 2 == 0 {
                let left = limit.saturating_sub(1 + (i / 2) as u8);
                let state = &mut VCTState::defending(&b, attacker, left, threat_limit.max(1));
                let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
                if !solver.search(state, &mut NodeBudget::unlimited()) {
                    return false;
                }
            }
        }
        m.path.len() % 2 == 1
    }

    /// A four the path does not need is dropped with its block; the four
    /// that wins is kept.
    #[test]
    fn test_drop_redundant_four() {
        // Black's B5 is a four (B2-B5, White on B1) blocked at B6; K8 then
        // makes the straight four H8-K8 on its own.
        let board: Board = "H8,I8,J8,B2,B3,B4/B1,O15,N15".parse().unwrap();
        let mate = Mate::new(Unknown, points("B5,B6,K8"));
        let m = drop_redundant(&board, Black, mate, 2, false, |_, _| false);
        assert_eq!(path(&m), "K8");
        assert!(matches!(m.end, Fours(_, _)));
        // A path that needs its first four keeps it: without B5 there is no
        // five at B6 to threaten.
        let mate = Mate::new(Unknown, points("B5,B6,B7"));
        assert_eq!(
            path(&drop_redundant(&board, Black, mate, 2, false, |_, _| false)),
            "B5,B6,B7"
        );
    }

    /// Without options the call is `solve_with_stats`; a position without a
    /// mate, or a search cut short, is answered the same with them.
    #[test]
    fn test_options_leave_the_answer() {
        let board: Board = "H8,I9,J9,H7".parse().unwrap();
        let both = PathOptions::new()
            .with_shortest(true)
            .with_drop_redundant(true);
        for mode in [VCFDFS, VCTDFPNS, VCTAUTO] {
            let limits = SolveLimits::new(3).with_threat_limit(1);
            let plain = solve_with_stats(mode, &board, Black, limits);
            assert_eq!(
                solve_path(mode, &board, Black, limits, PathOptions::new()),
                plain
            );
            assert_eq!(solve_path(mode, &board, Black, limits, both).0, plain.0);
        }
        let limits = SolveLimits::new(4).with_threat_limit(1).with_max_nodes(5);
        let board = vct_board();
        assert!(
            solve_path(VCTDFPNS, &board, Black, limits, both)
                .0
                .is_unknown()
        );
    }

    /// DFS finds a longer VCF first; the shortest is the straight four.
    #[test]
    fn test_shortest_vcf() {
        let board: Board = "F10,G9,G10,G11,H5,H8,J10,K5,L5/D7,D9,D10,E11,I8,J6,J11,L4,L7"
            .parse()
            .unwrap();
        let limits = SolveLimits::new(5);
        let options = PathOptions::new().with_shortest(true);
        let plain = solve_path(VCFDFS, &board, Black, limits, PathOptions::new()).0;
        assert_eq!(path(plain.mate().unwrap()), "G7,G8,I9");
        let short = solve_path(VCFDFS, &board, Black, limits, options).0;
        assert_eq!(path(short.mate().unwrap()), "G8");
    }

    /// A VCT asked with four attacks more than it needs: each option
    /// shortens the path, and every path still holds.
    #[test]
    fn test_vct_paths_hold() {
        let board = game_board();
        let limits = SolveLimits::new(8).with_threat_limit(2);
        let mut lengths = vec![];
        for (shortest, drop) in [(false, false), (true, false), (false, true), (true, true)] {
            let options = PathOptions::new()
                .with_shortest(shortest)
                .with_drop_redundant(drop);
            let (result, _) = solve_path(VCTDFPNS, &board, White, limits, options);
            let mate = result.into_mate().expect("a VCT");
            assert!(
                holds(&board, White, &mate, 8, 2),
                "{options:?}: {}",
                path(&mate)
            );
            lengths.push(mate.n_attacks());
        }
        let plain = lengths[0];
        assert!(lengths[1..].iter().all(|&n| n < plain), "{lengths:?}");
        // The shortest is the 4 attacks the case is known for.
        assert_eq!(lengths[1], 4);
    }

    /// The board of `vct_board` in `solve.rs`'s tests.
    fn vct_board() -> Board {
        "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . o x o . . . . .
         . . . . . . x o . x . . . . .
         . . . . . . . x o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse()
        .unwrap()
    }

    /// `benches/cases/vct_game_white_l04_2.txt`: White's shortest VCT
    /// needs 4 attacks (`threat_limit` 2).
    fn game_board() -> Board {
        "
         . . . . . . . . . . . . . . .
         . . . . . x . x . . . . . . .
         . . . . . o x o o o x . . . .
         . . o . . o . o x x x o . . .
         . . . x . o x o o o . x . . .
         . . . . x o o o x x o . . . .
         . . . x o x x x x o x . . . .
         . . x . o x . o . . o . . . .
         . . . . o x . o . . o . . . .
         . . o x o o x o . . . . . . .
         . . o x x x x o . . . . . . .
         . . . . x . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse()
        .unwrap()
    }
}
