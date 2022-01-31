use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::game::*;
use std::collections::HashSet;

pub fn solve(state: &mut State, depth: u8, deadends: &mut HashSet<u64>) -> Option<Solution> {
    if depth == 0 {
        return None;
    }

    let board_hash = state.game().board().zobrist_hash();
    if deadends.contains(&board_hash) {
        return None;
    }

    let (may_first_eye, may_another_eye) = state.game().inspect_last_four_eyes();
    if may_another_eye.is_some() {
        deadends.insert(board_hash);
        return None;
    } else if let Some(last_eye) = may_first_eye {
        let may_move_pair = state
            .next_sequences_on(last_eye, Single, 3)
            .flat_map(sword_eyes_pair)
            .filter(|&(e1, _)| e1 == last_eye)
            .next();
        return if let Some((attack, defence)) = may_move_pair {
            solve_one(state, depth - 1, attack, defence, deadends)
        } else {
            deadends.insert(board_hash);
            None
        };
    }

    let neighbor_move_pairs: Vec<_> = state
        .next_sequences_on(state.game().last2_move(), Single, 3)
        .flat_map(sword_eyes_pair)
        .collect();
    for &(attack, defence) in &neighbor_move_pairs {
        let result = solve_one(state, depth - 1, attack, defence, deadends);
        if result.is_some() {
            return result;
        }
    }

    let move_pairs: Vec<_> = state
        .next_sequences(Single, 3)
        .flat_map(sword_eyes_pair)
        .collect();
    for &(attack, defence) in &move_pairs {
        if neighbor_move_pairs.iter().any(|(a, _)| *a == attack) {
            continue;
        }
        let result = solve_one(state, depth - 1, attack, defence, deadends);
        if result.is_some() {
            return result;
        }
    }

    deadends.insert(board_hash);
    None
}

fn solve_one(
    state: &mut State,
    depth: u8,
    attack: Point,
    defence: Point,
    deadends: &mut HashSet<u64>,
) -> Option<Solution> {
    if state.game().is_forbidden_move(attack) {
        return None;
    }

    let last2_move_attack = state.game().last2_move();
    state.play_mut(attack);

    if let Some(win) = state.game().won_by_last() {
        state.undo_mut(last2_move_attack);
        return Some(Solution::new(win, vec![attack]));
    }

    let last2_move_defence = state.game().last2_move();
    state.play_mut(defence);

    if let Some(mut solution) = solve(state, depth, deadends) {
        let mut path = vec![attack, defence];
        path.append(&mut solution.path);
        state.undo_mut(last2_move_defence);
        state.undo_mut(last2_move_attack);
        return Some(Solution::new(solution.win, path));
    }

    state.undo_mut(last2_move_defence);
    state.undo_mut(last2_move_attack);
    None
}

fn sword_eyes_pair((start, sword): (Index, Sequence)) -> [(Point, Point); 2] {
    let mut eyes = start.mapped(sword.eyes()).map(|i| i.to_point());
    let e1 = eyes.next().unwrap();
    let e2 = eyes.next().unwrap();
    [(e1, e2), (e2, e1)]
}

pub struct State {
    game: GameState,
}

impl State {
    pub fn new(game: GameState) -> Self {
        Self { game: game }
    }

    pub fn init(board: Board, turn: Player) -> Self {
        let game = GameState::init(board, turn);
        Self::new(game)
    }

    pub fn game(&self) -> &'_ GameState {
        &self.game
    }

    pub fn play_mut(&mut self, next_move: Point) {
        self.game.play_mut(next_move);
    }

    pub fn undo_mut(&mut self, last2_move: Point) {
        self.game.undo_mut(last2_move);
    }

    pub fn next_sequences(
        &self,
        k: SequenceKind,
        n: u8,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        let r = self.game().turn();
        self.game.board().sequences(r, k, n, r.is_black())
    }

    pub fn next_sequences_on(
        &self,
        p: Point,
        k: SequenceKind,
        n: u8,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        let r = self.game().turn();
        self.game.board().sequences_on(p, r, k, n, r.is_black())
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
        let state = &mut State::init(board, Black);

        let result = solve(state, 12, &mut HashSet::new());
        let result = result.map(|s| Points(s.path).to_string());
        let solution = "
            J12,K13,G9,F8,G6,H7,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .split_whitespace()
        .collect();
        assert_eq!(result, Some(solution));

        let result = solve(state, 11, &mut HashSet::new());
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
        let state = &mut State::init(board, White);

        let result = solve(state, 5, &mut HashSet::new());
        let result = result.map(|s| Points(s.path).to_string());
        let solution = "L13,L11,K12,J11,I12,H12,I13,I14,H14".to_string();
        assert_eq!(result, Some(solution));

        let result = solve(state, 4, &mut HashSet::new());
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_opponent_overline_not_double_four() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . o . x . . . .
         . . . . . . . . . o x . . . .
         . . . . . . x o o o . . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . . . x . . . .
         . . . . . . . . . . o . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let state = &mut State::init(board, Black);
        let result = solve(state, 3, &mut HashSet::new());
        let result = result.map(|s| Points(s.path).to_string());
        let solution = "K8,L8,H11".split_whitespace().collect();
        assert_eq!(result, Some(solution));
        Ok(())
    }

    #[test]
    fn test_long() -> Result<(), String> {
        // "shadows and fog" by Tama Hoshiduki
        let board = "
         . o x . x o . o x x x x o x x
         . . . o . . x o o x . . x o o
         x . o . . . . . . . . . o . o
         o . . . x x . . . . . . . x x
         . . o . . . . . . . . . . o x
         x o x x . . . . . . . . . o o
         x o . o . . x . . . . o . . .
         o x x x . . . o . x . . . . x
         x x . . . . . . . . . . . . x
         x . . . . . x o x . . . . . x
         o . . . o . . . . x . . . . o
         . o . o . . . x o . . . . . .
         . . . . . . x . o o . . . . .
         o . . . . . . . . o . . x o .
         . . . o . . o x . . o . . . o
        "
        .parse::<Board>()?;
        let state = &mut State::init(board, Black);
        let result = solve(state, u8::MAX, &mut HashSet::new());
        let result = result.map(|s| Points(s.path).to_string());
        let solution = "
            F6,G7,C3,B2,E1,D2,C1,F1,A1,B1,A4,A3,C4,E4,C5,C2,C6,C7,D5,B5,
            E6,B3,D6,B6,G8,F7,D7,D3,F5,G5,G4,H3,F8,E7,I8,E8,F2,E3,F3,F4,
            H5,E2,H7,H9,L1,K2,M1,N1,I1,J1,I2,I5,H2,G2,K5,J4,L4,M3,M5,K3,
            L5,N5,L3,L2,L6,L7,M6,K4,J6,I7,K6,N6,M4,J7,M7,M8,N8,O9,N7,N9,
            O2,N3,O3,O4,K7,N4,K9,K8,M9,L8,J9,I9,K10,L11,M10,L10,M12,M11,L13,K14,
            K13,N13,K11,K12,J10,L12,I13,J13,J12,G15,I11,L14,H12,G13,H11,H13,G11,J11,E11,F11,
            I10,I12,G10,H10,E9,F10,F9,C9,D11,E10,B11,A11,B13,B12,F13,G12,D13,E13,D12,D15,
            B14,A15,E14,C12,C14
        "
        .split_whitespace()
        .collect();
        assert_eq!(result, Some(solution));

        Ok(())
    }
}
