use super::dfs::DFSSolver;
use super::state::VCFState;
use crate::mate::mate::*;

pub struct IDDFSSolver {
    solver: DFSSolver,
    limits: Vec<u8>,
}

impl IDDFSSolver {
    pub fn init(limits: Vec<u8>) -> Self {
        Self {
            solver: DFSSolver::init(),
            limits: limits,
        }
    }

    pub fn solve(&mut self, state: &mut VCFState) -> Option<Mate> {
        let max_limit = state.limit;
        for &limit in &self.limits {
            if limit >= max_limit {
                break;
            }
            state.limit = limit;
            let result = self.solver.solve(state);
            if result.is_some() {
                state.limit = max_limit;
                return result;
            }
        }
        state.limit = max_limit;
        self.solver.solve(state)
    }
}
