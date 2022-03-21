use super::vcf;
use super::vct;
use crate::board::Player::*;
use crate::board::StructureKind::*;
use crate::board::*;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SolverKind {
    VCF,
    VCTDFS,
    VCTDFPN,
}

impl FromStr for SolverKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vcf" => Ok(VCF),
            "vct_dfs" => Ok(VCTDFS),
            "vct_dfpn" => Ok(VCTDFPN),
            _ => Err("Unknown SolverKind"),
        }
    }
}

use SolverKind::*;

pub fn solve(kind: SolverKind, max_depth: u8, board: &Board, turn: Player) -> Option<Vec<Point>> {
    if let Err(e) = validate(board, turn) {
        return e;
    }
    match kind {
        VCF => {
            let state = &mut vcf::State::init(board.clone(), turn);
            let mut solver = vcf::dfs::Solver::init();
            solver.solve(state, max_depth).map(|s| s.path)
        }
        VCTDFS => {
            let state = &mut vct::State::init(board.clone(), turn);
            let mut solver = vct::dfs::Solver::init();
            solver.solve(state, max_depth).map(|s| s.path)
        }
        VCTDFPN => {
            let state = &mut vct::State::init(board.clone(), turn);
            let mut solver = vct::dfpn::Solver::init();
            solver.solve(state, max_depth).map(|s| s.path)
        }
    }
}

fn validate(board: &Board, turn: Player) -> Result<(), Option<Vec<Point>>> {
    if board.structures(Black, Five).next().is_some() {
        return Err(None);
    }
    if board.structures(White, Five).next().is_some() {
        return Err(None);
    }
    if board.structures(Black, OverFive).next().is_some() {
        return Err(None);
    }
    if board.structures(turn, Four).next().is_some() {
        return Err(Some(vec![]));
    }
    Ok(())
}
