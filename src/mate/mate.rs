use super::super::board::Player::*;
use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf;
use std::collections::HashSet;

pub fn solve_vcf(board: &Board, player: Player, depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate_initial(board, player) {
        return e;
    }

    let (last_move, last2_move) = choose_last_moves(board, player);
    let state = GameState::new(board.clone(), player, last_move, last2_move);
    let mut searched = HashSet::new();
    vcf::solve(&mut state.clone(), depth, &mut searched)
}

fn validate_initial(board: &Board, player: Player) -> Result<(), Option<Vec<Point>>> {
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
        .sequences(player, Single, 5, player.is_black())
        .next()
        .is_some()
    {
        return Err(Some(vec![]));
    }

    Ok(())
}

fn choose_last_moves(board: &Board, player: Player) -> (Point, Point) {
    let opponent = player.opponent();
    let mut opponent_fours = board.sequences(opponent, Single, 4, opponent.is_black());
    let last_move = if let Some((index, _)) = opponent_fours.next() {
        let start = index.to_point();
        let next = index.walk(1).to_point();
        board.stones(opponent).find(|&s| s == start || s == next)
    } else {
        board.stones(opponent).next()
    };
    let last2_move = board.stones(player).next();
    let default = Point(0, 0);
    (last_move.unwrap_or(default), last2_move.unwrap_or(default))
}
