use super::game::*;
use super::mate::*;
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
    VCTDFPNS,
}

impl FromStr for SolverKind {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "vcf" => Ok(VCF),
            "vct_dfs" => Ok(VCTDFS),
            "vct_dfpns" => Ok(VCTDFPNS),
            _ => Err("Unknown SolverKind"),
        }
    }
}

use SolverKind::*;

pub fn solve(kind: SolverKind, limit: u8, board: &Board, turn: Player) -> Option<Mate> {
    if let Err(e) = validate(board, turn) {
        return e;
    }
    match kind {
        VCF => {
            let state = &mut vcf::State::init(board.clone(), turn, limit);
            let mut solver = vcf::dfs::Solver::init();
            solver.solve(state)
        }
        VCTDFS => {
            let state = &mut vct::State::init(board.clone(), turn, limit);
            let mut solver = vct::dfs::Solver::init();
            solver.solve(state)
        }
        VCTDFPNS => {
            let state = &mut vct::State::init(board.clone(), turn, limit);
            let searcher = vct::dfpns::Searcher::init();
            let may_table = searcher.search(state);
            may_table.and_then(|table| {
                let mut resolver = vct::resolver::Resolver::init(table);
                resolver.resolve(state)
            })
        }
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
