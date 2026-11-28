use super::budget::NodeBudget;
use super::game::*;
use super::mate::*;
use super::outcome::{Outcome, Proof};
use super::solver::Solver;
use super::vcf::*;
use super::vct::*;
use crate::board::Player::*;
use crate::board::RowKind::*;
use crate::board::*;
use std::convert::TryFrom;
use std::str::FromStr;

/// What `defender_vcf_depth` was fixed at before it could be given.
pub const DEFAULT_DEFENDER_VCF_DEPTH: u8 = 2;

/// Searches `board` for a mate by `attacker` with `mode`, within `limits`.
///
/// The result tells "no mate" apart from "gave up": see [`SolveResult`].
///
/// ```
/// use quintet::board::{Board, Player};
/// use quintet::mate::{SolveLimits, SolveMode, solve};
///
/// let board = Board::new();
/// let limits = SolveLimits::new(5).with_max_nodes(1_000);
/// let result = solve(SolveMode::VCFDFS, &board, Player::Black, limits);
/// assert!(result.is_disproven());
/// ```
pub fn solve(mode: SolveMode, board: &Board, attacker: Player, limits: SolveLimits) -> SolveResult {
    solve_with_stats(mode, board, attacker, limits).0
}

/// [`solve`], also reporting what the search cost: see [`SolveStats`].
///
/// The statistics are deterministic — the same arguments give the same
/// numbers on every run and every machine — which is what makes them the
/// measure to compare solver changes by (see `benches/solvers.rs`).
///
/// ```
/// use quintet::board::{Board, Player};
/// use quintet::mate::{SolveLimits, SolveMode, solve_with_stats};
///
/// let board: Board = "H8,I9,J9,H7".parse().unwrap();
/// let limits = SolveLimits::new(3);
/// let (result, stats) = solve_with_stats(SolveMode::VCFDFS, &board, Player::Black, limits);
/// assert!(result.is_disproven());
/// assert!(stats.nodes > 0);
/// ```
pub fn solve_with_stats(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
) -> (SolveResult, SolveStats) {
    ask::<Line>(mode, board, attacker, limits)
}

/// [`solve`] without the winning line: whether `attacker` has a mate, and
/// its first move ([`Solver::prove`]).
///
/// The answer and the first move are the ones `solve` gives. What is saved
/// is the walk the VCT solvers take through their tables to recover the
/// line after the proof: little next to the search on a fresh solver, as
/// here, but most of the call on a reused solver asked again.
///
/// ```
/// use quintet::board::{Board, Player};
/// use quintet::mate::{Outcome, Proof, SolveLimits, SolveMode, prove};
///
/// // Black's three G8-I8 becomes an open four at once.
/// let board: Board = "G8,H8,I8/G1,H1".parse().unwrap();
/// let limits = SolveLimits::new(2).with_threat_limit(1);
/// let outcome = prove(SolveMode::VCTDFPNS, &board, Player::Black, limits);
/// assert!(outcome.is_proven());
/// ```
pub fn prove(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
) -> Outcome<Proof> {
    prove_with_stats(mode, board, attacker, limits).0
}

/// [`solve`] bounded by a caller-supplied [`NodeBudget`] (which may carry a
/// node cap, a stop signal such as a deadline, or both). `limits.max_nodes` is
/// ignored in favour of `budget`.
pub fn solve_with_budget(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
    budget: &mut NodeBudget<'_>,
) -> SolveResult {
    ask_with_budget::<Line>(mode, board, attacker, limits, budget).0
}

/// [`prove`] bounded by a caller-supplied [`NodeBudget`], as
/// [`solve_with_budget`].
pub fn prove_with_budget(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
    budget: &mut NodeBudget<'_>,
) -> Outcome<Proof> {
    ask_with_budget::<FirstMove>(mode, board, attacker, limits, budget).0
}

/// [`prove`], also reporting what the search cost: see [`SolveStats`]. The
/// nodes are those [`solve_with_stats`] counts, which leaves recovering the
/// line out.
pub fn prove_with_stats(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
) -> (Outcome<Proof>, SolveStats) {
    ask::<FirstMove>(mode, board, attacker, limits)
}

/// What one [`solve_with_stats`] call cost.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct SolveStats {
    /// Nodes visited, counted as [`NodeBudget`] counts them: the nested VCF
    /// searches are included, recovering the winning line after a proof is
    /// not.
    pub nodes: u64,
    /// Entries the solver's memos held when the search ended
    /// ([`Solver::memo_len`]): a measure of the memory it needed.
    pub memo_len: usize,
}

/// Which search [`solve`] runs.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SolveMode {
    VCFDFS,
    /// Reserved: no solver answers it, and [`solve`] returns
    /// [`Outcome::Unknown`].
    VCFIDDFS,
    VCTDFS,
    /// Reserved, as [`Self::VCFIDDFS`].
    VCTIDDFS,
    VCTPNS,
    VCTDFPNS,
    /// df-pn with only the node budget: `limit` and `threat_limit` are
    /// ignored ([`DFPNSVCTSolver::solve_auto`]). `Disproven` means no VCT
    /// under any of [`AUTO_THREAT_LIMITS`], of any length.
    VCTAUTO,
}

pub use SolveMode::*;

// The numeric codes are the wasm/JS boundary representation of a mode (see
// `src/wasm.rs`) and are public API: do not renumber them.
impl TryFrom<u8> for SolveMode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VCFDFS),
            1 => Ok(VCFIDDFS),
            10 => Ok(VCTDFS),
            11 => Ok(VCTIDDFS),
            15 => Ok(VCTPNS),
            16 => Ok(VCTDFPNS),
            17 => Ok(VCTAUTO),
            _ => Err("Unknown solve mode"),
        }
    }
}

impl FromStr for SolveMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vcf" => Ok(VCFDFS),
            "vcf_iddfs" => Ok(VCFIDDFS),
            "vct" => Ok(VCTDFS),
            "vct_iddfs" => Ok(VCTIDDFS),
            "vct_pns" => Ok(VCTPNS),
            "vct_dfpns" => Ok(VCTDFPNS),
            "vct_auto" => Ok(VCTAUTO),
            _ => Err("Unknown solve mode"),
        }
    }
}

/// How far a search may go.
///
/// `limit` is the depth of the main line, `threat_limit` and
/// `defender_vcf_depth` those of the nested VCF searches (VCT only), and
/// `max_nodes` is the budget (see [`NodeBudget`]). Build one with [`SolveLimits::new`] and the
/// `with_*` methods so that later fields do not break callers.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SolveLimits {
    /// Max number of attacker moves in the main line.
    pub limit: u8,
    /// Max number of attacker moves in the nested VCF that decides whether a
    /// move is a threat (VCT only).
    pub threat_limit: u8,
    /// Max number of attacker moves in the nested VCF that looks for the
    /// defender's counter-VCF (VCT only).
    pub defender_vcf_depth: u8,
    /// Max number of nodes; `None` searches until the search finishes.
    pub max_nodes: Option<u64>,
}

