use super::super::board::*;
use super::state::*;
use std::collections::HashSet;

pub fn solve_vcf(depth: u8, board: &Board, player: Player) -> Option<Vec<Point>> {
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

    let state = GameState::from_board(board, player);
    let mut searched = HashSet::new();
    solve(depth, &state, &mut searched)
}

fn solve(depth: u8, state: &GameState, searched: &mut HashSet<u64>) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // check if already searched (and was dead-end)
    let board_hash = state.board_hash();
    if searched.contains(&board_hash) {
        return None;
    }
    searched.insert(board_hash);

    // check opponent's four
    let opponent = state.last_player();
    let opponent_four_eyes = match state.last_move() {
        Some(p) => state.board.row_eyes_along(opponent, RowKind::Four, p),
        None => state.board.row_eyes(opponent, RowKind::Four),
    };
    if opponent_four_eyes.len() >= 2 {
        return None;
    }
    let opponent_four_eye = opponent_four_eyes.into_iter().next();

    // continue four move
    let player = state.next_player();
    let sword_eyes = state.board.row_eyes(player, RowKind::Sword);
    for next_move in sword_eyes {
        if opponent_four_eye.map_or(false, |e| e != next_move) {
            continue;
        }

        if !state.is_legal_move(next_move) {
            continue;
        }

        let next_state = state.play(next_move);

        let next_four_eyes = next_state
            .board
            .row_eyes_along(player, RowKind::Four, next_move);
        if next_four_eyes.len() >= 2 {
            return Some(vec![next_move]);
        }

        let next2_move = next_four_eyes[0]; // exists
        if !next_state.is_legal_move(next2_move) {
            return Some(vec![next_move]);
        }

        let next2_state = next_state.play(next2_move);
        if let Some(mut ps) = solve(depth - 1, &next2_state, searched) {
            let mut result = vec![next_move, next2_move];
            result.append(&mut ps);
            return Some(result);
        }
    }

    None
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
        let result = solve_vcf(12, &board, Black);
        let solution = "
            G6,H7,J12,K13,G9,F8,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .parse::<Points>()?
        .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(11, &board, Black);
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
        let result = solve_vcf(5, &board, White);
        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14"
            .parse::<Points>()?
            .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(4, &board, White);
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
        let result = solve_vcf(u8::MAX, &board, Black);
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
