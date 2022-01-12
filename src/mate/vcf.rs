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

    for attack in Attacks::new(state) {
        let mut state = state.play(attack);
        let may_defence = Defences::new(&state).next();
        if may_defence.is_none() {
            return Some(vec![attack]);
        }
        let defence = may_defence.unwrap();
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
    let mut result = solution.clone();
    for i in 0..(solution.len() / 2) {
        let mut trimmed = result.clone();
        trimmed.remove(2 * i);
        trimmed.remove(2 * i);
        if is_solution(state, &trimmed) {
            result = trim(state, &trimmed);
            break;
        }
    }
    result
}

pub fn is_solution(state: &GameState, solution: &Vec<Point>) -> bool {
    let attacker = state.next_player();
    let mut state = state.clone();
    for &p in solution.iter() {
        if state.next_player() == attacker {
            if !Attacks::new(&state).any(|a| p == a) {
                return false;
            }
        } else {
            if !Defences::new(&state).any(|a| p == a) {
                return false;
            }
        }
        state = state.play(p);
    }
    true
}

struct Attacks {
    searcher: MoveSearcher,
}

impl Attacks {
    pub fn new(state: &GameState) -> Self {
        Attacks {
            searcher: MoveSearcher::new(state),
        }
    }
}

impl Iterator for Attacks {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.searcher.last_four_found() {
            return self
                .searcher
                .pop_last_four_closer()
                .filter(|&p| self.searcher.is_next_four_move(p));
        }
        self.searcher.pop_next_four_move()
    }
}

struct Defences {
    searcher: MoveSearcher,
}

impl Defences {
    pub fn new(state: &GameState) -> Self {
        Defences {
            searcher: MoveSearcher::new(state),
        }
    }
}

impl Iterator for Defences {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.searcher.pop_last_four_closer()
    }
}

struct MoveSearcher {
    state: GameState,
    last_four_inited: bool,
    last_four_count: usize,
    last_four_closer: Option<Point>,
    next_four_inited: bool,
    next_four_moves: Vec<Point>,
}

impl MoveSearcher {
    pub fn new(state: &GameState) -> Self {
        Self {
            state: state.clone(),
            last_four_inited: false,
            last_four_count: 0,
            last_four_closer: None,
            next_four_inited: false,
            next_four_moves: vec![],
        }
    }

    pub fn last_four_found(&mut self) -> bool {
        self.init_last_four();
        self.last_four_count >= 1
    }

    pub fn is_next_four_move(&mut self, p: Point) -> bool {
        self.init_next_four();
        self.next_four_moves.iter().any(|&m| m == p)
    }

    pub fn pop_last_four_closer(&mut self) -> Option<Point> {
        self.init_last_four();
        self.last_four_closer
            .take()
            .filter(|&p| !self.state.is_forbidden_move(p))
    }

    pub fn pop_next_four_move(&mut self) -> Option<Point> {
        self.init_next_four();
        while let Some(p) = self.next_four_moves.pop() {
            if !self.state.is_forbidden_move(p) {
                return Some(p);
            }
        }
        None
    }

    fn init_last_four(&mut self) {
        if self.last_four_inited {
            return;
        }
        let mut last_four_eyes = self.state.row_eyes_along_last_move(Four);
        self.last_four_count = last_four_eyes.len();
        if self.last_four_count == 1 {
            self.last_four_closer = last_four_eyes.pop();
        }
        self.last_four_inited = true;
    }

    fn init_next_four(&mut self) {
        if self.next_four_inited {
            return;
        }
        self.next_four_moves = self.state.row_eyes(self.state.next_player(), Sword);
        self.next_four_moves.reverse(); // pop from first
        self.next_four_inited = true;
    }
}

#[cfg(test)]
mod tests {
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
        let state = GameState::new(&board, Player::Black, Point(11, 10));

        let result = solve(&state, 12, &mut HashSet::new());
        let solution = "
            G6,H7,J12,K13,G9,F8,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .parse::<Points>()?
        .into_vec();
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
        let state = GameState::new(&board, Player::White, Point(12, 11));

        let result = solve(&state, 5, &mut HashSet::new()).map(|ps| Points(ps).to_string());
        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14".to_string();
        assert_eq!(result, Some(solution));

        let result = solve(&state, 4, &mut HashSet::new()).map(|ps| Points(ps).to_string());
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
        let state = GameState::new(&board, Player::Black, Point(0, 0));

        let result = solve(&state, u8::MAX, &mut HashSet::new()).map(|ps| Points(ps).to_string());
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
