use super::game::*;
use super::mate::*;
use super::vcf::*;
use super::vct::*;
use super::vct_lazy::*;
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
    VCTLAZY,
}

pub use SolveMode::*;

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
            20 => Ok(VCTLAZY),
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
            "vct_lazy" => Ok(VCTLAZY),
            _ => Err("Unknown solve mode"),
        }
    }
}

pub fn solve(
    mode: SolveMode,
    limit: u8,
    board: &Board,
    attacker: Player,
    threat_limit: u8,
) -> Option<Mate> {
    if let Err(e) = validate(board, attacker) {
        return e;
    }
    match mode {
        VCFDFS => {
            let state = &mut VCFState::init(board, attacker, limit);
            let mut solver = DFSSolver::init();
            solver.solve(state)
        }
        VCTDFS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = DFSVCTSolver::init(threat_limit, 2);
            solver.solve(state)
        }
        VCTPNS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = PNSVCTSolver::init(threat_limit, 2);
            solver.solve(state)
        }
        VCTDFPNS => {
            let state = &mut VCTState::init(board, attacker, limit);
            let mut solver = DFPNSVCTSolver::init(threat_limit, 2);
            solver.solve(state)
        }
        VCTLAZY => {
            let state = &mut LazyVCTState::init(board, attacker, limit);
            let mut solver = LazyVCTSolver::init();
            solver.solve(state)
        }
        _ => None,
    }
}

fn validate(board: &Board, attacker: Player) -> Result<(), Option<Mate>> {
    if board.structures(Black, Five).next().is_some() {
        return Err(None);
    }
    if board.structures(White, Five).next().is_some() {
        return Err(None);
    }
    if board.structures(Black, OverFive).next().is_some() {
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

        let solution = "F10,G9,I10";

        let result = solve(VCTLAZY, 4, &board, Black, 1);
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

        let solution = "I10,I8,F7,E6,J11";

        let result = solve(VCTLAZY, 4, &board, White, 1);
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

        let solution = "F7,C10,E6,G8,E8,H5,E7";

        let result = solve(VCTLAZY, 4, &board, White, 1);
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

        let solution = "J8,I7,I8,G8,L8,K8,K7";

        let result = solve(VCTLAZY, 4, &board, Black, 1);
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

        let solution = "G12,E10,F12,I12,H14,H13,F14,G13,F13,F11,E14,D15,G14";

        let result = solve(VCTLAZY, 7, &board, Black, 3);
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

        let solution = "K3,G7,I3,I4,L2,J4,L3,K2,J3";
        let result = solve(VCTLAZY, 5, &board, White, 1);
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

        let solution =
            "F6,E5,E7,D8,G9,H10,F10,E11,H4,I3,J6,I7,I5,G3,D6,C5,J5,H5,J8,J9,I9,J10,J4,J7,K7,L8,M5,L6,K4,I4,J3,J2,K5,L5,K6";

        let result = solve(VCTDFPNS, 255, &board, Black, 3);
        assert_eq!(path_string(result), solution);

        Ok(())
    }

    fn path_string(maybe_mate: Option<Mate>) -> String {
        maybe_mate
            .map(|m| Points(m.path).to_string())
            .unwrap_or("".to_string())
    }
}
