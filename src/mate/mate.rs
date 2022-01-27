use super::super::board::Player::*;
use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf;
use std::collections::HashSet;

pub fn solve_vcf(board: &Board, player: Player, depth: u8, trim: bool) -> Option<Vec<Point>> {
    if let Err(e) = validate_initial(board, player) {
        return e;
    }

    let (last_move, last2_move) = choose_last_moves(board, player);
    let state = GameState::new(board.clone(), player, last_move, last2_move);
    let mut searched = HashSet::new();
    let solution = vcf::solve(&mut state.clone(), depth, &mut searched);

    if trim {
        solution.map(|solution| vcf::trim(&state, &solution))
    } else {
        solution
    }
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
        let expected = "G1,G3,E6,H9,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13".to_string();
        assert_eq!(result, Some(expected));

        let result = solve_vcf(&board, White, 10, true);
        let result = result.map(|ps| Points(ps).to_string());
        let expected = "E6,H9,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13".to_string();
        assert_eq!(result, Some(expected));
        Ok(())
    }
}
