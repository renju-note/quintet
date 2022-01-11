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
}
