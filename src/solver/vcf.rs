use super::super::analyzer::*;
use super::super::board::*;

pub fn solve(depth: u8, board: &Board, black: bool) -> Option<Vec<Point>> {
    // Already exists five
    if rows(board, black, RowKind::Five).len() >= 1 {
        return None;
    }
    if rows(board, !black, RowKind::Five).len() >= 1 {
        return None;
    }

    // Already exists four
    if rows(board, black, RowKind::Four).len() >= 1 {
        return Some(vec![]);
    }

    solve_all(depth, board, black, None)
}

fn solve_all(
    depth: u8,
    board: &Board,
    black: bool,
    prev_move: Option<&Point>,
) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // Exists opponent's four
    let op_four_eyes = match prev_move {
        Some(p) => row_eyes_around(board, !black, RowKind::Four, p),
        None => row_eyes(board, !black, RowKind::Four),
    };
    if op_four_eyes.len() >= 2 {
        return None;
    } else if op_four_eyes.len() == 1 {
        let next_move = &op_four_eyes[0];
        return solve_one(depth, board, black, next_move);
    }

    // Continue four move
    let next_move_cands = row_eyes(board, black, RowKind::Sword);
    for next_move in &next_move_cands {
        match solve_one(depth, board, black, next_move) {
            Some(ps) => return Some(ps),
            None => continue,
        }
    }

    None
}

fn solve_one(depth: u8, board: &Board, black: bool, next_move: &Point) -> Option<Vec<Point>> {
    if black && forbidden(board, next_move).is_some() {
        return None;
    }
    let next_board = board.put(black, next_move);
    let next_four_eyes = row_eyes_around(&next_board, black, RowKind::Four, next_move);
    if next_four_eyes.len() >= 2 {
        Some(vec![*next_move])
    } else if next_four_eyes.len() == 1 {
        let next2_move = &next_four_eyes[0];
        if !black && forbidden(&next_board, next2_move).is_some() {
            return Some(vec![*next_move]);
        }
        let next2_board = next_board.put(!black, next2_move);
        solve_all(depth - 1, &next2_board, black, Some(next2_move)).map(|mut ps| {
            let mut result = vec![*next_move, *next2_move];
            result.append(&mut ps);
            result
        })
    } else {
        None
    }
}
