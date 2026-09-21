use super::dfs::DFSSolver;
use super::state::VCFState;
use crate::mate::budget::NodeBudget;
use crate::mate::mate::*;

pub struct IDDFSSolver {
    solver: DFSSolver,
    limits: Vec<u8>,
}

impl IDDFSSolver {
    pub fn init(limits: Vec<u8>) -> Self {
        Self {
            solver: DFSSolver::init(),
            limits,
        }
    }

    /// Like [`Self::init`], but bounding what the deadend memo carries from
    /// one search into the next (see [`DFSSolver::advance_generation`]).
    pub fn with_carry_capacity(limits: Vec<u8>, carry_capacity: usize) -> Self {
        Self {
            solver: DFSSolver::with_carry_capacity(carry_capacity),
            limits,
        }
    }

    /// Forgets the deadends memoized so far.
    pub fn clear(&mut self) {
        self.solver.clear();
    }

    /// See [`DFSSolver::advance_generation`]. `solve` is called many times
    /// within one VCT search, so this is not done per call: the outermost
    /// caller decides where one question ends and the next begins.
    pub fn advance_generation(&mut self) {
        self.solver.advance_generation();
    }

    pub fn deadends_len(&self) -> usize {
        self.solver.deadends_len()
    }

    /// Searches for a VCF, deepening the limit step by step. `None` means
    /// either "no VCF within `state.limit`" or "gave up"; the two are told
    /// apart by `budget.is_exhausted()`.
    pub fn solve(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        let max_limit = state.limit;
        for &limit in &self.limits {
            if limit >= max_limit {
                break;
            }
            state.limit = limit;
            let result = self.solver.solve(state, budget);
            if result.is_some() || budget.is_exhausted() {
                state.limit = max_limit;
                return result;
            }
        }
        state.limit = max_limit;
        self.solver.solve(state, budget)
    }
}
