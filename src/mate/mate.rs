use super::super::board::Player::*;
use super::super::board::RowKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf;
use super::vct;
use std::collections::HashSet;

pub fn solve_vcf(board: &Board, player: Player, depth: u8, trim: bool) -> Option<Vec<Point>> {
    if let Err(e) = validate_initial(board, player) {
        return e;
    }

    let last_move = choose_last_move(board, player);
    let state = GameState::new(board, player, last_move);
    let mut searched = HashSet::new();
    let solution = vcf::solve(&state, depth, &mut searched);

    if trim {
        solution.map(|solution| vcf::trim(&state, &solution))
    } else {
        solution
    }
}

pub fn solve_vct(board: &Board, player: Player, depth: u8) -> Option<Vec<Point>> {
    if let Err(e) = validate_initial(board, player) {
        return e;
    }

    let last_move = choose_last_move(board, player);
    let state = GameState::new(board, player, last_move);
    let mut searched = HashSet::new();
    vct::solve(&state, depth, &mut searched)
}

fn validate_initial(board: &Board, player: Player) -> Result<(), Option<Vec<Point>>> {
    let opponent = player.opponent();

    // Already exists five
    if board.rows(player, Five).len() >= 1 || board.rows(opponent, Five).len() >= 1 {
        return Err(None);
    }

    // Already exists overline
    if board.rows(Black, Overline).len() >= 1 {
        return Err(None);
    }

    // Already exists four
    if board.rows(player, Four).len() >= 1 {
        return Err(Some(vec![]));
    }

    Ok(())
}

fn choose_last_move(board: &Board, player: Player) -> Point {
    let opponent = player.opponent();
    let stones = board.stones(opponent);
    if let Some(four) = board.rows(opponent, Four).iter().next() {
        stones
            .into_iter()
            .find(|&s| s == four.start || s == four.end)
    } else {
        stones.into_iter().next()
    }
    .unwrap_or(Point(0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim() -> Result<(), String> {
        let board = "
            ---------------
            ---------------
            -----o---------
            --xox----------
            -o---xo--o-----
            --x-o---x------
            ---x-ox--------
            ----xoxoooox---
            ----oxxxoxxxox-
            ---x-oox-ooox--
            ---o--x-xoxo---
            ----xoxooox----
            -----x-o-x-----
            ----o-x-o------
            ---------------
        "
        .parse::<Board>()?;
        let result = solve_vcf(&board, White, 10, false).map(|ps| Points(ps).to_string());
        let solution = "E6,H9,G1,G3,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13".to_string();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(&board, White, 10, true).map(|ps| Points(ps).to_string());
        let solution = "E6,H9,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13".to_string();
        assert_eq!(result, Some(solution));
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
        let result = solve_vcf(&board, Black, u8::MAX, false).map(|ps| Points(ps).to_string());
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
        .split_whitespace()
        .collect();
        assert_eq!(result, Some(solution));

        Ok(())
    }
}
