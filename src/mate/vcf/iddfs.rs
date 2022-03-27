use super::dfs;
use super::state::State;
use crate::mate::mate::*;

pub struct Solver {
    solver: dfs::Solver,
    limits: Vec<u8>,
}

impl Solver {
    pub fn init(limits: Vec<u8>) -> Self {
        Self {
            solver: dfs::Solver::init(),
            limits: limits,
        }
    }

    pub fn solve(&mut self, state: &mut State) -> Option<Mate> {
        let max_limit = state.limit();
        for &limit in &self.limits {
            if limit >= max_limit {
                break;
            }
            state.set_limit(limit);
            let result = self.solver.solve(state);
            if result.is_some() {
                state.set_limit(max_limit);
                return result;
            }
        }
        state.set_limit(max_limit);
        self.solver.solve(state)
    }
}
