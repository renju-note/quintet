use super::super::board::Player::*;
use super::super::board::RowKind::*;
use super::super::board::*;
use super::state::*;
use std::collections::HashSet;

pub fn solve_vcf(depth: u8, board: &Board, player: Player, trim: bool) -> Option<Vec<Point>> {
    let opponent = player.opponent();

    // Already exists five
    if board.rows(player, Five).len() >= 1 || board.rows(opponent, Five).len() >= 1 {
        return None;
    }

    // Already exists four
    if board.rows(player, Four).len() >= 1 {
        return Some(vec![]);
    }

    // Already exists overline
    if board.rows(Black, Overline).len() >= 1 {
        return None;
    }

    let state = VCFState::new(board, player);
    let mut searched = HashSet::new();
    let solution = solve(depth, &state, &mut searched);

    if trim {
        solution.map(|solution| trim_solution(&state, &solution))
    } else {
        solution
    }
}

fn solve(depth: u8, state: &VCFState, searched: &mut HashSet<u64>) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // check if already searched (and was dead-end)
    let hash = state.board_hash();
    if searched.contains(&hash) {
        return None;
    }
    searched.insert(hash);

    for attack in state.valid_moves() {
        let mut state = state.play(attack);
        let defences = state.valid_moves();
        if defences.is_empty() {
            return Some(vec![attack]);
        }

        let defence = defences[0];
        state.play_mut(defence);
        if let Some(mut ps) = solve(depth - 1, &state, searched) {
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

    pub fn play_mut(&mut self, next_move: Point) {
        self.game_state.play_mut(next_move);
    }

    pub fn play(&self, next_move: Point) -> Self {
        let mut result = self.clone();
        result.play_mut(next_move);
        result
    }

    pub fn is_valid_move(&self, p: Point) -> bool {
        self.valid_moves().into_iter().find(|&m| m == p).is_some()
    }

    pub fn valid_moves(&self) -> Vec<Point> {
        if self.game_state.next_player() == self.attacker {
            self.attacking_moves()
        } else {
            self.defending_moves()
        }
    }

    fn attacking_moves(&self) -> Vec<Point> {
        let last_four_eyes = self.game_state.row_eyes_along_last_move(Four);
        if last_four_eyes.len() >= 2 {
            return vec![];
        }
        let last_four_eye = last_four_eyes.into_iter().next();
        self.game_state
            .row_eyes(self.attacker, Sword)
            .into_iter()
            .filter(|&p| last_four_eye.map_or(true, |e| e == p) && self.game_state.is_legal_move(p))
            .collect()
    }

    fn defending_moves(&self) -> Vec<Point> {
        let last_four_eyes = self.game_state.row_eyes_along_last_move(Four);
        if last_four_eyes.len() == 0 {
            panic!();
        }
        if last_four_eyes.len() >= 2 {
            return vec![];
        }
        last_four_eyes
            .into_iter()
            .filter(|&p| self.game_state.is_legal_move(p))
            .collect()
    }

    pub fn board_hash(&self) -> u64 {
        self.game_state.board_hash()
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
        let result = solve_vcf(12, &board, Black, false);
        let solution = "
            G6,H7,J12,K13,G9,F8,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .parse::<Points>()?
        .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(11, &board, Black, false);
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
        let result = solve_vcf(5, &board, White, false);
        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14"
            .parse::<Points>()?
            .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(4, &board, White, false);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_do_trim() -> Result<(), String> {
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
        let result = solve_vcf(10, &board, White, false);
        let solution = "E6,H9,G1,G3,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13"
            .parse::<Points>()?
            .into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vcf(10, &board, White, true);
        let solution = "E6,H9,H5,I6,F5,E5,C8,D7,C11,C9,C14,C13,D13"
            .parse::<Points>()?
            .into_vec();
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
        let result = solve_vcf(u8::MAX, &board, Black, false);
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
