use super::super::board::{forbidden, Board, Point, RowKind};

pub fn solve(depth: u8, board: &Board, black: bool) -> Option<Vec<Point>> {
    let mut board = board.clone();

    // Already exists five
    if board.rows(black, RowKind::Five).len() >= 1 {
        return None;
    }
    if board.rows(!black, RowKind::Five).len() >= 1 {
        return None;
    }

    // Already exists four
    if board.rows(black, RowKind::Four).len() >= 1 {
        return Some(vec![]);
    }

    // Already exists overline
    if board.rows(true, RowKind::Overline).len() >= 1 {
        return None;
    }

    solve_all(depth, &mut board, black, None)
}

fn solve_all(
    depth: u8,
    board: &mut Board,
    black: bool,
    prev_move: Option<Point>,
) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // Exists opponent's four
    let opponent_four_eyes = match prev_move {
        Some(p) => board.row_eyes_along(p, !black, RowKind::Four),
        None => board.row_eyes(!black, RowKind::Four),
    };
    if opponent_four_eyes.len() >= 2 {
        return None;
    } else if opponent_four_eyes.len() == 1 {
        let next_move = opponent_four_eyes.into_iter().next().unwrap();
        return solve_one(depth, board, black, next_move);
    }

    // Continue four move
    let next_move_cands = board.row_eyes(black, RowKind::Sword);
    let mut next_move_cands = next_move_cands.into_iter().collect::<Vec<_>>();
    next_move_cands.sort_unstable();
    for next_move in next_move_cands {
        let mut board = board.clone();
        match solve_one(depth, &mut board, black, next_move) {
            Some(ps) => return Some(ps),
            None => continue,
        }
    }

    None
}

fn solve_one(depth: u8, board: &mut Board, black: bool, next_move: Point) -> Option<Vec<Point>> {
    if black && forbidden(board, next_move).is_some() {
        return None;
    }

    board.put(black, next_move);
    let next_four_eyes = board.row_eyes_along(next_move, black, RowKind::Four);
    if next_four_eyes.len() >= 2 {
        Some(vec![next_move])
    } else if next_four_eyes.len() == 1 {
        let next2_move = next_four_eyes.into_iter().next().unwrap();
        if !black && forbidden(&board, next2_move).is_some() {
            return Some(vec![next_move]);
        }

        board.put(!black, next2_move);
        solve_all(depth - 1, board, black, Some(next2_move)).map(|mut ps| {
            let mut result = vec![next_move, next2_move];
            result.append(&mut ps);
            result
        })
    } else {
        None
    }
}