impl SolveLimits {
    pub fn new(limit: u8) -> Self {
        Self {
            limit,
            threat_limit: 0,
            defender_vcf_depth: DEFAULT_DEFENDER_VCF_DEPTH,
            max_nodes: None,
        }
    }

    pub fn with_threat_limit(self, threat_limit: u8) -> Self {
        Self {
            threat_limit,
            ..self
        }
    }

    pub fn with_defender_vcf_depth(self, defender_vcf_depth: u8) -> Self {
        Self {
            defender_vcf_depth,
            ..self
        }
    }

    pub fn with_max_nodes(self, max_nodes: u64) -> Self {
        Self {
            max_nodes: Some(max_nodes),
            ..self
        }
    }

    pub(crate) fn budget(&self) -> NodeBudget<'static> {
        match self.max_nodes {
            Some(max_nodes) => NodeBudget::new(max_nodes),
            None => NodeBudget::unlimited(),
        }
    }
}

/// What [`solve`] concluded: the winning line, if there is a mate.
pub type SolveResult = Outcome<Mate>;

/// [`solve_with_stats`] and [`prove_with_stats`]: they differ only in what
/// they ask of the solver ([`Question`]).
fn ask<Q: Question>(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
) -> (Outcome<Q::Answer>, SolveStats) {
    let mut budget = limits.budget();
    ask_with_budget::<Q>(mode, board, attacker, limits, &mut budget)
}

fn ask_with_budget<Q: Question>(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
    budget: &mut NodeBudget<'_>,
) -> (Outcome<Q::Answer>, SolveStats) {
    // These codes are only kept taken (see `SolveMode`). Nothing is
    // searched, so nothing is known either, whatever the board.
    if matches!(mode, VCFIDDFS | VCTIDDFS) {
        return (Outcome::Unknown, SolveStats::default());
    }
    // Answered here, not by the solvers, so that no solver is built.
    if let Some(outcome) = trivial_result(board, attacker) {
        return (Q::trivial(outcome), SolveStats::default());
    }
    let limit = limits.limit;
    let threat_limit = limits.threat_limit;
    let defender_vcf_depth = limits.defender_vcf_depth;
    let vcf_state = || VCFState::from_board(board, attacker, limit);
    let vct_state = || VCTState::from_board(board, attacker, limit, threat_limit);
    let (outcome, memo_len) = match mode {
        VCFDFS => run::<Q, _>(DFSSolver::new(), vcf_state(), budget),
        VCTDFS => run::<Q, _>(DFSVCTSolver::new(defender_vcf_depth), vct_state(), budget),
        VCTPNS => run::<Q, _>(PNSVCTSolver::new(defender_vcf_depth), vct_state(), budget),
        VCTDFPNS => run::<Q, _>(DFPNSVCTSolver::new(defender_vcf_depth), vct_state(), budget),
        VCTAUTO => {
            let mut solver = DFPNSVCTSolver::new(defender_vcf_depth);
            let outcome = Q::ask_auto(&mut solver, board, attacker, budget);
            (outcome, solver.memo_len())
        }
        VCFIDDFS | VCTIDDFS => unreachable!("answered above"),
    };
    let stats = SolveStats {
        nodes: budget.nodes(),
        memo_len,
    };
    (outcome, stats)
}

/// Runs `solver` on `state`, reporting the outcome and how much the
/// solver's memos hold afterwards.
fn run<Q: Question, S: Solver>(
    mut solver: S,
    mut state: S::State,
    budget: &mut NodeBudget,
) -> (Outcome<Q::Answer>, usize) {
    let outcome = Q::ask(&mut solver, &mut state, budget);
    (outcome, solver.memo_len())
}

/// What [`ask`] asks of a solver: the line ([`Line`]) or only the first
/// move ([`FirstMove`]).
trait Question {
    type Answer;
    fn ask<S: Solver>(
        solver: &mut S,
        state: &mut S::State,
        budget: &mut NodeBudget,
    ) -> Outcome<Self::Answer>;
    fn ask_auto(
        solver: &mut DFPNSVCTSolver,
        board: &Board,
        attacker: Player,
        budget: &mut NodeBudget,
    ) -> Outcome<Self::Answer>;
    fn trivial(outcome: Outcome<Mate>) -> Outcome<Self::Answer>;
}

struct Line;

impl Question for Line {
    type Answer = Mate;
    fn ask<S: Solver>(
        solver: &mut S,
        state: &mut S::State,
        budget: &mut NodeBudget,
    ) -> Outcome<Mate> {
        solver.solve(state, budget)
    }
    fn ask_auto(
        solver: &mut DFPNSVCTSolver,
        board: &Board,
        attacker: Player,
        budget: &mut NodeBudget,
    ) -> Outcome<Mate> {
        solver.solve_auto(board, attacker, budget)
    }
    fn trivial(outcome: Outcome<Mate>) -> Outcome<Mate> {
        outcome
    }
}

struct FirstMove;

impl Question for FirstMove {
    type Answer = Proof;
    fn ask<S: Solver>(
        solver: &mut S,
        state: &mut S::State,
        budget: &mut NodeBudget,
    ) -> Outcome<Proof> {
        solver.prove(state, budget)
    }
    fn ask_auto(
        solver: &mut DFPNSVCTSolver,
        board: &Board,
        attacker: Player,
        budget: &mut NodeBudget,
    ) -> Outcome<Proof> {
        solver.prove_auto(board, attacker, budget)
    }
    fn trivial(outcome: Outcome<Mate>) -> Outcome<Proof> {
        outcome.to_proof()
    }
}

