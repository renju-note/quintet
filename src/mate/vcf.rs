use super::super::board::RowKind::*;
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

    for (attack, defence) in AttackDefencePairs::new(state) {
        if state.is_forbidden_move(attack) {
            continue;
        }
        let mut state = state.play(attack);
        if state.row_eyes_along_last_move(Four).len() >= 2 {
            return Some(vec![attack]);
        }
        if state.is_forbidden_move(defence) {
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

pub fn is_solution(state: &GameState, solution: &Vec<Point>) -> bool {
    let mut state = state.clone();
    for i in 0..((solution.len() + 1) / 2) {
        if i * 2 + 1 == solution.len() {
            let attack = solution[i * 2];
            if !AttackDefencePairs::new(&state).any(|(a, _)| a == attack) {
                return false;
            }
        } else {
            let (attack, defence) = (solution[i * 2], solution[i * 2 + 1]);
            if !AttackDefencePairs::new(&state).any(|(a, d)| a == attack && d == defence) {
                return false;
            }
            state.play_mut(attack);
            state.play_mut(defence);
        }
    }
    true
}

struct AttackDefencePairs {
    moves: Vec<(Point, Point)>,
}

impl AttackDefencePairs {
    pub fn new(state: &GameState) -> Self {
        let next_player = state.next_player();
        let last_fours = state.rows_on(state.last_player(), Four, state.last_move());
        let last_four_count = last_fours.len();
        let moves: Vec<(Point, Point)> = if last_four_count >= 2 {
            vec![]
        } else if last_four_count == 1 {
            let last_four_eye = last_fours[0].eye1.unwrap();
            state
                .rows_on(next_player, Sword, last_four_eye)
                .into_iter()
                .map(|s| {
                    let (e1, e2) = (s.eye1.unwrap(), s.eye2.unwrap());
                    if e1 == last_four_eye {
                        Some((e1, e2))
                    } else if e2 == last_four_eye {
                        Some((e2, e1))
                    } else {
                        None
                    }
                })
                .flatten()
                .collect()
        } else {
            state
                .rows(next_player, Sword)
                .into_iter()
                .map(|s| {
                    let (e1, e2) = (s.eye1.unwrap(), s.eye2.unwrap());
                    [(e1, e2), (e2, e1)]
                })
                .flatten()
                .collect()
        };
        AttackDefencePairs { moves: moves }
    }
}

impl Iterator for AttackDefencePairs {
    type Item = (Point, Point);

    fn next(&mut self) -> Option<Self::Item> {
        self.moves.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::board::Player::*;
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
        let state = GameState::new(&board, Black, Point(11, 10));

        let result = solve(&state, 12, &mut HashSet::new());
        let result = result.map(|ps| Points(ps).to_string());
        let solution = "
            G6,H7,J12,K13,G9,F8,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
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
