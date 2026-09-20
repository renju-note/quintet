use super::state::VCFState;
use crate::board::*;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::state::State;
use std::collections::HashSet;

pub struct DFSSolver {
    deadends: HashSet<u64>,
}

impl DFSSolver {
    pub fn init() -> Self {
        Self {
            deadends: HashSet::new(),
        }
    }

    /// Forgets the deadends memoized so far.
    pub fn clear(&mut self) {
        self.deadends.clear();
    }

    /// Searches for a VCF. `None` means either "no VCF within `state.limit`"
    /// or "gave up"; the two are told apart by `budget.is_exhausted()`.
    pub fn solve(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        if state.limit == 0 {
            return None;
        }
        if !budget.consume() {
            return None;
        }

        let hash = state.zobrist_hash();
        if self.deadends.contains(&hash) {
            return None;
        }
        let result = self.solve_move_pairs(state, budget);
        // A search that gave up proves nothing, so it must not be memoized.
        if result.is_none() && !budget.is_exhausted() {
            self.deadends.insert(hash);
        }
        result
    }

    fn solve_move_pairs(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => None,
                Forced(p) => state
                    .forced_move_pair(p)
                    .and_then(|(a, d)| self.solve_attack(state, a, d, budget)),
            };
        }

        let neighbor_pairs = state.neighbor_move_pairs();
        for &(attack, defence) in &neighbor_pairs {
            let result = self.solve_attack(state, attack, defence, budget);
            if result.is_some() {
                return result;
            }
            if budget.is_exhausted() {
                return None;
            }
        }

        let pairs = state.move_pairs();
        for &(attack, defence) in &pairs {
            if neighbor_pairs.iter().any(|(a, _)| *a == attack) {
                continue;
            }
            let result = self.solve_attack(state, attack, defence, budget);
            if result.is_some() {
                return result;
            }
            if budget.is_exhausted() {
                return None;
            }
        }

        None
    }

    fn solve_attack(
        &mut self,
        state: &mut VCFState,
        attack: Point,
        defence: Point,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if state.is_forbidden_move(attack) {
            return None;
        }

        state.into_play(Some(attack), |s| {
            self.solve_defence(s, defence, budget)
                .map(|m| m.unshift(attack))
        })
    }

    fn solve_defence(
        &mut self,
        state: &mut VCFState,
        defence: Point,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if let Some(Defeated(end)) = state.check_event() {
            return Some(Mate::new(end, vec![]));
        }

        state.into_play(Some(defence), |s| {
            self.solve(s, budget).map(|m| m.unshift(defence))
        })
    }
}