/// The answer for a board that needs no search, if it is one: the game
/// is already over (`Disproven`), or the attacker has a four and wins by
/// making five (`Proven`, by that one move).
///
/// [`solve`], [`Solver::solve`] and [`Solver::prove`] check this before
/// searching; the searches themselves do not handle these boards.
///
/// ```
/// use quintet::board::{Board, Player};
/// use quintet::mate::{SolveResult, trivial_result};
///
/// assert_eq!(trivial_result(&Board::new(), Player::Black), None);
/// let five: Board = "H8,A1,I8,A2,J8,A3,K8,A4,L8".parse().unwrap();
/// assert_eq!(trivial_result(&five, Player::White), Some(SolveResult::Disproven));
/// ```
pub fn trivial_result(board: &Board, attacker: Player) -> Option<SolveResult> {
    let over = board.rows(Black, Five).next().is_some()
        || board.rows(White, Five).next().is_some()
        || board.rows(Black, Overlined).next().is_some();
    if over {
        return Some(SolveResult::Disproven);
    }
    if let Some(eye) = board.rows(attacker, Four).flat_map(|r| r.eyes()).next() {
        return Some(SolveResult::Proven(Mate::new(Unknown, vec![eye])));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`solve`] with only the two depths, reporting just the mate.
    fn solve_mate(
        mode: SolveMode,
        limit: u8,
        board: &Board,
        attacker: Player,
        threat_limit: u8,
    ) -> Option<Mate> {
        let limits = SolveLimits::new(limit).with_threat_limit(threat_limit);
        solve(mode, board, attacker, limits).into_mate()
    }

    #[test]
    fn test_vcf_black() -> Result<(), String> {
        // https://renjuportal.com/puzzle/3040/
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . o . . .
         . . . . o . x o . . . . . . .
         . . . . . . . o . . x . . . .
         . . . . . . . x o . . x . . .
         . . . . . . o o x . o . . . .
         . . . . . x . x x o . x . . .
         . . . . . . . o o x . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution: String = "
            J12,K13,G9,F8,G6,H7,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .split_whitespace()
        .collect();

        let result = solve_mate(VCFDFS, 12, &board, Black, 0);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCFDFS, 11, &board, Black, 0);
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_vcf_white() -> Result<(), String> {
        // https://renjuportal.com/puzzle/2990/
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . x . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . x . x o . .
         . . . . . . . . x . . . o . .
         . . . . . . . x x o . x . . .
         . . . . . . o x o o . . o . .
         . . . . . x o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14";

        let result = solve_mate(VCFDFS, 5, &board, White, 0);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCFDFS, 4, &board, White, 0);
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_vcf_long() -> Result<(), String> {
        // "shadows and fog" by Tama Hoshiduki
        let board = "
         . o x . x o . o x x x x o x x
         . . . o . . x o o x . . x o o
         x . o . . . . . . . . . o . o
         o . . . x x . . . . . . . x x
         . . o . . . . . . . . . . o x
         x o x x . . . . . . . . . o o
         x o . o . . x . . . . o . . .
         o x x x . . . o . x . . . . x
         x x . . . . . . . . . . . . x
         x . . . . . x o x . . . . . x
         o . . . o . . . . x . . . . o
         . o . o . . . x o . . . . . .
         . . . . . . x . o o . . . . .
         o . . . . . . . . o . . x o .
         . . . o . . o x . . o . . . o
        "
        .parse::<Board>()?;

        let solution: String = "
            F6,G7,C3,B2,E1,D2,C1,F1,A1,B1,A4,A3,C4,E4,C5,C2,C6,C7,D5,B5,
            E6,B3,D6,B6,G8,F7,D7,D3,F5,G5,G4,H3,F8,E7,I8,E8,F2,E3,F3,F4,
            H5,E2,H7,H9,L1,K2,M1,N1,I1,J1,I2,I5,H2,G2,K5,J4,L4,M3,M5,K3,
            L5,N5,L3,L2,L6,L7,M6,K4,J6,I7,K6,N6,M4,J7,M7,M8,N8,O9,N7,N9,
            O2,N3,O3,O4,K7,N4,K9,K8,M9,L8,J9,I9,K10,L11,M10,L10,M12,M11,L13,K14,
            K13,N13,K11,K12,J10,L12,I13,J13,J12,G15,I11,L14,H12,G13,H11,H13,G11,J11,E11,F11,
            I10,I12,G10,H10,E9,F10,F9,C9,D11,E10,B11,A11,B13,B12,F13,G12,D13,E13,D12,D15,
            B14,A15,E14,C12,C14
        "
        .split_whitespace()
        .collect();

        let result = solve_mate(VCFDFS, u8::MAX, &board, Black, u8::MAX);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    /// White's forced block K9 makes two fours, K5-K9 and K9-K12, but both
    /// are completed only at K8, so one block stops them: it is not a
    /// double-four, and Black's K8 is both that block and a new four.
    #[test]
    fn test_vcf_not_opponent_double_four() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . . . x . o . .
         . . . . . . . . o . x o . . .
         . . . . . . . . x o . . . . .
         . . . . . . x o o . . . . . .
         . . . . . . . . o . x . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "J8,K9,K8,L8,H11";

        let result = solve_mate(VCFDFS, 3, &board, Black, 0);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    /// Black's fours G8 and I8 are the two ends of one sword. G8 first
    /// loses: White's block at I8 makes a double-four. The line has to
    /// start with I8.
    #[test]
    fn test_vcf_counter() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . o . o o x . . .
         . . . . . . . . o x . o . . .
         . . . . . . . . o . x . . . .
         . . . . . . . . x . . x . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution: String = "I8,G8,I10,I9,J9".split_whitespace().collect();

        let result = solve_mate(VCFDFS, u8::MAX, &board, Black, 0);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    fn test_vct_black() -> Result<(), String> {
        // No. 02 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
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
        .parse::<Board>()?;

        let solution = "F10,G9,I10,G10,H11,H12,G12";

        let result = solve_mate(VCTDFS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFS, 3, &board, Black, 1);
        assert!(result.is_none());

        let result = solve_mate(VCTPNS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFPNS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    fn test_vct_white() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . o . . o . . . . .
         . . . . . . o x x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "I10,I6,I11,I8,F7,G7,G8";

        let result = solve_mate(VCTDFS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFS, 3, &board, White, 1);
        assert!(result.is_none());

        let solution = "I10,I6,I11,I8,J11,G7,G8";
        let result = solve_mate(VCTPNS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFPNS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    fn test_vct_counter() -> Result<(), String> {
        // No. 63 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . o . . . . .
         . . . . . . . o x . . . . . .
         . . . x x o . x o . . . . . .
         . . . . . o . o o x . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . x . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "F7,E8,G8,E6,G5,G7,H6";

        let result = solve_mate(VCTDFS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFS, 3, &board, White, 1);
        assert!(result.is_none());

        let result = solve_mate(VCTPNS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFPNS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    fn test_vct_forbidden_breaker() -> Result<(), String> {
        // No. 68 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . o x o . . . . .
         . . . . . . o x o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "J8,I7,I8,G8,L8,K8,K7";

        let result = solve_mate(VCTDFS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFS, 3, &board, Black, 1);
        assert!(result.is_none());

        let result = solve_mate(VCTPNS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFPNS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    fn test_vct_fukumi_move() -> Result<(), String> {
        // https://twitter.com/nachirenju/status/1487315157382414336
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . x o o . . . . . .
         . . . . . o o o x x . . . . .
         . . . . o x x x x o . . . . .
         . . . x . x o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let result = solve_mate(VCTDFS, 7, &board, Black, 3);
        let solution = "G12,H13,E12,F12,E10,D10,E11";
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFS, 6, &board, Black, 3);
        assert!(result.is_none());

        let result = solve_mate(VCTDFS, 7, &board, Black, 2);
        assert!(result.is_none());

        let result = solve_mate(VCTPNS, 7, &board, Black, 3);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFPNS, 7, &board, Black, 3);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    fn test_vct_dual_forbiddens() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . o o . . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . . x x o . . . . .
         . . . . . . o o x . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "I3,I4,K3,G7,L2,J4,L3,J8,J3";
        let result = solve_mate(VCTDFS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFS, 4, &board, White, 1);
        assert!(result.is_none());

        let result = solve_mate(VCTDFPNS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let solution = "K3,G7,I3,I4,L2,J4,L3,J8,J3";
        let result = solve_mate(VCTPNS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    /// Problem 88 of Shigeru Nakamura's "中村シェフのおいしい詰連珠": Black's
    /// first attack N4 makes no two on any line, yet threatens a VCF whose
    /// fours fill row 4 (L4, O4, K4, M4). Attacks generated from the twos
    /// alone never tried it.
    #[test]
    fn test_vct_quiet_attack() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . x x . . .
         . . . . . . . . . . o . . . .
         . . . . . . . x . x o x o . .
         . . . . . . x o o x o o x . .
         . . . . . . . o x o o o x . .
         . . . . . . x . o x x . o . .
         . . . . . . . x . . . o . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "N4,L4,N7,M3,K5,K4,N9,N11,N5,O4,M5";
        for mode in [VCTDFS, VCTPNS, VCTDFPNS] {
            let result = solve_mate(mode, 6, &board, Black, 4);
            assert_eq!(path_string(result), solution, "{mode:?}");
        }

        let result = solve_mate(VCTDFPNS, 5, &board, Black, 4);
        assert!(result.is_none());

        Ok(())
    }

    /// The position of `test_vct_black`: Black to move wins by VCT in 4.
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
        .parse::<Board>()
        .unwrap()
    }

    const VCT_SOLUTION: &str = "F10,G9,I10,G10,H11,H12,G12";

    #[test]
    fn test_solve_result() {
        let board = vct_board();
        let limits = SolveLimits::new(4).with_threat_limit(1);

        let result = solve(VCTDFPNS, &board, Black, limits);
        assert!(result.is_proven());
        assert_eq!(
            Points(result.into_mate().unwrap().path).to_string(),
            VCT_SOLUTION
        );

        // There is no VCT in 3, and the search says so rather than giving up.
        let result = solve(
            VCTDFPNS,
            &board,
            Black,
            SolveLimits::new(3).with_threat_limit(1),
        );
        assert_eq!(result, SolveResult::Disproven);

        // One node is not enough to decide anything.
        let result = solve(VCTDFPNS, &board, Black, limits.with_max_nodes(1));
        assert_eq!(result, SolveResult::Unknown);
    }

    /// Positions that are decided before any search.
    #[test]
    fn test_solve_decided_positions() -> Result<(), String> {
        let limits = SolveLimits::new(3);
        let result = |board: &str, attacker| -> Result<SolveResult, String> {
            Ok(solve(VCFDFS, &board.parse::<Board>()?, attacker, limits))
        };

        // The game is already over: there is nothing to win.
        for board in [
            "H8,I8,J8,K8,L8/A1",       // Black five
            "A1,A2/H8,I8,J8,K8,L8",    // White five
            "G8,H8,I8,J8,K8,L8/A1,A2", // Black overline
        ] {
            assert_eq!(result(board, Black)?, SolveResult::Disproven, "{board}");
            assert_eq!(result(board, White)?, SolveResult::Disproven, "{board}");
        }

        // The attacker has a four already and wins by making five.
        let four = "H8,I8,J8,K8/G8";
        let l8 = "L8".parse::<Point>()?;
        assert_eq!(
            result(four, Black)?,
            SolveResult::Proven(Mate::new(Unknown, vec![l8]))
        );
        let board = four.parse::<Board>()?;
        let proof = Outcome::Proven(Proof { first_move: l8 });
        assert_eq!(prove(VCTDFPNS, &board, Black, limits), proof);
        Ok(())
    }

    /// A solver asked directly answers the decided positions as [`solve`]
    /// does, rather than searching them.
    #[test]
    fn test_solver_answers_decided_positions() -> Result<(), String> {
        let budget = &mut NodeBudget::unlimited();
        let five = "H8,I8,J8,K8,L8/A1,A2,A3,A4".parse::<Board>()?;
        let state = &mut VCFState::from_board(&five, White, 3);
        assert_eq!(DFSSolver::new().solve(state, budget), Outcome::Disproven);
        let state = &mut VCTState::from_board(&five, White, 3, 1);
        let mut vct = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        assert_eq!(vct.prove(state, budget), Outcome::Disproven);

        let four = "H8,I8,J8,K8/G8,A1,A2".parse::<Board>()?;
        let l8 = "L8".parse::<Point>()?;
        let state = &mut VCTState::from_board(&four, Black, 3, 1);
        assert_eq!(
            vct.prove(state, budget),
            Outcome::Proven(Proof { first_move: l8 })
        );
        assert_eq!(budget.nodes(), 0);
        Ok(())
    }

    /// [`prove`] gives the answer and the first move [`solve`] gives, in
    /// every mode, for the same nodes.
    #[test]
    fn test_prove_agrees_with_solve() {
        let boards = [(vct_board(), Black), (vct_board(), White)];
        let modes = [VCFDFS, VCTDFS, VCTPNS, VCTDFPNS, VCTAUTO, VCFIDDFS];
        for (board, attacker) in &boards {
            for mode in modes {
                for limit in [3, 4] {
                    for max_nodes in [10, 1_000_000] {
                        let limits = SolveLimits::new(limit)
                            .with_threat_limit(1)
                            .with_max_nodes(max_nodes);
                        let (line, line_stats) = solve_with_stats(mode, board, *attacker, limits);
                        let (proof, proof_stats) = prove_with_stats(mode, board, *attacker, limits);
                        let what = format!("{mode:?} {attacker:?} {limit} {max_nodes}");
                        assert_eq!(proof, line.to_proof(), "{what}");
                        assert_eq!(proof_stats.nodes, line_stats.nodes, "{what}");
                    }
                }
            }
        }
    }

    /// The root report of a proven root: the winning attack is proven, and
    /// the nodes under the moves add up to no more than the search's.
    #[test]
    fn test_root_report_proven() {
        let board = vct_board();
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        assert!(solver.prove(state, budget).is_proven());
        let report = solver.root_report();
        assert!(report.attacking);
        assert_eq!(report.root.pn, 0);
        assert_eq!(report.nodes, budget.nodes());
        assert!(!report.moves.is_empty());
        let first = report.get(threat_start()).expect("the winning attack");
        assert_eq!(first.status, MoveStatus::Proven);
        assert!(first.nodes > 0);
        let under: u64 = report.moves.iter().map(|m| m.nodes).sum();
        assert!(under < report.nodes);
        // A point that was no candidate is not in it.
        assert!(report.get("A1".parse().unwrap()).is_none());
    }

    /// A disproven root: every move is decided, and each decision is what
    /// a fresh search of the position after it finds.
    #[test]
    fn test_root_report_disproven_agrees_with_fresh_searches() {
        let board = vct_board();
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let state = &mut VCTState::from_board(&board, Black, 3, 1);
        assert!(
            solver
                .prove(state, &mut NodeBudget::unlimited())
                .is_disproven()
        );
        let report = solver.root_report().clone();
        assert_eq!(report.root.dn, 0);
        assert!(!report.moves.is_empty());
        for m in &report.moves {
            assert!(
                matches!(m.status, MoveStatus::Disproven | MoveStatus::Forbidden),
                "{m:?}"
            );
            // The attack and every defence: Black has no VCT in 2 after it.
            let next = board.with_stone(Black, m.point);
            let state = &mut VCTState::defending(&next, Black, 2, 1);
            let mut fresh = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
            assert!(!fresh.search(state, &mut NodeBudget::unlimited()), "{m:?}");
        }
    }

    /// A search cut short still reports: the root is open, and the move it
    /// was searching when the budget ran out has numbers though the tables
    /// do not hold them.
    #[test]
    fn test_root_report_unfinished() {
        let board = vct_board();
        for max_nodes in [10, 50, 200] {
            let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
            let budget = &mut NodeBudget::new(max_nodes);
            let state = &mut VCTState::from_board(&board, Black, 4, 1);
            assert!(solver.prove(state, budget).is_unknown());
            let report = solver.root_report();
            assert!(report.root.pn > 0 && report.root.dn > 0, "{max_nodes}");
            let busiest = report.moves.iter().max_by_key(|m| m.nodes).unwrap();
            assert!(busiest.nodes > 0, "{max_nodes}");
            assert!(
                matches!(busiest.status, MoveStatus::Open { .. }),
                "{max_nodes}: {busiest:?}"
            );
            let under: u64 = report.moves.iter().map(|m| m.nodes).sum();
            assert!(under < report.nodes, "{max_nodes}");
        }
    }

    /// A forbidden attack is reported as such, not as an ordinary disproof.
    #[test]
    fn test_root_report_forbidden_attack() -> Result<(), String> {
        // H8 is a double three (F8-G8-H8 and H6-H7-H8).
        let board = "F8,G8,H6,H7/A1,A3,A5,A7".parse::<Board>()?;
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let state = &mut VCTState::from_board(&board, Black, 1, 1);
        assert!(
            solver
                .prove(state, &mut NodeBudget::unlimited())
                .is_disproven()
        );
        let report = solver.root_report();
        let h8 = report.get("H8".parse()?).expect("H8 is generated");
        assert_eq!(h8.status, MoveStatus::Forbidden);
        let i8 = report.get("I8".parse()?).expect("I8 is generated");
        assert_eq!(i8.status, MoveStatus::Disproven);
        Ok(())
    }

    /// A root with the defender to move: its moves are the defences, and
    /// each one's answer is what a fresh search after it finds, with the
    /// limit counting the attacker's moves after the defence.
    #[test]
    fn test_root_report_defending() {
        let board = vct_board().with_stone(Black, threat_start());
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let state = &mut VCTState::defending(&board, Black, 3, 1);
        assert!(solver.search(state, &mut NodeBudget::unlimited()));
        let report = solver.root_report().clone();
        assert!(!report.attacking);
        assert_eq!(report.root.pn, 0);
        assert!(report.moves.len() > 1);
        for m in &report.moves {
            assert_eq!(m.status, MoveStatus::Proven, "{m:?}");
            let next = board.with_stone(White, m.point);
            let limits = SolveLimits::new(3).with_threat_limit(1);
            assert!(solve(VCTDFPNS, &next, Black, limits).is_proven(), "{m:?}");
        }
        // With one attack fewer, some defence holds.
        let state = &mut VCTState::defending(&board, Black, 2, 1);
        assert!(!solver.search(state, &mut NodeBudget::unlimited()));
        let report = solver.root_report();
        assert_eq!(report.root.dn, 0);
        assert!(
            report
                .moves
                .iter()
                .any(|m| m.status == MoveStatus::Disproven)
        );
    }

    /// A reused VCT solver asked a question it has answered finds the proof
    /// in its tables, and [`Solver::prove`] reports the first move
    /// [`Solver::solve`] puts first, before and after.
    #[test]
    fn test_reused_solver_proves_again() {
        let board = vct_board();
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        let first = solver.prove(state, budget);
        assert_eq!(
            first,
            Outcome::Proven(Proof {
                first_move: threat_start()
            })
        );
        let before = budget.nodes();
        assert_eq!(solver.prove(state, budget), first);
        assert!(budget.nodes() - before <= 1);
        assert_eq!(path_string(solver.solve(state, budget)), VCT_SOLUTION);
        let state = &mut VCTState::from_board(&board, Black, 3, 1);
        assert_eq!(solver.prove(state, budget), Outcome::Disproven);
    }

    /// The numeric codes and names are what callers pass in: see
    /// `src/wasm.rs` and `examples/solve.rs`.
    #[test]
    fn test_solve_mode_codes() {
        let modes = [
            (0, "vcf", VCFDFS),
            (1, "vcf_iddfs", VCFIDDFS),
            (10, "vct", VCTDFS),
            (11, "vct_iddfs", VCTIDDFS),
            (15, "vct_pns", VCTPNS),
            (16, "vct_dfpns", VCTDFPNS),
        ];
        for (code, name, mode) in modes {
            assert_eq!(SolveMode::try_from(code), Ok(mode));
            assert_eq!(name.parse::<SolveMode>(), Ok(mode));
        }
        assert!(SolveMode::try_from(2).is_err());
        assert!("dfpn".parse::<SolveMode>().is_err());
    }

    /// The reserved modes search nothing, so they must not claim there is
    /// no mate: not where there is one, nor where the board decides it.
    #[test]
    fn test_reserved_modes_are_aborted() -> Result<(), String> {
        let four = "H8,I8,J8,K8/G8".parse::<Board>()?;
        for board in [vct_board(), four] {
            for mode in [VCFIDDFS, VCTIDDFS] {
                let limits = SolveLimits::new(4).with_threat_limit(3);
                let (result, stats) = solve_with_stats(mode, &board, Black, limits);
                assert_eq!(result, SolveResult::Unknown, "{mode:?}");
                assert_eq!(stats, SolveStats::default());
            }
        }
        Ok(())
    }

    /// `solve_attacks` names the solution's first move as proven, and each
    /// attack's answer agrees with the root's: the root is proven exactly
    /// when some attack is, and a shorter limit proves none.
    #[test]
    fn test_solve_attacks() {
        let board = vct_board();
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        let attacks = solver.solve_attacks(state, 100_000);
        assert!(attacks.len() > 1);
        let first = VCT_SOLUTION.split(',').next().unwrap();
        let answer = |p: &str| attacks.iter().find(|(a, _)| a.to_string() == p).unwrap().1;
        assert_eq!(answer(first), Some(true));
        assert!(attacks.iter().all(|(_, a)| a.is_some()));

        let state = &mut VCTState::from_board(&board, Black, 3, 1);
        let attacks = solver.solve_attacks(state, 100_000);
        assert!(attacks.iter().all(|(_, a)| *a == Some(false)));

        // The same solver still finds the mate afterwards.
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        let result = solver.solve(state, &mut NodeBudget::unlimited());
        assert_eq!(path_string(result), VCT_SOLUTION);
    }

    /// A search that gave up must not leave anything behind that hides the
    /// mate from the next, longer search on the same solver.
    #[test]
    fn test_abort_leaves_no_wrong_memo() {
        let board = vct_board();
        for max_nodes in [1, 2, 3, 5, 8, 13, 21, 34, 55, 89] {
            let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);

            let budget = &mut NodeBudget::new(max_nodes);
            let state = &mut VCTState::from_board(&board, Black, 4, 1);
            let aborted = solver.solve(state, budget);

            let budget = &mut NodeBudget::unlimited();
            let state = &mut VCTState::from_board(&board, Black, 4, 1);
            let result = path_string(solver.solve(state, budget));
            assert_eq!(result, VCT_SOLUTION, "after {} nodes", max_nodes);

            // Whatever the small budget did find must have been right too.
            if let Some(mate) = aborted.into_mate() {
                assert_eq!(Points(mate.path).to_string(), VCT_SOLUTION);
            }
        }
    }

    /// One solver, one budget, several positions: the budget adds up and
    /// `clear` throws the tables away.
    #[test]
    fn test_reused_solver() {
        let board = vct_board();
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();

        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        assert_eq!(path_string(solver.solve(state, budget)), VCT_SOLUTION);
        let first = budget.nodes();
        assert!(first > 0);

        // The second search reuses the tables, so it costs less than the first.
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        assert_eq!(path_string(solver.solve(state, budget)), VCT_SOLUTION);
        let second = budget.nodes() - first;
        assert!(second < first);

        solver.clear();
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        assert_eq!(path_string(solver.solve(state, budget)), VCT_SOLUTION);
        let third = budget.nodes() - first - second;
        assert_eq!(third, first);
    }

    /// Both attackers on one solver. Every memo is keyed by the attacker, so
    /// what the Black searches leave behind must not reach the White ones.
    #[test]
    fn test_reused_solver_both_attackers() {
        let board = vct_board();
        let mut reused = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();

        for round in 0..3 {
            for attacker in [Black, White] {
                let state = &mut VCTState::from_board(&board, attacker, 4, 1);
                let got = reused.solve(state, budget).is_proven();

                let mut fresh = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
                let state = &mut VCTState::from_board(&board, attacker, 4, 1);
                let want = fresh.solve(state, &mut NodeBudget::unlimited()).is_proven();

                assert_eq!(got, want, "round {round}, {attacker:?} attacking");
            }
        }
        // Black wins here and White does not, so the two answers differ and a
        // key that ignored the attacker would have had to get one of them
        // wrong.
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        assert!(reused.solve(state, budget).is_proven());
        let state = &mut VCTState::from_board(&board, White, 4, 1);
        assert!(reused.solve(state, budget).is_disproven());
    }

    /// The reuse this is for: one move's worth of questions — does Black have
    /// a VCT, and does each candidate defence stop it — asked of a single
    /// solver. Every answer has to be the one a fresh solver gives.
    #[test]
    fn test_reused_solver_matches_fresh_over_a_move() {
        let board = vct_board();
        let limits = SolveLimits::new(4).with_threat_limit(1);

        let threat = solve(VCTDFPNS, &board, Black, limits)
            .into_mate()
            .expect("Black has a VCT");
        let defences = VCTState::from_board(&board, White, 4, 1).threat_defences(&threat);

        let mut reused = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        let mut seen = std::collections::HashSet::new();
        let mut checked = 0;
        for p in defences {
            if !seen.insert(p) || board.forbidden(p).is_some() {
                continue;
            }
            let next = board.with_stone(White, p);

            let state = &mut VCTState::from_board(&next, Black, 4, 1);
            let got = reused.solve(state, budget).is_proven();

            let mut fresh = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
            let state = &mut VCTState::from_board(&next, Black, 4, 1);
            let want = fresh.solve(state, &mut NodeBudget::unlimited()).is_proven();

            assert_eq!(got, want, "after White {p}");
            checked += 1;
        }
        assert!(checked > 1, "expected several defences to check");
    }

    /// A proof carried over from an earlier search may have lost the
    /// entries under it: a memo drops what the searches before the last one
    /// wrote, and a search that only read an entry did not write it again.
    /// With every generation but the last dropped, solving the line of
    /// `test_vct_black` backwards proved each position, but the root's line
    /// could not be extracted and `solve` answered `None`.
    #[test]
    fn test_reused_solver_extracts_a_proof_whose_subtree_was_dropped() {
        let board = "
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
        .parse::<Board>()
        .unwrap();
        let line: Vec<Point> = "F10,G9,I10,G10,H11,H12,G12"
            .split(',')
            .map(|m| m.parse().unwrap())
            .collect();
        let mut reused = DFPNSVCTSolver::with_carry_capacity(DEFAULT_DEFENDER_VCF_DEPTH, 1);
        let budget = &mut NodeBudget::unlimited();
        for k in (0..=2).rev() {
            let mut next = board.clone();
            for (i, &m) in line[..2 * k].iter().enumerate() {
                next.put(if i % 2 == 0 { Black } else { White }, m);
            }
            let state = &mut VCTState::from_board(&next, Black, 4 - k as u8, 1);
            let got = reused
                .solve(state, budget)
                .into_mate()
                .map(|m| Points(m.path).to_string());
            let want = Points(line[2 * k..].to_vec()).to_string();
            assert_eq!(got, Some(want), "{k} attacks in");
        }
    }

    /// A proof left in the table by an earlier search must not prove a
    /// position through a move that is forbidden there. After White E8,
    /// Black's search reaches the board below with D8 played before F10,
    /// when it was no double three yet. After White F11, Black must block at
    /// F10 first, and D8 is then a double three (D8-E7-F6 and D8-E9-F10): the
    /// reused solver took the stored proof and answered `F10,E8,D8,C7,C9`.
    #[test]
    fn test_reused_solver_skips_a_forbidden_attack_from_the_table() {
        let board: Board = "H8,J8,J9,J11,K10,G10,I7,H7,E6,E9,F6,E7\
                            /H9,G8,I10,J10,I8,I6,F9,F7,G9,F8,D6"
            .parse()
            .unwrap();
        let mut reused = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        for p in ["E8", "F11"] {
            let next = board.with_stone(White, p.parse().unwrap());
            let state = &mut VCTState::from_board(&next, Black, 5, 2);
            let got = reused
                .solve(state, budget)
                .into_mate()
                .map(|m| Points(m.path).to_string());
            let mut fresh = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
            let state = &mut VCTState::from_board(&next, Black, 5, 2);
            let want = fresh
                .solve(state, &mut NodeBudget::unlimited())
                .into_mate()
                .map(|m| Points(m.path).to_string());
            assert_eq!(got, want, "after White {p}");
            assert_eq!(got, None, "after White {p}");
        }
    }

    /// A reused solver must not grow with the number of searches. One search
    /// is held down by its node budget; a series of them is held down by the
    /// generations, which settle the memos at about two searches' worth.
    #[test]
    fn test_reused_solver_memo_stays_bounded() {
        let board = vct_board();
        let budget = &mut NodeBudget::unlimited();
        let positions: Vec<_> = board
            .empty_points()
            .take(24)
            .enumerate()
            .filter(|(_, p)| board.forbidden(*p).is_none())
            .map(|(i, p)| board.with_stone(if i % 2 == 0 { White } else { Black }, p))
            .collect();
        assert!(positions.len() > 20);

        // Small enough that every search drops what the ones before it left.
        let mut solver = DFPNSVCTSolver::with_carry_capacity(DEFAULT_DEFENDER_VCF_DEPTH, 0);
        let mut early = 0;
        let mut total_alone = 0;
        for (i, b) in positions.iter().enumerate() {
            solver.solve(&mut VCTState::from_board(b, Black, 4, 1), budget);
            if i == 3 {
                early = solver.memo_len();
            }
            let mut alone = DFPNSVCTSolver::with_carry_capacity(DEFAULT_DEFENDER_VCF_DEPTH, 0);
            alone.solve(
                &mut VCTState::from_board(b, Black, 4, 1),
                &mut NodeBudget::unlimited(),
            );
            total_alone += alone.memo_len();
        }
        let late = solver.memo_len();

        assert!(early > 0, "nothing was memoized");
        // Flat, not cumulative: keeping every search would reach `total_alone`.
        assert!(
            late < 2 * early,
            "grew from {early} after 4 searches to {late} after {}",
            positions.len()
        );
        assert!(
            late * 4 < total_alone,
            "{late} is not far enough below the {total_alone} of keeping everything"
        );
    }

    /// The whole point of keying decisions by position alone: a proof found
    /// at one limit answers at a larger one, and a disproof at a smaller.
    #[test]
    fn test_decisions_carry_between_limits() {
        let board = vct_board();
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();

        // Black has a VCT in 4 and none in 3.
        assert!(
            solver
                .solve(&mut VCTState::from_board(&board, Black, 3, 1), budget)
                .is_disproven()
        );
        let after_three = budget.nodes();
        assert!(
            solver
                .solve(&mut VCTState::from_board(&board, Black, 4, 1), budget)
                .is_proven()
        );
        let four = budget.nodes() - after_three;

        // Proven at 4, so limits 5 and 6 are answered from the bound.
        for limit in [5u8, 6] {
            let before = budget.nodes();
            assert!(
                solver
                    .solve(&mut VCTState::from_board(&board, Black, limit, 1), budget)
                    .is_proven(),
                "limit {limit}"
            );
            let spent = budget.nodes() - before;
            assert!(
                spent * 8 < four,
                "limit {limit} still cost {spent} of {four}"
            );
        }

        // Disproven at 3, so limits 2 and 1 are answered from the bound.
        for limit in [2u8, 1] {
            let before = budget.nodes();
            assert!(
                solver
                    .solve(&mut VCTState::from_board(&board, Black, limit, 1), budget)
                    .is_disproven(),
                "limit {limit}"
            );
            assert!(budget.nodes() - before < after_three);
        }
    }

    /// Carrying a decision between limits must not change any answer: one
    /// solver asked every limit in a jumbled order, against fresh solvers.
    #[test]
    fn test_reused_solver_matches_fresh_across_limits() {
        let board = vct_board();
        let mut reused = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        let mut verdicts = vec![];

        // Deliberately out of order, and both attackers, so that a larger
        // limit is asked before a smaller one and the other way round.
        for &limit in &[4u8, 2, 6, 1, 5, 3, 6, 2, 4] {
            for attacker in [Black, White] {
                let state = &mut VCTState::from_board(&board, attacker, limit, 1);
                let got = reused.solve(state, budget).is_proven();

                let mut fresh = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
                let state = &mut VCTState::from_board(&board, attacker, limit, 1);
                let want = fresh.solve(state, &mut NodeBudget::unlimited()).is_proven();

                assert_eq!(got, want, "{attacker:?} at limit {limit}");
                verdicts.push(got);
            }
        }
        assert!(verdicts.iter().any(|&v| v), "expected some proven");
        assert!(verdicts.iter().any(|&v| !v), "expected some disproven");
    }

    /// With only a budget, the fukumi position is proven: under the
    /// `threat_limit` 3 it needs, and longer than the 4 attacks of
    /// `test_vct_fukumi_move`, which no `limit` bounds now.
    #[test]
    fn test_vct_auto() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . x o o . . . . . .
         . . . . . o o o x x . . . . .
         . . . . o x x x x o . . . . .
         . . . x . x o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        // `limit` and `threat_limit` are ignored.
        let limits = SolveLimits::new(1).with_threat_limit(0);

        let (result, stats) = solve_with_stats(VCTAUTO, &board, Black, limits);
        let mate = result.into_mate().expect("proven");
        assert!(mate.n_attacks() > 4, "{}", mate.n_attacks());

        // Short of the nodes it took, it gives up, within the budget.
        let max_nodes = stats.nodes / 2;
        let (result, stats) =
            solve_with_stats(VCTAUTO, &board, Black, limits.with_max_nodes(max_nodes));
        assert!(result.is_unknown());
        assert!(stats.nodes <= max_nodes);

        // An empty board has no VCT at any length: disproven, not aborted.
        let limits = limits.with_max_nodes(1_000);
        assert!(solve(VCTAUTO, &Board::new(), Black, limits).is_disproven());
        Ok(())
    }

    /// One solver answers under several `threat_limit`s, in any order, as
    /// fresh solvers do: what it learned under one is not read under
    /// another. The fukumi position has a VCT in 7 only with threats of
    /// three moves (`test_vct_fukumi_move`), so the answers differ.
    #[test]
    fn test_reused_solver_matches_fresh_across_threat_limits() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . x o o . . . . . .
         . . . . . o o o x x . . . . .
         . . . . o x x x x o . . . . .
         . . . x . x o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let mut reused = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        let mut verdicts = vec![];
        for &(limit, threat_limit) in &[(7u8, 3u8), (7, 2), (6, 3), (7, 1), (7, 3), (7, 2)] {
            let state = &mut VCTState::from_board(&board, Black, limit, threat_limit);
            let got = reused.solve(state, budget).is_proven();

            let mut fresh = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
            let state = &mut VCTState::from_board(&board, Black, limit, threat_limit);
            let want = fresh.solve(state, &mut NodeBudget::unlimited()).is_proven();

            assert_eq!(got, want, "limit {limit}, threat_limit {threat_limit}");
            verdicts.push(got);
        }
        assert_eq!(verdicts, [true, false, false, false, true, false]);
        Ok(())
    }

    /// The assumption everything above rests on: the verdict only ever turns
    /// from "no mate" to "mate" as the limit grows, never back.
    ///
    /// For limits from `transfer_from` up this follows from the search: the
    /// candidate generation no longer moves with the limit there, so a
    /// winning line within `limit` is still a winning line when more moves
    /// are allowed. Below it the nested VCF depths do move, so this is a
    /// measurement rather than an argument — hence the test. Slow, so
    /// `#[ignore]`d; run with `cargo test --release -- --ignored`.
    #[test]
    #[ignore]
    fn test_verdict_is_monotone_in_limit() {
        let mut checked = 0;
        let mut breaks = vec![];
        for seed in 1..=60u64 {
            for n in [8usize, 12, 16] {
                let board = random_position(seed, n);
                if board.rows(Black, Five).next().is_some()
                    || board.rows(White, Five).next().is_some()
                    || board.rows(Black, Overlined).next().is_some()
                {
                    continue;
                }
                for attacker in [Black, White] {
                    if board.rows(attacker, Four).next().is_some() {
                        continue;
                    }
                    let verdicts: String = (1..=5u8)
                        .map(|limit| {
                            let limits = SolveLimits::new(limit)
                                .with_threat_limit(1)
                                .with_max_nodes(200_000);
                            match solve(VCTDFPNS, &board, attacker, limits) {
                                SolveResult::Proven(_) => 'P',
                                SolveResult::Disproven => 'D',
                                // An abort is no information either way.
                                SolveResult::Unknown => '?',
                            }
                        })
                        .collect();
                    checked += 1;
                    if let Some(first) = verdicts.find('P')
                        && verdicts[first..].contains('D')
                    {
                        breaks.push(format!("seed {seed} n {n} {attacker:?} {verdicts}"));
                    }
                }
            }
        }
        assert!(checked > 300, "only checked {checked}");
        assert!(breaks.is_empty(), "not monotone: {breaks:?}");
    }

    /// Deterministic pseudo-random position with `n` stones near the centre.
    fn random_position(seed: u64, n: usize) -> Board {
        let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let mut board = Board::new();
        let mut r = Black;
        let mut placed = 0;
        while placed < n {
            if board.rows(Black, Five).next().is_some() || board.rows(White, Five).next().is_some()
            {
                break;
            }
            let p = Point((next() % 9) as u8 + 3, (next() % 9) as u8 + 3);
            if board.stone(p).is_some() {
                continue;
            }
            board.put(r, p);
            placed += 1;
            r = r.opponent();
        }
        board
    }

    /// The defender's side of a threat: find the attacker's mate, ask which
    /// moves are worth trying against it, and check that one of them really
    /// makes it go away.
    #[test]
    fn test_threat_defences() {
        let board = vct_board();
        let limits = SolveLimits::new(4).with_threat_limit(1);

        let threat = solve(VCTDFPNS, &board, Black, limits)
            .into_mate()
            .expect("Black has a VCT");

        let state = VCTState::from_board(&board, White, 4, 1);
        let defences = state.threat_defences(&threat);
        assert!(defences.contains(&threat.path[0]));

        let stops_it = defences.iter().any(|&p| {
            board.forbidden(p).is_none()
                && solve(VCTDFPNS, &board.with_stone(White, p), Black, limits).is_disproven()
        });
        assert!(stops_it, "no candidate stops the VCT");
    }

    /// Asking whether the side to move is under a threat: let them pass and
    /// see whether the opponent then has a VCF.
    #[test]
    fn test_threat_after_pass() {
        let board = vct_board();
        let budget = &mut NodeBudget::unlimited();

        // White to move. After the pass it is Black's turn, and Black has no
        // VCF yet in this position.
        let mut game = Game::new(&board, White);
        game.play(None);
        let state = &mut VCFState::new(game, 5);
        assert!(DFSSolver::new().solve(state, budget).is_disproven());

        // After Black's first VCT move it is a threat: passing loses to a VCF.
        let board = board.with_stone(Black, threat_start());
        let mut game = Game::new(&board, White);
        game.play(None);
        let state = &mut VCFState::new(game, 5);
        assert!(DFSSolver::new().solve(state, budget).is_proven());
    }

    #[test]
    fn test_stop_signal_aborts_cleanly() {
        let board = vct_board();
        let always_stop = || true;

        // One-shot solve/prove with a stop signal and no node limit.
        for mode in [VCFDFS, VCTDFPNS, VCTAUTO] {
            let limits = SolveLimits::new(5).with_threat_limit(2);
            let mut b1 = NodeBudget::unlimited().with_stop(&always_stop);
            assert!(solve_with_budget(mode, &board, Black, limits, &mut b1).is_unknown());
            let mut b2 = NodeBudget::unlimited().with_stop(&always_stop);
            assert!(prove_with_budget(mode, &board, Black, limits, &mut b2).is_unknown());
        }

        // Reusing a solver after a stop-signal abort still finds the mate.
        let mut solver = DFPNSVCTSolver::new(DEFAULT_DEFENDER_VCF_DEPTH);
        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        let mut stopped = NodeBudget::unlimited().with_stop(&always_stop);
        assert!(solver.solve(state, &mut stopped).is_unknown());
        assert!(stopped.is_exhausted());

        let state = &mut VCTState::from_board(&board, Black, 4, 1);
        let mut unlimited = NodeBudget::unlimited();
        let mate = solver
            .solve(state, &mut unlimited)
            .into_mate()
            .expect("reused solver must still prove the VCT");
        assert_eq!(path_string(mate), VCT_SOLUTION);
    }

    fn threat_start() -> Point {
        VCT_SOLUTION.split(',').next().unwrap().parse().unwrap()
    }

    fn path_string(maybe_mate: impl Into<Option<Mate>>) -> String {
        maybe_mate
            .into()
            .map(|m| Points(m.path).to_string())
            .unwrap_or("".to_string())
    }
}
