use super::state::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::*;
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

    pub fn solve(&mut self, state: &mut State) -> Option<Mate> {
        if state.limit() == 0 {
            return None;
        }

        let hash = state.zobrist_hash();
        if self.deadends.contains(&hash) {
            return None;
        }
        let result = self.solve_attacks(state);
        if result.is_none() {
            self.deadends.insert(hash);
        }
        result
    }

    fn solve_attacks(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => None,
                Forced(m) => self.solve_attack(state, m),
            };
        }

        if let Some(vcf) = state.solve_vcf() {
            return Some(vcf);
        }

        let maybe_threat = state.solve_threat();

        let attacks = state.sorted_attacks(maybe_threat);

        for attack in attacks {
            let result = self.solve_attack(state, attack);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn solve_attack(&mut self, state: &mut State, attack: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(attack);
        let result = self.solve_defences(state).map(|m| m.unshift(attack));
        state.undo(last2_move);
        result
    }

    fn solve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(end) => Some(Mate::new(end, vec![])),
                Forced(m) => self.solve_defence(state, m),
            };
        }

        let maybe_threat = state.solve_threat();
        if maybe_threat.is_none() {
            return None;
        }

        if state.solve_vcf().is_some() {
            return None;
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());

        let mut result = Some(Mate::new(Unknown, vec![]));
        for defence in defences {
            let new_result = self.solve_defence(state, defence);
            if new_result.is_none() {
                result = None;
                break;
            }
            let new_mate = new_result.unwrap();
            result = result.map(|mate| Mate::preferred(mate, new_mate));
        }
        result
    }

    fn solve_defence(&mut self, state: &mut State, defence: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(defence);
        let result = self.solve(state).map(|m| m.unshift(defence));
        state.undo(last2_move);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::*;

    #[test]
    fn test_black() -> Result<(), String> {
        // No. 02 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . o x o . . . . .
         . . . . . . x o . x . . . . .
         . . . . . . . x o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let mut solver = Solver::init();

        let state = &mut State::init(board.clone(), Black, 4);
        let result = solver.solve(state);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("F10,G9,I10,G10,H11,H12,G12".to_string());
        assert_eq!(result, expected);

        let state = &mut State::init(board.clone(), Black, 3);
        let result = solver.solve(state);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_white() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . o . . o . . . . .
         . . . . . . o x x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let mut solver = Solver::init();

        let state = &mut State::init(board.clone(), White, 4);
        let result = solver.solve(state);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("I10,I6,I11,I8,J11,J8,G8".to_string());
        assert_eq!(result, expected);

        let state = &mut State::init(board.clone(), White, 3);
        let result = solver.solve(state);
        let result = result.map(|m| Points(m.path).to_string());
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_counter() -> Result<(), String> {
        // No. 63 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . o . . . . .
         . . . . . . . o x . . . . . .
         . . . x x o . x o . . . . . .
         . . . . . o . o o x . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . x . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let mut solver = Solver::init();

        let state = &mut State::init(board.clone(), White, 4);
        let result = solver.solve(state);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("F7,E8,G8,E6,G5,G7,H6".to_string());
        assert_eq!(result, expected);

        let state = &mut State::init(board.clone(), White, 3);
        let result = solver.solve(state);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_forbidden_breaker() -> Result<(), String> {
        // No. 68 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . o x o . . . . .
         . . . . . . o x o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let mut solver = Solver::init();

        let state = &mut State::init(board.clone(), Black, 4);
        let result = solver.solve(state);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("J8,I7,I8,G8,L8,K8,K7".to_string());
        assert_eq!(result, expected);

        let state = &mut State::init(board.clone(), Black, 3);
        let result = solver.solve(state);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_mise_move() -> Result<(), String> {
        // https://twitter.com/nachirenju/status/1487315157382414336
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . x o o . . . . . .
         . . . . . o o o x x . . . . .
         . . . . o x x x x o . . . . .
         . . . x . x o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let mut solver = Solver::init();

        let state = &mut State::init(board.clone(), Black, 7);
        let result = solver.solve(state);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("G12,E10,F12,I12,H14,H13,F14,G13,F13,F11,E14,D15,G14".to_string());
        assert_eq!(result, expected);

        let state = &mut State::init(board.clone(), Black, 6);
        let result = solver.solve(state);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_dual_forbiddens() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . o o . . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . . x x o . . . . .
         . . . . . . o o x . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;
        let mut solver = Solver::init();

        let state = &mut State::init(board.clone(), White, 5);
        let result = solver.solve(state);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("J4,K3,I4,I3,F8,G7,E6,G9,G6".to_string());
        assert_eq!(result, expected);

        let state = &mut State::init(board.clone(), White, 4);
        let result = solver.solve(state);
        assert_eq!(result, None);

        Ok(())
    }
}
