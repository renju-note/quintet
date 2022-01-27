use super::super::board::Player::*;
use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf;
use std::collections::HashSet;

pub fn solve_vcf(board: &Board, turn: Player, depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate_initial(board, turn) {
        return e;
    }

    let (last_move, last2_move) = choose_last_moves(board, turn);
    let state = GameState::new(board.clone(), turn, last_move, last2_move);
    let mut searched = HashSet::new();
    vcf::solve(&mut state.clone(), depth, &mut searched)
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

fn choose_last_moves(board: &Board, turn: Player) -> (Point, Point) {
    let last = turn.opponent();
    let mut last_fours = board.sequences(last, Single, 4, last.is_black());
    let last_move = if let Some((index, _)) = last_fours.next() {
        let start = index.to_point();
        let next = index.walk(1).to_point();
        board.stones(last).find(|&s| s == start || s == next)
    } else {
        board.stones(last).next()
    };
    let last2_move = board.stones(turn).next();
    let default = Point(0, 0);
    (last_move.unwrap_or(default), last2_move.unwrap_or(default))
}
