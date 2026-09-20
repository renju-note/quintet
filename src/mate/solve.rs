use super::budget::NodeBudget;
use super::game::*;
use super::mate::*;
use super::vcf::*;
use super::vct::*;
use crate::board::Player::*;
use crate::board::StructureKind::*;
use crate::board::*;
use std::convert::TryFrom;
use std::str::FromStr;

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
/// `limit` and `threat_limit` are the two depths that `solve` has always
/// taken; `defender_vcf_depth` used to be fixed at 2, and `max_nodes` is the
/// budget (see [`NodeBudget`]). Build one with [`SolveLimits::new`] and the
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

/// What `defender_vcf_depth` was fixed at before it could be given.
pub const DEFAULT_DEFENDER_VCF_DEPTH: u8 = 2;

pub fn solve(
    mode: SolveMode,
    limit: u8,
    board: &Board,
    attacker: Player,
    threat_limit: u8,
) -> Option<Mate> {
    let limits = SolveLimits::new(limit).with_threat_limit(threat_limit);
    solve_limited(mode, board, attacker, limits).into_mate()
}

/// Like [`solve`], but with a node budget and a result that tells "no mate"
/// apart from "gave up".
///
/// ```
/// use quintet::board::{Board, Player};
/// use quintet::mate::{SolveLimits, SolveMode, solve_limited};
///
/// let board = Board::new();
/// let limits = SolveLimits::new(5).with_max_nodes(1_000);
/// let result = solve_limited(SolveMode::VCFDFS, &board, Player::Black, limits);
/// assert!(result.is_disproven());
/// ```
pub fn solve_limited(
    mode: SolveMode,
    board: &Board,
    attacker: Player,
    limits: SolveLimits,
) -> SolveResult {
    if let Err(e) = validate(board, attacker) {
        return match e {
            Some(mate) => SolveResult::Proven(mate),
            None => SolveResult::Disproven,
        };
    }
    let limit = limits.limit;
    let threat_limit = limits.threat_limit;
    let defender_vcf_depth = limits.defender_vcf_depth;
    let budget = &mut limits.budget();
    let maybe_mate = match mode {
        VCFDFS => {
            let state = &mut VCFState::init(board, attacker, limit);
            let mut solver = DFSSolver::init();
            solver.solve(state, budget)
        }
        VCTDFS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = DFSVCTSolver::init(threat_limit, defender_vcf_depth);
            solver.solve(state, budget)
        }
        VCTPNS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = PNSVCTSolver::init(threat_limit, defender_vcf_depth);
            solver.solve(state, budget)
        }
        VCTDFPNS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = DFPNSVCTSolver::init(threat_limit, defender_vcf_depth);
            solver.solve(state, budget)
        }
        // VCFIDDFS and VCTIDDFS have no solver of their own here.
        _ => None,
    };
    match maybe_mate {
        Some(mate) => SolveResult::Proven(mate),
        None if budget.is_exhausted() => SolveResult::Aborted,
        None => SolveResult::Disproven,
    }
}

