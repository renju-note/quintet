use super::super::board::Player::*;
use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::vcf;
use super::vct;

pub fn solve_vcf(board: &Board, turn: Player, depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate_initial(board, turn) {
        return e;
    }

    let state = &mut vcf::State::init(board.clone(), turn);
    let mut solver = vcf::Solver::init();
    solver.solve(state, depth).map(|s| s.path)
}

pub fn solve_vct(board: &Board, turn: Player, depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate_initial(board, turn) {
        return e;
    }

    let state = &mut vct::State::init(board.clone(), turn);
    let mut solver = vct::Solver::init();
    solver.solve(state, depth).map(|s| s.path)
}

fn validate_initial(board: &Board, turn: Player) -> Result<(), Option<Vec<Point>>> {
    // Already exists five
    if board.sequences(Black, Single, 5, true).next().is_some() {
        return Err(None);
    }
    if board.sequences(White, Single, 5, false).next().is_some() {
        return Err(None);
    }

    // Already exists black overline
    if board.sequences(Black, Double, 5, false).next().is_some() {
        return Err(None);
    }

    // Already exists four
    if board
        .sequences(turn, Single, 5, turn.is_black())
        .next()
        .is_some()
    {
        return Err(Some(vec![]));
    }

    Ok(())
}
