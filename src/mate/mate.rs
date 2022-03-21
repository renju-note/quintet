use super::super::board::Player::*;
use super::super::board::StructureKind::*;
use super::super::board::*;
use super::vcf;
use super::vct;

pub fn solve_vcf(board: &Board, turn: Player, max_depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate(board, turn) {
        return e;
    }
    let state = &mut vcf::State::init(board.clone(), turn);
    let mut solver = vcf::dfs::Solver::init();
    solver.solve(state, max_depth).map(|s| s.path)
}

pub fn solve_vct(board: &Board, turn: Player, max_depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate(board, turn) {
        return e;
    }
    let state = &mut vct::State::init(board.clone(), turn);
    let mut solver = vct::dfs::Solver::init();
    solver.solve(state, max_depth).map(|s| s.path)
}

pub fn solve_vct_dfpn(board: &Board, turn: Player, max_depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate(board, turn) {
        return e;
    }
    let state = &mut vct::State::init(board.clone(), turn);
    let mut solver = vct::dfpn::Solver::init();
    solver.solve(state, max_depth).map(|s| s.path)
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
