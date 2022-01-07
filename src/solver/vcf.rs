use super::super::board::*;
use std::collections::HashSet;

pub fn solve(depth: u8, board: &Board, player: Player) -> Option<Vec<Point>> {
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

    let mut zcache = HashSet::new();
    solve_all(depth, board, player, None, &mut zcache)
}

fn solve_all(
    depth: u8,
    board: &Board,
    next_player: Player,
    last_move: Option<Point>,
    searched: &mut HashSet<u64>,
) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // check if already searched (and was dead-end)
    let z_hash = board.zobrist_hash();
    if searched.contains(&z_hash) {
        return None;
    }
    searched.insert(z_hash);

    // Exists opponent's four
    let opponent = next_player.opponent();
    let opponent_four_eyes = match last_move {
        Some(p) => board.row_eyes_along(opponent, RowKind::Four, p),
        None => board.row_eyes(opponent, RowKind::Four),
    };
    if opponent_four_eyes.len() >= 2 {
        return None;
    } else if opponent_four_eyes.len() == 1 {
        let next_move = opponent_four_eyes.into_iter().next().unwrap();
        return solve_one(depth, board, next_player, next_move, searched);
    }

    // Continue four move
    let next_move_candidates = board.row_eyes(next_player, RowKind::Sword);
    for next_move in next_move_candidates {
        if let Some(ps) = solve_one(depth, board, next_player, next_move, searched) {
            return Some(ps);
        }
    }

    None
}

fn solve_one(
    depth: u8,
    board: &Board,
    next_player: Player,
    next_move: Point,
    searched: &mut HashSet<u64>,
) -> Option<Vec<Point>> {
    if next_player.is_black() && board.forbidden(next_move).is_some() {
        return None;
    }

    let mut next_board = board.put(next_player, next_move);
    let next_four_eyes = next_board.row_eyes_along(next_player, RowKind::Four, next_move);
    if next_four_eyes.len() >= 2 {
        Some(vec![next_move])
    } else if next_four_eyes.len() == 1 {
        let opponent = next_player.opponent();
        let next2_move = next_four_eyes.into_iter().next().unwrap();
        if opponent.is_black() && next_board.forbidden(next2_move).is_some() {
            return Some(vec![next_move]);
        }

        next_board.put_mut(opponent, next2_move);
        let next2_board = next_board;
        solve_all(
            depth - 1,
            &next2_board,
            next_player,
            Some(next2_move),
            searched,
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

#[cfg(test)]
mod tests {
    use super::Player::*;
    use super::*;

    #[test]
    fn test_black() -> Result<(), String> {
        // https://renjuportal.com/puzzle/3040/
        let board = "
            ---------------
            --------x--o---
            ----o-xo-------
            -------o--x----
            -------xo--x---
            ------oox-o----
            -----x-xxo-x---
            -------oox-----
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Board>()?;
        let result = solve(12, &board, Black);
        let solution = "
            G6,H7,J12,K13,G9,F8,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .parse::<Points>()?
        .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve(11, &board, Black);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_white() -> Result<(), String> {
        // https://renjuportal.com/puzzle/2990/
        let board = "
            ---------------
            -----------x---
            ----------o----
            ---------x-xo--
            --------x---o--
            -------xxo-x---
            ------oxoo--o--
            -----xoo-------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Board>()?;
        let result = solve(5, &board, White);
        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14"
            .parse::<Points>()?
            .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve(4, &board, White);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_long() -> Result<(), String> {
        // "孤高の露天風呂" by Shigeru Nakamura
        let board = "
            xxx----o-x-ooo-
            xxo---o---o-x-o
            o-----o---xo---
            xo---x---------
            -o---o-----o--o
            xx---x-x----o--
            o----x----xx-x-
            o----x-o-o----o
            -x-xx----------
            ----o---x--x--x
            oo-----o-------
            x------o------o
            x--o------x-o-x
            x-x--x--x----ox
            x--o-o----oo-ox
        "
        .parse::<Board>()?;
        let result = solve(u8::MAX, &board, Black);
        let solution = "
            A7,A6,E2,C4,F3,G4,J1,M1,H1,I1,H3,H2,E3,G3,C3,B3,E5,E4,E1,G1,
            C1,B1,D2,B4,D4,D5,G5,F4,I5,F5,J2,I3,F6,B2,B6,C5,D6,C6,C7,B8,
            E9,D8,G6,H6,J5,K5,J4,J3,I4,G2,E8,F7,J6,J7,K4,L4,K7,L8,L7,K6,
            L12,L14,M2,L3,L2,K2,N4,O5,N5,N3,M6,K8,M4,M5,N7,L5,M7,O7,N6,N8,
            M9,M8,O15,K15,O12,O13,O10,O9,M12,N11,K10,N13,L10,N10,N12,K12,M13,M11,K11,N14,
            J10,I10,J12,I13,J11,J9,H13,I12,J14,J13,I14,H14,G12,E10,G10,G11,
            D13,E12,E13,F13,C13,B13,C11,D10,C10,C12,D11,E11,C9,C8,D9,B9,I11,H11,F14,H12,
            D14,E14,E15,A11,G15,D12,F15
        "
        .parse::<Points>()?
        .into_vec();
        assert_eq!(result, Some(solution));

        Ok(())
    }
}
