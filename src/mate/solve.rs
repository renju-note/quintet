use super::game::*;
use super::mate::*;
use super::vcf;
use super::vct;
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

use SolveMode::*;

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

pub fn solve(mode: SolveMode, limit: u8, board: &Board, turn: Player) -> Option<Mate> {
    if let Err(e) = validate(board, turn) {
        return e;
    }
    match mode {
        VCFDFS => {
            let state = &mut vcf::State::init(board.clone(), turn, limit);
            let mut solver = vcf::dfs::Solver::init();
            solver.solve(state)
        }
        VCTDFS => {
            let state = &mut vct::State::init(board.clone(), turn, limit);
            let mut solver = vct::dfs::Solver::init();
            solver.solve(state)
        }
        VCTPNS => {
            let state = &mut vct::State::init(board.clone(), turn, limit);
            let mut solver = vct::pns::Solver::init();
            solver.solve(state)
        }
        VCTDFPNS => {
            let state = &mut vct::State::init(board.clone(), turn, limit);
            let mut solver = vct::dfpns::Solver::init();
            solver.solve(state)
        }
        _ => None,
    }
}

fn validate(board: &Board, turn: Player) -> Result<(), Option<Mate>> {
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
        return Err(Some(Mate::new(Unknown, vec![])));
    }
    Ok(())
}
