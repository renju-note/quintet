use super::super::board::Player::*;
use super::super::board::RowKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf;
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

fn validate_initial(board: &Board, player: Player) -> Result<(), Option<Vec<Point>>> {
    let opponent = player.opponent();

    // Already exists five
    if board.rows(player, Five).next().is_some() || board.rows(opponent, Five).next().is_some() {
        return Err(None);
    }

    // Already exists overline
    if board.rows(Black, Overline).next().is_some() {
        return Err(None);
    }

    // Already exists four
    if board.rows(player, Four).next().is_some() {
        return Err(Some(vec![]));
    }

    Ok(())
}

fn choose_last_move(board: &Board, player: Player) -> Point {
    let opponent = player.opponent();
    let stones = board.stones(opponent);
    if let Some(four) = board.rows(opponent, Four).next() {
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
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . o . . . . . . . . .
         . . x o x . . . . . . . . . .
         . o . . . x o . . o . . . . .
         . . x . o . . . x . . . . . .
         . . . x . o x . . . . . . . .
         . . . . x o x o o o o x . . .
         . . . . o x x x o x x x o x .
         . . . x . o o x . o o o x . .
         . . . o . . x . x o x o . . .
         . . . . x o x o o o x . . . .
         . . . . . x . o . x . . . . .
         . . . . o . x . o . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let result = solve_vcf(&board, White, 10, false);
        let result = result.map(|ps| Points(ps).to_string());
        let expected = "H5,I6,E6,H9,F5,E5,C8,D7,G3,G1,C11,C9,C14,C13,D13".to_string();
        assert_eq!(result, Some(expected));

        let result = solve_vcf(&board, White, 10, true);
        let result = result.map(|ps| Points(ps).to_string());
        let expected = "H5,I6,E6,H9,F5,E5,C8,D7,C11,C9,C14,C13,D13".to_string();
        assert_eq!(result, Some(expected));
        Ok(())
    }
}
