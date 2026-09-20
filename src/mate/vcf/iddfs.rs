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

    /// Forgets the deadends memoized so far.
    pub fn clear(&mut self) {
        self.solver.clear();
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
