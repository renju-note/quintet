use super::super::bitboard::*;
use super::zhash::*;
use std::collections::HashSet;

pub fn solve(depth: u8, board: &mut Board, player: Player) -> Option<Vec<Point>> {
    // Already exists five
    if board.rows(player, RowKind::Five).len() >= 1 {
        return None;
    }
    if board.rows(player.opponent(), RowKind::Five).len() >= 1 {
        return None;
    }

    // Already exists four
    if board.rows(player, RowKind::Four).len() >= 1 {
        return Some(vec![]);
    }

    // Already exists overline
    if board.rows(Player::Black, RowKind::Overline).len() >= 1 {
        return None;
    }

    let zhash = 0;
    let ztable = ZobristTable::new();
    let mut zcache = HashSet::new();
    solve_all(depth, board, player, None, zhash, &ztable, &mut zcache)
}

fn solve_all(
    depth: u8,
    board: &mut Board,
    player: Player,
    prev_move: Option<Point>,
    zhash: u64,
    ztable: &ZobristTable,
    zcache: &mut HashSet<u64>,
) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // cache hit
    if zcache.contains(&zhash) {
        return None;
    }

    // Exists opponent's four
    let opponent = player.opponent();
    let opponent_four_eyes = match prev_move {
        Some(p) => board.row_eyes_along(p, opponent, RowKind::Four),
        None => board.row_eyes(opponent, RowKind::Four),
    };
    if opponent_four_eyes.len() >= 2 {
        zcache.insert(zhash);
        return None;
    } else if opponent_four_eyes.len() == 1 {
        let next_move = opponent_four_eyes.into_iter().next().unwrap();
        let mut board = board.clone();
        let result = solve_one(depth, &mut board, player, next_move, zhash, ztable, zcache);
        if result.is_none() {
            zcache.insert(zhash);
        }
        return result;
    }

    // Continue four move
    let next_move_cands = board.row_eyes(player, RowKind::Sword);
    for next_move in next_move_cands {
        let mut board = board.clone();
        match solve_one(depth, &mut board, player, next_move, zhash, ztable, zcache) {
            Some(ps) => return Some(ps),
            None => continue,
        }
    }

    zcache.insert(zhash);
    None
}

fn solve_one(
    depth: u8,
    board: &mut Board,
    player: Player,
    next_move: Point,
    zhash: u64,
    ztable: &ZobristTable,
    zcache: &mut HashSet<u64>,
) -> Option<Vec<Point>> {
    if player.is_black() && board.forbidden(next_move).is_some() {
        return None;
    }

    board.put(player, next_move);
    let next_zhash = ztable.apply(zhash, player, next_move);
    let next_four_eyes = board.row_eyes_along(next_move, player, RowKind::Four);
    if next_four_eyes.len() >= 2 {
        Some(vec![next_move])
    } else if next_four_eyes.len() == 1 {
        let opponent = player.opponent();
        let next2_move = next_four_eyes.into_iter().next().unwrap();
        if opponent.is_black() && board.forbidden(next2_move).is_some() {
            return Some(vec![next_move]);
        }

        board.put(opponent, next2_move);
        let next2_zhash = ztable.apply(next_zhash, opponent, next2_move);
        solve_all(
            depth - 1,
            board,
            player,
            Some(next2_move),
            next2_zhash,
            ztable,
            zcache,
        )
        .map(|mut ps| {
            let mut result = vec![next_move, next2_move];
            result.append(&mut ps);
            result
        })
    } else {
        None
    }
}
