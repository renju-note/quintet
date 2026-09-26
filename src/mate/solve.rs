use super::budget::NodeBudget;
use super::game::*;
use super::mate::*;
use super::solver::Solver;
use super::vcf::*;
use super::vct::*;
use crate::board::Player::*;
use crate::board::SequenceKind::*;
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
    if let Err(e) = validate(board, attacker) {
        let result = match e {
            Some(mate) => SolveResult::Proven(mate),
            None => SolveResult::Disproven,
        };
        return (result, SolveStats::default());
    }
    let limit = limits.limit;
    let threat_limit = limits.threat_limit;
    let defender_vcf_depth = limits.defender_vcf_depth;
    let budget = &mut limits.budget();
    let (maybe_mate, memo_len) = match mode {
        VCFDFS => {
            let state = &mut VCFState::init(board, attacker, limit);
            let mut solver = DFSSolver::init();
            (solver.solve(state, budget), solver.memo_len())
        }
        VCTDFS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = DFSVCTSolver::init(threat_limit, defender_vcf_depth);
            (solver.solve(state, budget), solver.memo_len())
        }
        VCTPNS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = PNSVCTSolver::init(threat_limit, defender_vcf_depth);
            (solver.solve(state, budget), solver.memo_len())
        }
        VCTDFPNS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = DFPNSVCTSolver::init(threat_limit, defender_vcf_depth);
            (solver.solve(state, budget), solver.memo_len())
        }
        // VCFIDDFS and VCTIDDFS have no solver of their own here.
        _ => (None, 0),
    };
    let result = match maybe_mate {
        Some(mate) => SolveResult::Proven(mate),
        None if budget.is_exhausted() => SolveResult::Aborted,
        None => SolveResult::Disproven,
    };
    let stats = SolveStats {
        nodes: budget.nodes(),
        memo_len,
    };
    (result, stats)
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SolveMode {
    VCFDFS,
    VCFIDDFS,
    VCTDFS,
    VCTIDDFS,
    VCTPNS,
    VCTDFPNS,
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

    fn budget(&self) -> NodeBudget {
        match self.max_nodes {
            Some(max_nodes) => NodeBudget::new(max_nodes),
            None => NodeBudget::unlimited(),
        }
    }
}

/// What a search concluded.
///
/// `Disproven` and `Aborted` are both "no mate to report", but only
/// `Disproven` says there is none: `Aborted` means the search ran out of
/// budget and the position is still open.
#[derive(Debug, PartialEq, Eq)]
pub enum SolveResult {
    /// The attacker wins, by this line.
    Proven(Mate),
    /// The attacker has no mate within the limits.
    Disproven,
    /// The budget ran out first; nothing is known.
    Aborted,
}

impl SolveResult {
    pub fn is_proven(&self) -> bool {
        matches!(self, Self::Proven(_))
    }

    pub fn is_disproven(&self) -> bool {
        matches!(self, Self::Disproven)
    }

    pub fn is_aborted(&self) -> bool {
        matches!(self, Self::Aborted)
    }

    pub fn mate(&self) -> Option<&Mate> {
        match self {
            Self::Proven(mate) => Some(mate),
            _ => None,
        }
    }

    pub fn into_mate(self) -> Option<Mate> {
        match self {
            Self::Proven(mate) => Some(mate),
            _ => None,
        }
    }
}

