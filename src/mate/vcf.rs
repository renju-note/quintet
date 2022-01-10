use super::super::board::Player::*;
use super::super::board::RowKind::*;
use super::super::board::*;
use super::state::*;
use std::collections::HashSet;
use std::collections::VecDeque;

pub fn solve_vcf(board: &Board, player: Player, depth: u8, trim: bool) -> Option<Vec<Point>> {
    let opponent = player.opponent();

    // Already exists five
    if board.rows(player, Five).len() >= 1 || board.rows(opponent, Five).len() >= 1 {
        return None;
    }

    // Already exists overline
    if board.rows(Black, Overline).len() >= 1 {
        return None;
    }

    // Already exists four
    if board.rows(player, Four).len() >= 1 {
        return Some(vec![]);
    }

    let state = VCFState::new(board, player);
    let mut searched = HashSet::new();
    let solution = solve(&state, depth, &mut searched);

    if trim {
        solution.map(|solution| trim_solution(&state, &solution))
    } else {
        solution
    }
}

fn solve(state: &VCFState, depth: u8, searched: &mut HashSet<u64>) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // check if already searched (and was dead-end)
    let hash = state.board_hash();
    if searched.contains(&hash) {
        return None;
    }
    searched.insert(hash);

    for attack in state.attacks() {
        let mut state = state.play(attack);
        let may_defence = state.defences().next();
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

fn trim_solution(state: &VCFState, solution: &Vec<Point>) -> Vec<Point> {
    let mut result = solution.clone();
    for i in 0..(solution.len() / 2) {
        // remove a pair of moves
        let mut trimmed = result.clone();
        trimmed.remove(2 * i);
        trimmed.remove(2 * i);
        if is_solution(state, &trimmed) {
            result = trim_solution(state, &trimmed);
            break;
        }
    }
    result
}

fn is_solution(state: &VCFState, solution: &Vec<Point>) -> bool {
    let mut state = state.clone();
    for &p in solution.iter() {
        if !state.is_valid_move(p) {
            return false;
        }
        state = state.play(p);
    }
    true
}

#[derive(Clone)]
struct VCFState {
    game_state: GameState,
    attacker: Player,
}

impl VCFState {
    pub fn new(board: &Board, player: Player) -> Self {
        let last_move = Self::choose_last_move(board, player);
        let game_state = GameState::from_board(board, player, last_move);
        Self {
            game_state: game_state.clone(),
            attacker: game_state.next_player(),
        }
    }

    pub fn board_hash(&self) -> u64 {
        self.game_state.board_hash()
    }

    pub fn play_mut(&mut self, next_move: Point) {
        self.game_state.play_mut(next_move);
    }

    pub fn play(&self, next_move: Point) -> Self {
        let mut result = self.clone();
        result.play_mut(next_move);
        result
    }

    pub fn is_valid_move(&self, p: Point) -> bool {
        let valid_moves: HashSet<Point> = if self.game_state.next_player() == self.attacker {
            self.attacks().collect()
        } else {
            self.defences().collect()
        };

        valid_moves.contains(&p)
    }

    pub fn attacks(&self) -> impl Iterator<Item = Point> {
        if self.game_state.next_player() != self.attacker {
            panic!()
        }
        VCFAttacks::new(&self)
    }

    pub fn defences(&self) -> impl Iterator<Item = Point> {
        if self.game_state.last_player() != self.attacker {
            panic!()
        }
        VCFDefences::new(&self)
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
}

struct VCFAttacks {
    state: VCFState,
    last_four_inited: bool,
    last_four_count: usize,
    last_four_closer: Option<Point>,
    next_four_inited: bool,
    next_four_moves: VecDeque<Point>,
}

impl VCFAttacks {
    fn new(state: &VCFState) -> Self {
        Self {
            state: state.clone(),
            last_four_inited: false,
            last_four_count: 0,
            last_four_closer: None,
            next_four_inited: false,
            next_four_moves: VecDeque::new(),
        }
    }

    fn init_last_four(&mut self) {
        if !self.last_four_inited {
            let mut last_four_eyes = self.state.game_state.row_eyes_along_last_move(Four);
            self.last_four_count = last_four_eyes.len();
            if self.last_four_count == 1 {
                self.last_four_closer = last_four_eyes.pop();
            }
            self.last_four_inited = true;
        }
    }

    fn pop_valid_last_four_closer(&mut self) -> Option<Point> {
        self.last_four_closer.take().filter(|&p| {
            self.next_four_moves.iter().find(|&&m| p == m).is_some()
                && !self.state.game_state.is_forbidden_move(p)
        })
    }

    fn init_next_four(&mut self) {
        // TODO: find three eyes first
        if !self.next_four_inited {
            let next_player = self.state.game_state.next_player();
            self.next_four_moves = self.state.game_state.row_eyes(next_player, Sword).into();
            self.next_four_inited = true;
        }
    }

    fn pop_valid_next_four_move(&mut self) -> Option<Point> {
        while let Some(p) = self.next_four_moves.pop_front() {
            if !self.state.game_state.is_forbidden_move(p) {
                return Some(p);
            }
        }
        None
    }
}

impl Iterator for VCFAttacks {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.init_last_four();
        if self.last_four_count >= 2 {
            return None;
        }
        self.init_next_four();
        if self.last_four_count == 1 {
            return self.pop_valid_last_four_closer();
        }
        self.pop_valid_next_four_move()
    }
}

struct VCFDefences {
    state: VCFState,
    last_four_inited: bool,
    last_four_count: usize,
    last_four_closer: Option<Point>,
}

impl VCFDefences {
    fn new(state: &VCFState) -> Self {
        Self {
            state: state.clone(),
            last_four_inited: false,
            last_four_count: 0,
            last_four_closer: None,
        }
    }

    fn init_last_four(&mut self) {
        if !self.last_four_inited {
            let mut last_four_eyes = self.state.game_state.row_eyes_along_last_move(Four);
            self.last_four_count = last_four_eyes.len();
            if self.last_four_count == 1 {
                self.last_four_closer = last_four_eyes.pop();
            }
            self.last_four_inited = true;
        }
    }

    fn pop_valid_last_four_closer(&mut self) -> Option<Point> {
        self.last_four_closer
            .take()
            .filter(|&p| !self.state.game_state.is_forbidden_move(p))
    }
}

impl Iterator for VCFDefences {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.init_last_four();
        if self.last_four_count >= 2 {
            return None;
        }
        self.pop_valid_last_four_closer()
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
        let result = solve_vcf(&board, Black, 12, false);
        let solution = "
            G6,H7,J12,K13,G9,F8,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .parse::<Points>()?
        .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(&board, Black, 11, false);
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
        let result = solve_vcf(&board, White, 5, false).map(|ps| Points(ps).to_string());
        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14".to_string();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(&board, White, 4, false).map(|ps| Points(ps).to_string());
        assert_eq!(result, None);

        Ok(())
    }

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
