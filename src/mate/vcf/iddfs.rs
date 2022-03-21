use super::dfs;
use super::state::State;
use crate::mate::game::Mate;

pub struct Solver {
    solver: dfs::Solver,
    depths: Vec<u8>,
}

impl Solver {
    pub fn init(max_depths: Vec<u8>) -> Self {
        Self {
            solver: dfs::Solver::init(),
            depths: max_depths,
        }
    }

    pub fn solve(&mut self, state: &mut State, max_depth: u8) -> Option<Mate> {
        for &depth in &self.depths {
            if depth >= max_depth {
                break;
            }
            let result = self.solver.solve(state, depth);
            if result.is_some() {
                return result;
            }
        }
        self.solver.solve(state, max_depth)
    }
}