fn validate(board: &Board, attacker: Player) -> Result<(), Option<Mate>> {
    if board.sequences(Black, Five).next().is_some() {
        return Err(None);
    }
    if board.sequences(White, Five).next().is_some() {
        return Err(None);
    }
    if board.sequences(Black, Overlined).next().is_some() {
        return Err(None);
    }
    if board.sequences(attacker, Four).next().is_some() {
        return Err(Some(Mate::new(Unknown, vec![])));
    }
    Ok(())
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

        let solution = "I10,I6,I11,I8,J11,J8,G8";

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

        let solution = "G12,E10,F12,I12,H14,H13,F14,G13,F13,F11,E14,D15,G14";

        let result = solve_mate(VCTDFS, 7, &board, Black, 3);
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

        let solution = "J4,G7,I4,I3,E6,G4,G6";
        let result = solve_mate(VCTDFS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTDFS, 4, &board, White, 1);
        assert!(result.is_none());

        let solution = "J4,K3,I4,I3,F8,G7,E6,G9,G6";
        let result = solve_mate(VCTDFPNS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve_mate(VCTPNS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

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
        assert_eq!(result, SolveResult::Aborted);
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

        // The attacker has a four already and wins next move, which has no
        // line worth showing.
        let four = "H8,I8,J8,K8/G8";
        assert_eq!(
            result(four, Black)?,
            SolveResult::Proven(Mate::new(Unknown, vec![]))
        );
        Ok(())
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

    /// A search that gave up must not leave anything behind that hides the
    /// mate from the next, longer search on the same solver.
    #[test]
    fn test_abort_leaves_no_wrong_memo() {
        let board = vct_board();
        for max_nodes in [1, 2, 3, 5, 8, 13, 21, 34, 55, 89] {
            let mut solver = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);

            let budget = &mut NodeBudget::new(max_nodes);
            let state = &mut VCTState::init(&board, Black, 4);
            let aborted = solver.solve(state, budget);

            let budget = &mut NodeBudget::unlimited();
            let state = &mut VCTState::init(&board, Black, 4);
            let result = path_string(solver.solve(state, budget));
            assert_eq!(result, VCT_SOLUTION, "after {} nodes", max_nodes);

            // Whatever the small budget did find must have been right too.
            if let Some(mate) = aborted {
                assert_eq!(Points(mate.path).to_string(), VCT_SOLUTION);
            }
        }
    }

    /// One solver, one budget, several positions: the budget adds up and
    /// `clear` throws the tables away.
    #[test]
    fn test_reused_solver() {
        let board = vct_board();
        let mut solver = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();

        let state = &mut VCTState::init(&board, Black, 4);
        assert_eq!(path_string(solver.solve(state, budget)), VCT_SOLUTION);
        let first = budget.nodes();
        assert!(first > 0);

        // The second search reuses the tables, so it costs less than the first.
        let state = &mut VCTState::init(&board, Black, 4);
        assert_eq!(path_string(solver.solve(state, budget)), VCT_SOLUTION);
        let second = budget.nodes() - first;
        assert!(second < first);

        solver.clear();
        let state = &mut VCTState::init(&board, Black, 4);
        assert_eq!(path_string(solver.solve(state, budget)), VCT_SOLUTION);
        let third = budget.nodes() - first - second;
        assert_eq!(third, first);
    }

    /// Both attackers on one solver. Every memo is keyed by the attacker, so
    /// what the Black searches leave behind must not reach the White ones.
    #[test]
    fn test_reused_solver_both_attackers() {
        let board = vct_board();
        let mut reused = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();

        for round in 0..3 {
            for attacker in [Black, White] {
                let state = &mut VCTState::init(&board, attacker, 4);
                let got = reused.solve(state, budget).is_some();

                let mut fresh = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
                let state = &mut VCTState::init(&board, attacker, 4);
                let want = fresh.solve(state, &mut NodeBudget::unlimited()).is_some();

                assert_eq!(got, want, "round {round}, {attacker:?} attacking");
            }
        }
        // Black wins here and White does not, so the two answers differ and a
        // key that ignored the attacker would have had to get one of them
        // wrong.
        let state = &mut VCTState::init(&board, Black, 4);
        assert!(reused.solve(state, budget).is_some());
        let state = &mut VCTState::init(&board, White, 4);
        assert!(reused.solve(state, budget).is_none());
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
        let defences = VCTState::init(&board, White, 4).threat_defences(&threat);

        let mut reused = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        let mut seen = std::collections::HashSet::new();
        let mut checked = 0;
        for p in defences {
            if !seen.insert(p) || board.forbidden(p).is_some() {
                continue;
            }
            let next = board.put(White, p);

            let state = &mut VCTState::init(&next, Black, 4);
            let got = reused.solve(state, budget).is_some();

            let mut fresh = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
            let state = &mut VCTState::init(&next, Black, 4);
            let want = fresh.solve(state, &mut NodeBudget::unlimited()).is_some();

            assert_eq!(got, want, "after White {p}");
            checked += 1;
        }
        assert!(checked > 1, "expected several defences to check");
    }

    /// A reused solver must not grow with the number of searches. One search
    /// is held down by its node budget; a series of them is held down by the
    /// generations, which settle the memos at about two searches' worth.
    #[test]
    fn test_reused_solver_memo_stays_bounded() {
        let board = vct_board();
        let budget = &mut NodeBudget::unlimited();
        let positions: Vec<_> = board
            .empties()
            .take(24)
            .enumerate()
            .filter(|(_, p)| board.forbidden(*p).is_none())
            .map(|(i, p)| board.put(if i % 2 == 0 { White } else { Black }, p))
            .collect();
        assert!(positions.len() > 20);

        // Small enough that every search drops what the ones before it left.
        let mut solver = DFPNSVCTSolver::with_carry_capacity(1, DEFAULT_DEFENDER_VCF_DEPTH, 0);
        let mut early = 0;
        let mut total_alone = 0;
        for (i, b) in positions.iter().enumerate() {
            solver.solve(&mut VCTState::init(b, Black, 4), budget);
            if i == 3 {
                early = solver.memo_len();
            }
            let mut alone = DFPNSVCTSolver::with_carry_capacity(1, DEFAULT_DEFENDER_VCF_DEPTH, 0);
            alone.solve(
                &mut VCTState::init(b, Black, 4),
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
        let mut solver = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();

        // Black has a VCT in 4 and none in 3.
        assert!(
            solver
                .solve(&mut VCTState::init(&board, Black, 3), budget)
                .is_none()
        );
        let after_three = budget.nodes();
        assert!(
            solver
                .solve(&mut VCTState::init(&board, Black, 4), budget)
                .is_some()
        );
        let four = budget.nodes() - after_three;

        // Proven at 4, so limits 5 and 6 are answered from the bound.
        for limit in [5u8, 6] {
            let before = budget.nodes();
            assert!(
                solver
                    .solve(&mut VCTState::init(&board, Black, limit), budget)
                    .is_some(),
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
                    .solve(&mut VCTState::init(&board, Black, limit), budget)
                    .is_none(),
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
        let mut reused = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
        let budget = &mut NodeBudget::unlimited();
        let mut verdicts = vec![];

        // Deliberately out of order, and both attackers, so that a larger
        // limit is asked before a smaller one and the other way round.
        for &limit in &[4u8, 2, 6, 1, 5, 3, 6, 2, 4] {
            for attacker in [Black, White] {
                let state = &mut VCTState::init(&board, attacker, limit);
                let got = reused.solve(state, budget).is_some();

                let mut fresh = DFPNSVCTSolver::init(1, DEFAULT_DEFENDER_VCF_DEPTH);
                let state = &mut VCTState::init(&board, attacker, limit);
                let want = fresh.solve(state, &mut NodeBudget::unlimited()).is_some();

                assert_eq!(got, want, "{attacker:?} at limit {limit}");
                verdicts.push(got);
            }
        }
        assert!(verdicts.iter().any(|&v| v), "expected some proven");
        assert!(verdicts.iter().any(|&v| !v), "expected some disproven");
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
                if board.sequences(Black, Five).next().is_some()
                    || board.sequences(White, Five).next().is_some()
                    || board.sequences(Black, Overlined).next().is_some()
                {
                    continue;
                }
                for attacker in [Black, White] {
                    if board.sequences(attacker, Four).next().is_some() {
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
                                SolveResult::Aborted => '?',
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
            if board.sequences(Black, Five).next().is_some()
                || board.sequences(White, Five).next().is_some()
            {
                break;
            }
            let p = Point((next() % 9) as u8 + 3, (next() % 9) as u8 + 3);
            if board.stone(p).is_some() {
                continue;
            }
            board.put_mut(r, p);
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

        let state = VCTState::init(&board, White, 4);
        let defences = state.threat_defences(&threat);
        assert!(defences.contains(&threat.path[0]));

        let stops_it = defences.iter().any(|&p| {
            board.forbidden(p).is_none()
                && solve(VCTDFPNS, &board.put(White, p), Black, limits).is_disproven()
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
        let mut game = Game::init(&board, White);
        game.play(None);
        let state = &mut VCFState::new(game, 5);
        assert!(DFSSolver::init().solve(state, budget).is_none());

        // After Black's first VCT move it is a threat: passing loses to a VCF.
        let board = board.put(Black, threat_start());
        let mut game = Game::init(&board, White);
        game.play(None);
        let state = &mut VCFState::new(game, 5);
        assert!(DFSSolver::init().solve(state, budget).is_some());
    }

    fn threat_start() -> Point {
        VCT_SOLUTION.split(',').next().unwrap().parse().unwrap()
    }

    fn path_string(maybe_mate: Option<Mate>) -> String {
        maybe_mate
            .map(|m| Points(m.path).to_string())
            .unwrap_or("".to_string())
    }
}
