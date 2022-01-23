use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::state::*;
use std::collections::HashSet;

pub fn solve(state: &GameState, depth: u8, searched: &mut HashSet<u64>) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    let hash = state.board_hash();
    if searched.contains(&hash) {
        return None;
    }
    searched.insert(hash);

    let move_pairs = collect_four_move_pairs(state);
    for (attack, defence) in move_pairs {
        if state.is_forbidden_move(attack) {
            continue;
        }
        let mut state = state.play(attack);
        if state.won_by_last() {
            return Some(vec![attack]);
        }
        state.play_mut(defence);
        if let Some(mut ps) = solve(&state, depth - 1, searched) {
            let mut result = vec![attack, defence];
            result.append(&mut ps);
            return Some(result);
        }
    }
    None
}

fn collect_four_move_pairs(state: &GameState) -> Vec<(Point, Point)> {
    let mut last_four_eyes = state.last_four_eyes();
    if let Some(last_four_eye) = last_four_eyes.next() {
        if last_four_eyes.next().is_some() {
            vec![]
        } else {
            state
                .sequences_on(last_four_eye, state.next_player(), Single, 3)
                .flat_map(|(i, s)| {
                    let eyes = s.eyes();
                    let e1 = i.walk(eyes[0] as i8).to_point();
                    let e2 = i.walk(eyes[1] as i8).to_point();
                    if e1 == last_four_eye {
                        Some((e1, e2))
                    } else if e2 == last_four_eye {
                        Some((e2, e1))
                    } else {
                        None
                    }
                })
                .collect()
        }
    } else {
        state
            .sequences(state.next_player(), Single, 3)
            .flat_map(|(i, s)| {
                let eyes = s.eyes();
                let e1 = i.walk(eyes[0] as i8).to_point();
                let e2 = i.walk(eyes[1] as i8).to_point();
                [(e1, e2), (e2, e1)]
            })
            .collect()
    }
}

pub fn trim(state: &GameState, solution: &Vec<Point>) -> Vec<Point> {
    for i in 0..(solution.len() / 2) {
        let mut trimmed = solution.clone();
        trimmed.remove(2 * i);
        trimmed.remove(2 * i);
        if is_solution(state, &trimmed) {
            return trim(state, &trimmed);
        }
    }
    solution.clone()
}

pub fn is_solution(state: &GameState, cand_moves: &Vec<Point>) -> bool {
    let cand_attack = cand_moves[0];

    let move_pairs = collect_four_move_pairs(state);
    let move_pair = move_pairs.iter().find(|(a, _)| *a == cand_attack);
    if move_pair.is_none() {
        return false;
    }
    let (attack, defence) = *move_pair.unwrap();

    if state.is_forbidden_move(attack) {
        return false;
    }

    let mut state = state.play(cand_attack);
    if cand_moves.len() == 1 {
        return state.won_by_last();
    }

    let cand_defence = cand_moves[1];
    if cand_defence != defence {
        return false;
    }
    state.play_mut(cand_defence);

    let mut rest = cand_moves.clone();
    rest.remove(0);
    rest.remove(0);
    is_solution(&state, &rest)
}

#[cfg(test)]
mod tests {
    use super::super::super::board::Player::*;
    use super::*;

    #[test]
    fn test_black() -> Result<(), String> {
        // https://renjuportal.com/puzzle/3040/
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . o . . .
         . . . . o . x o . . . . . . .
         . . . . . . . o . . x . . . .
         . . . . . . . x o . . x . . .
         . . . . . . o o x . o . . . .
         . . . . . x . x x o . x . . .
         . . . . . . . o o x . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let state = GameState::new(&board, Black, Point(11, 10));

        let result = solve(&state, 12, &mut HashSet::new());
        let result = result.map(|ps| Points(ps).to_string());
        let solution = "
            J12,K13,G9,F8,G6,H7,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,F14,F13,C11
        "
        .split_whitespace()
        .collect();
        assert_eq!(result, Some(solution));

        let result = solve(&state, 11, &mut HashSet::new());
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_white() -> Result<(), String> {
        // https://renjuportal.com/puzzle/2990/
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . x . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . x . x o . .
         . . . . . . . . x . . . o . .
         . . . . . . . x x o . x . . .
         . . . . . . o x o o . . o . .
         . . . . . x o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let state = GameState::new(&board, White, Point(12, 11));

        let result = solve(&state, 5, &mut HashSet::new());
        let result = result.map(|ps| Points(ps).to_string());
        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14".to_string();
        assert_eq!(result, Some(solution));

        let result = solve(&state, 4, &mut HashSet::new());
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_long() -> Result<(), String> {
        // "孤高の露天風呂" by Shigeru Nakamura
        let board = "
         x x x . . . . o . x . o o o .
         x x o . . . o . . . o . x . o
         o . . . . . o . . . x o . . .
         x o . . . x . . . . . . . . .
         . o . . . o . . . . . o . . o
         x x . . . x . x . . . . o . .
         o . . . . x . . . . x x . x .
         o . . . . x . o . o . . . . o
         . x . x x . . . . . . . . . .
         . . . . o . . . x . . x . . x
         o o . . . . . o . . . . . . .
         x . . . . . . o . . . . . . o
         x . . o . . . . . . x . o . x
         x . x . . x . . x . . . . o x
         x . . o . o . . . . o o . o x
        "
        .parse::<Board>()?;
        let state = GameState::new(&board, Black, Point(0, 0));
        let result = solve(&state, u8::MAX, &mut HashSet::new());
        let result = result.map(|ps| Points(ps).to_string());
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
