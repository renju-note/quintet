use super::state::State;
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
        if state.game().limit == 0 {
            return None;
        }

        let hash = state.game().zobrist_hash();
        if self.deadends.contains(&hash) {
            return None;
        }
        let result = self.solve_move_pairs(state);
        if result.is_none() {
            self.deadends.insert(hash);
        }
        result
    }

    fn solve_move_pairs(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => None,
                Forced(p) => state
                    .forced_move_pair(p)
                    .and_then(|(a, d)| self.solve_attack(state, a, d)),
            };
        }

        let neighbor_pairs = state.neighbor_move_pairs();
        for &(attack, defence) in &neighbor_pairs {
            let result = self.solve_attack(state, attack, defence);
            if result.is_some() {
                return result;
            }
        }

        let pairs = state.move_pairs();
        for &(attack, defence) in &pairs {
            if neighbor_pairs.iter().any(|(a, _)| *a == attack) {
                continue;
            }
            let result = self.solve_attack(state, attack, defence);
            if result.is_some() {
                return result;
            }
        }

        None
    }

    fn solve_attack(&mut self, state: &mut State, attack: Point, defence: Point) -> Option<Mate> {
        if state.game().is_forbidden_move(attack) {
            return None;
        }

        state.into_play(attack, |s| {
            self.solve_defence(s, defence).map(|m| m.unshift(attack))
        })
    }

    fn solve_defence(&mut self, state: &mut State, defence: Point) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            match event {
                Defeated(end) => return Some(Mate::new(end, vec![])),
                _ => (),
            };
        }

        state.into_play(defence, |s| self.solve(s).map(|m| m.unshift(defence)))
    }
}