fn validate(board: &Board, attacker: Player) -> Result<(), Option<Mate>> {
    if board.structures(Black, Five).next().is_some() {
        return Err(None);
    }
    if board.structures(White, Five).next().is_some() {
        return Err(None);
    }
    if board.structures(Black, Overlined).next().is_some() {
        return Err(None);
    }
    if board.structures(attacker, Four).next().is_some() {
        return Err(Some(Mate::new(Unknown, vec![])));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let result = solve(VCFDFS, 12, &board, Black, 0);
        assert_eq!(path_string(result), solution);

        let result = solve(VCFDFS, 11, &board, Black, 0);
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

        let result = solve(VCFDFS, 5, &board, White, 0);
        assert_eq!(path_string(result), solution);

        let result = solve(VCFDFS, 4, &board, White, 0);
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

        let result = solve(VCFDFS, u8::MAX, &board, Black, u8::MAX);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

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

        let result = solve(VCFDFS, 3, &board, Black, 0);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

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

        let result = solve(VCFDFS, u8::MAX, &board, Black, 0);
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

        let result = solve(VCTDFS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFS, 3, &board, Black, 1);
        assert!(result.is_none());

        let result = solve(VCTPNS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFPNS, 4, &board, Black, 1);
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

        let result = solve(VCTDFS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFS, 3, &board, White, 1);
        assert!(result.is_none());

        let solution = "I10,I6,I11,I8,J11,J8,G8";

        let result = solve(VCTPNS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFPNS, 4, &board, White, 1);
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

        let result = solve(VCTDFS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFS, 3, &board, White, 1);
        assert!(result.is_none());

        let result = solve(VCTPNS, 4, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFPNS, 4, &board, White, 1);
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

        let result = solve(VCTDFS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFS, 3, &board, Black, 1);
        assert!(result.is_none());

        let result = solve(VCTPNS, 4, &board, Black, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFPNS, 4, &board, Black, 1);
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

        let result = solve(VCTDFS, 7, &board, Black, 3);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFS, 6, &board, Black, 3);
        assert!(result.is_none());

        let result = solve(VCTDFS, 7, &board, Black, 2);
        assert!(result.is_none());

        let result = solve(VCTPNS, 7, &board, Black, 3);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFPNS, 7, &board, Black, 3);
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
        let result = solve(VCTDFS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTDFS, 4, &board, White, 1);
        assert!(result.is_none());

        let solution = "J4,K3,I4,I3,F8,G7,E6,G9,G6";
        let result = solve(VCTDFPNS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        let result = solve(VCTPNS, 5, &board, White, 1);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    #[ignore]
    fn bench_vct_black() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . o x o o . . . . .
         . . . . . . . x x . . . . . .
         . . . . . . x o x o . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . . x . x . . . . .
         . . . . . . x o o o . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "J10,J12,L8,K9,K7,I5,L9,L7,M8,K10,M9,N10,M7,M6,N8,E4,F5,K8,L6,K6,K5";

        let result = solve(VCTDFPNS, 14, &board, Black, 2);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    #[ignore]
    fn bench_vct_white() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . o . . . . . . . .
         . . . . o x x x o o . x . . .
         . . . . . . o x x x x o o . .
         . . . . . o x o o x x . . . .
         . . . . . x . o o . x . . . .
         . . . . x . o . . x o o . . .
         . . . o . x . . x . . . . . .
         . . . . . . o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution =
            "K11,K10,N12,M11,N8,H5,H6,L8,J5,J7,M5,L4,M6,N5,L5,K5,J3,F3,I6,E2,D1,K4,J4,J2,K3";

        let result = solve(VCTDFPNS, 15, &board, White, 2);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    #[ignore]
    fn bench_vct_unstable() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . x o . o o . . . . .
         . . . . . o x x x o . . . . .
         . . . . . . . o x o x . . . .
         . . . . . . . x o x . x . . .
         . . . . . . . x o o o . . . .
         . . . . . . . . o . x . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution =
            "J12,J11,K10,H10,L10,M10,K7,L8,K11,M9,I12,F8,E7,L9,N11,L6,L5,E8,H11,J13,G12,F13,H12";

        // fast
        let result = solve(VCTDFPNS, 12, &board, Black, 3);
        assert_eq!(path_string(result), solution);

        // slow
        // let result = solve(VCTDFPNS, 10, &board, Black, 3);
        // assert_eq!(path_string(result), solution);

        Ok(())
    }

    #[test]
    #[ignore]
    fn bench_vct_small_but_long() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . o x o . . . . . . .
         . . . . . x o x . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . o . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let solution = "F6,E5,E7,D8,G9,H10,F10,E11,H4,I3,J6,I7,I5,G3,D6,C5,J5,H5,J8,J9,I9,J10,J4,J7,K7,L8,M5,L6,K4,I4,J3,J2,K5,L5,K6";

        let result = solve(VCTDFPNS, 255, &board, Black, 3);
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
    fn test_solve_limited_three_values() {
        let board = vct_board();
        let limits = SolveLimits::new(4).with_threat_limit(1);

        let result = solve_limited(VCTDFPNS, &board, Black, limits);
        assert!(result.is_proven());
        assert_eq!(
            Points(result.into_mate().unwrap().path).to_string(),
            VCT_SOLUTION
        );

        // There is no VCT in 3, and the search says so rather than giving up.
        let result = solve_limited(
            VCTDFPNS,
            &board,
            Black,
            SolveLimits::new(3).with_threat_limit(1),
        );
        assert_eq!(result, SolveResult::Disproven);

        // One node is not enough to decide anything.
        let result = solve_limited(VCTDFPNS, &board, Black, limits.with_max_nodes(1));
        assert_eq!(result, SolveResult::Aborted);
    }

    #[test]
    fn test_solve_limited_matches_solve() {
        let board = vct_board();
        let limits = SolveLimits::new(4).with_threat_limit(1);
        for mode in [VCFDFS, VCTDFS, VCTPNS, VCTDFPNS] {
            let expected = path_string(solve(mode, 4, &board, Black, 1));
            let got = path_string(solve_limited(mode, &board, Black, limits).into_mate());
            assert_eq!(got, expected, "{:?}", mode);
        }
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

    /// The defender's side of a threat: find the attacker's mate, ask which
    /// moves are worth trying against it, and check that one of them really
    /// makes it go away.
    #[test]
    fn test_threat_defences() {
        let board = vct_board();
        let limits = SolveLimits::new(4).with_threat_limit(1);

        let threat = solve_limited(VCTDFPNS, &board, Black, limits)
            .into_mate()
            .expect("Black has a VCT");

        let state = VCTState::init(&board, White, 4);
        let defences = state.threat_defences(&threat);
        assert!(defences.contains(&threat.path[0]));

        let stops_it = defences.iter().any(|&p| {
            board.forbidden(p).is_none()
                && solve_limited(VCTDFPNS, &board.put(White, p), Black, limits).is_disproven()
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
