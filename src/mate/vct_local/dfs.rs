use super::state::LocalVCTState;
use super::state::Moveset;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::state::State;
use std::collections::HashSet;

pub struct DFSLocalVCTSolver {
    deadends: HashSet<u64>,
}

impl DFSLocalVCTSolver {
    pub fn init() -> Self {
        Self {
            deadends: HashSet::new(),
        }
    }

    pub fn solve(&mut self, state: &mut LocalVCTState) -> Option<Mate> {
        if state.limit == 0 {
            return None;
        }

        let hash = state.zobrist_hash();
        if self.deadends.contains(&hash) {
            return None;
        }
        let result = self.solve_movesets(state);
        if result.is_none() {
            self.deadends.insert(hash);
        }
        result
    }

    fn solve_movesets(&mut self, state: &mut LocalVCTState) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => None,
                Forced(p) => state
                    .forced_moveset(p)
                    .and_then(|ms| self.solve_attack(state, &ms)),
            };
        }

        let four_movesets = state.four_movesets();
        for ms in &four_movesets {
            let result = self.solve_attack(state, ms);
            if result.is_some() {
                return result;
            }
        }

        let three_movesets = state.three_movesets();
        for ms in &three_movesets {
            if four_movesets.iter().any(|nms| nms.attack == ms.attack) {
                continue;
            }
            let result = self.solve_attack(state, ms);
            if result.is_some() {
                // TODO: validate by including related opponent's four moves
                // 1. replace defences with current four moves
                // 2. limit += 1
                // 3. replay
                return result;
            }
        }

        None
    }

    fn solve_attack(&mut self, state: &mut LocalVCTState, moveset: &Moveset) -> Option<Mate> {
        let (attack, defences) = (moveset.attack, &moveset.defences);
        if state.is_forbidden_move(attack) {
            return None;
        }

        state.into_play(Some(attack), |s| {
            self.solve_defences(s, defences).map(|m| m.unshift(attack))
        })
    }

    fn solve_defences(&mut self, state: &mut LocalVCTState, defences: &[Point]) -> Option<Mate> {
        let mut result = Some(Mate::new(End::Unknown, vec![]));
        for &defence in defences {
            let maybe_mate = self.solve_defence(state, defence);
            if maybe_mate.is_none() {
                return None;
            }
            let new_mate = maybe_mate.unwrap();
            if result.is_none() {
                result = Some(new_mate);
                continue;
            }
            let old_mate = result.unwrap();
            result = if new_mate.n_moves() > old_mate.n_moves() {
                Some(new_mate)
            } else {
                Some(old_mate)
            };
        }
        result
    }

    fn solve_defence(&mut self, state: &mut LocalVCTState, defence: Point) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            match event {
                Defeated(end) => return Some(Mate::new(end, vec![])),
                _ => (),
            };
        }

        state.into_play(Some(defence), |s| self.solve(s).map(|m| m.unshift(defence)))
    }
}
