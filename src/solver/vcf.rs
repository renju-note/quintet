use super::super::board::*;
use std::collections::HashSet;

pub fn solve(depth: u8, board: &Board, black: bool) -> Option<Vec<Point>> {
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

    let mut cache = HashSet::new();
    solve_all(depth, board, black, None, &mut cache)
}

fn solve_all(
    depth: u8,
    board: &Board,
    black: bool,
    prev_move: Option<Point>,
    cache: &mut HashSet<MiniBoard>,
) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // cache hit
    let mini_board = board.mini_board();
    if cache.contains(&mini_board) {
        return None;
    }

    // Exists opponent's four
    let opponent_four_eyes = match prev_move {
        Some(p) => board.row_eyes_along(p, !black, RowKind::Four),
        None => board.row_eyes(!black, RowKind::Four),
    };
    if opponent_four_eyes.len() >= 2 {
        cache.insert(mini_board);
        return None;
    } else if opponent_four_eyes.len() == 1 {
        let next_move = opponent_four_eyes.into_iter().next().unwrap();
        let mut board = board.clone();
        let result = solve_one(depth, &mut board, black, next_move, cache);
        if result.is_none() {
            cache.insert(mini_board);
        }
        return result;
    }

    // Continue four move
    let next_move_cands = board.row_eyes(black, RowKind::Sword);
    let mut next_move_cands = next_move_cands.into_iter().collect::<Vec<_>>();
    next_move_cands.sort_unstable();
    for next_move in next_move_cands {
        let mut board = board.clone();
        match solve_one(depth, &mut board, black, next_move, cache) {
            Some(ps) => return Some(ps),
            None => continue,
        }
    }

    cache.insert(mini_board);
    None
}

fn solve_one(
    depth: u8,
    board: &mut Board,
    black: bool,
    next_move: Point,
    cache: &mut HashSet<MiniBoard>,
) -> Option<Vec<Point>> {
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
        solve_all(depth - 1, board, black, Some(next2_move), cache).map(|mut ps| {
            let mut result = vec![next_move, next2_move];
            result.append(&mut ps);
            result
        })
    } else {
        None
    }
}
