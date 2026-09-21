use super::state::VCFState;
use crate::board::*;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::memo::Memo;
use crate::mate::state::State;

/// How many deadends a solver carries into a new search before it starts
/// dropping what older searches left behind. See [`Memo`].
pub const DEFAULT_CARRY_CAPACITY: usize = 1 << 16;

pub struct DFSSolver {
    /// Positions already shown to have no VCF within their limit, keyed by
    /// [`State::zobrist_hash`]. `solve` is only ever called with the
    /// attacker to move, so the attacker in the key pins the turn as well.
    deadends: Memo<()>,
}

impl DFSSolver {
    pub fn init() -> Self {
        Self::with_carry_capacity(DEFAULT_CARRY_CAPACITY)
    }

    pub fn with_carry_capacity(carry_capacity: usize) -> Self {
        Self {
            deadends: Memo::new(carry_capacity),
        }
    }

    /// Forgets the deadends memoized so far.
    pub fn clear(&mut self) {
        self.deadends.clear();
    }

    /// See [`Memo::advance_generation`]. `solve` recurses into itself, so it
    /// cannot do this on its own: a caller driving the solver over a series
    /// of positions calls it once per question.
    pub fn advance_generation(&mut self) {
        self.deadends.advance_generation();
    }

    pub fn deadends_len(&self) -> usize {
        self.deadends.len()
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
        if self.deadends.contains(hash) {
            return None;
        }
        let result = self.solve_move_pairs(state, budget);
        // A search that gave up proves nothing, so it must not be memoized.
        if result.is_none() && !budget.is_exhausted() {
            self.deadends.insert(hash, ());
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
