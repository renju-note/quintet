use super::dfs::DFSLocalVCTSolver;
use super::state::LocalVCTState;
use crate::mate::mate::*;

pub struct IDDFSLocalVCTSolver {
    solver: DFSLocalVCTSolver,
    limits: Vec<u8>,
}

impl IDDFSLocalVCTSolver {
    pub fn init(limits: Vec<u8>) -> Self {
        Self {
            solver: DFSLocalVCTSolver::init(),
            limits: limits,
        }
    }

    pub fn solve(&mut self, state: &mut LocalVCTState) -> Option<Mate> {
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
