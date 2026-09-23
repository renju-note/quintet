use super::dfs::DFSSolver;
use super::state::VCFState;
use crate::mate::budget::NodeBudget;
use crate::mate::mate::*;
use crate::mate::solver::Solver;

/// Iterative-deepening VCF search: a [`DFSSolver`] run at each of `limits`
/// in turn, then at the state's own limit.
///
/// Shallow passes find short VCFs without exploring the whole tree, and
/// what they memoize is keyed by the limit, so the deeper passes lose
/// nothing to them. The VCT solver uses `limits = [1]`: "check for a
/// one-move win first".
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
    /// one search into the next (see [`Solver::advance_generation`]).
    pub fn with_carry_capacity(limits: Vec<u8>, carry_capacity: usize) -> Self {
        Self {
            solver: DFSSolver::with_carry_capacity(carry_capacity),
            limits,
        }
    }

    /// The search itself, deepening the limit step by step. Unlike
    /// [`Solver::solve`] it opens no generation: the VCT solver calls it many
    /// times within one question, and the outermost caller decides where one
    /// question ends and the next begins.
    pub fn search(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        let max_limit = state.limit;
        for &limit in &self.limits {
            if limit >= max_limit {
                break;
            }
            state.limit = limit;
            let result = self.solver.search(state, budget);
            if result.is_some() || budget.is_exhausted() {
                state.limit = max_limit;
                return result;
            }
        }
        state.limit = max_limit;
        self.solver.search(state, budget)
    }
}

impl Solver for IDDFSSolver {
    type State = VCFState;

    fn solve(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        self.advance_generation();
        self.search(state, budget)
    }

    fn clear(&mut self) {
        self.solver.clear();
    }

    fn advance_generation(&mut self) {
        self.solver.advance_generation();
    }

    fn memo_len(&self) -> usize {
        self.solver.memo_len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::Black;
    use crate::board::{Board, Points};

    /// Black wins at once with G8 (an open four, G8-G11), but a plain DFS
    /// comes across a longer VCF first and stops there.
    fn board() -> Board {
        "F10,G9,G10,G11,H5,H8,J10,K5,L5/D7,D9,D10,E11,I8,J6,J11,L4,L7"
            .parse()
            .unwrap()
    }

    fn path(mate: Option<Mate>) -> String {
        Points(mate.unwrap().path).to_string()
    }

    #[test]
    fn test_finds_the_shortest_vcf_first() {
        let budget = &mut NodeBudget::unlimited();

        let state = &mut VCFState::init(&board(), Black, 5);
        assert_eq!(path(DFSSolver::init().solve(state, budget)), "G7,G8,I9");

        let state = &mut VCFState::init(&board(), Black, 5);
        let mut solver = IDDFSSolver::init(vec![1, 2, 3]);
        assert_eq!(path(solver.solve(state, budget)), "G8");
        // The limit it deepened through is put back.
        assert_eq!(state.limit, 5);
    }
}
