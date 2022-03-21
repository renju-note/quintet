use super::state::State;
use crate::board::*;
use crate::mate::game::*;
use std::collections::HashSet;

pub struct Solver {
    deadends: HashSet<u64>,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            deadends: HashSet::new(),
        }
    }

    pub fn solve(&mut self, state: &mut State, max_depth: u8) -> Option<Mate> {
        self.solve_limit(state, max_depth)
    }

    fn solve_limit(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        if limit == 0 {
            return None;
        }

        let hash = state.game().get_hash(limit);
        if self.deadends.contains(&hash) {
            return None;
        }
        let result = self.solve_move_pairs(state, limit);
        if result.is_none() {
            self.deadends.insert(hash);
        }
        result
    }

    fn solve_move_pairs(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        let (maybe_four_eye, maybe_four_eye_another) = state.game().check_last_four_eyes();
        if maybe_four_eye_another.is_some() {
            return None;
        } else if let Some(four_eye) = maybe_four_eye {
            if let Some((attack, defence)) = state.mandatory_move_pair(four_eye) {
                return self.solve_attack(state, limit, attack, defence);
            } else {
                return None;
            }
        }

        let neighbor_pairs = state.neighbor_move_pairs();
        for &(attack, defence) in &neighbor_pairs {
            let result = self.solve_attack(state, limit, attack, defence);
            if result.is_some() {
                return result;
            }
        }

        let pairs = state.move_pairs();
        for &(attack, defence) in &pairs {
            if neighbor_pairs.iter().any(|(a, _)| *a == attack) {
                continue;
            }
            let result = self.solve_attack(state, limit, attack, defence);
            if result.is_some() {
                return result;
            }
        }

        None
    }

    fn solve_attack(
        &mut self,
        state: &mut State,
        limit: u8,
        attack: Point,
        defence: Point,
    ) -> Option<Mate> {
        if state.game().is_forbidden_move(attack) {
            return None;
        }

        let last2_move = state.game().last2_move();
        state.game().play(attack);
        let result = self
            .solve_defence(state, limit, defence)
            .map(|m| m.unshift(attack));
        state.game().undo(last2_move);
        result
    }

    fn solve_defence(&mut self, state: &mut State, limit: u8, defence: Point) -> Option<Mate> {
        if let Some(win) = state.game().check_win() {
            return Some(Mate::new(win, vec![]));
        }

        let last2_move = state.game().last2_move();
        state.game().play(defence);
        let result = self
            .solve_limit(state, limit - 1)
            .map(|m| m.unshift(defence));
        state.game().undo(last2_move);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::*;

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
        let mut solver = Solver::init();

        let result = solver.solve(state, 12);
        let result = result.map(|m| Points(m.path).to_string());
        let mate = "
            J12,K13,G9,F8,G6,H7,G8,G7,G12,G11,F12,I12,D12,E12,F10,E11,E10,D10,F11,D9,
            F14,F13,C11
        "
        .split_whitespace()
        .collect();
        assert_eq!(result, Some(mate));

        let result = solver.solve(state, 11);
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
        let mut solver = Solver::init();

        let result = solver.solve(state, 5);
        let result = result.map(|m| Points(m.path).to_string());
        let mate = "L13,L11,K12,J11,I12,H12,I13,I14,H14".to_string();
        assert_eq!(result, Some(mate));

        let result = solver.solve(state, 4);
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
        let mut solver = Solver::init();

        let result = solver.solve(state, 3);
        let result = result.map(|m| Points(m.path).to_string());
        let mate = "K8,L8,H11".split_whitespace().collect();
        assert_eq!(result, Some(mate));
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
        let mut solver = Solver::init();

        let result = solver.solve(state, u8::MAX);
        let result = result.map(|m| Points(m.path).to_string());
        let mate = "
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
        assert_eq!(result, Some(mate));

        Ok(())
    }
}
