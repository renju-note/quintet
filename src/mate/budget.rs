/// How much work a search may do, counted in nodes.
///
/// A budget is the only way to interrupt a solver. The crate must keep
/// compiling for `wasm32-unknown-unknown`, so there is no clock here: a caller
/// with a time control converts it into a number of nodes itself.
///
/// The same budget can be passed to several `solve` calls in a row to bound
/// their total work; a solver that is handed an exhausted budget gives up
/// immediately.
///
/// ```
/// use quintet::board::{Board, Player};
/// use quintet::mate::{DFSSolver, NodeBudget, Solver, VCFState};
///
/// let board = Board::new();
/// let mut budget = NodeBudget::new(1_000);
/// let mut solver = DFSSolver::new();
/// let state = &mut VCFState::from_board(&board, Player::Black, 5);
/// assert!(solver.solve(state, &mut budget).is_none());
/// assert!(!budget.is_exhausted());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NodeBudget {
    max_nodes: Option<u64>,
    nodes: u64,
    exhausted: bool,
}

impl NodeBudget {
    /// A budget of `max_nodes` nodes.
    pub fn new(max_nodes: u64) -> Self {
        Self {
            max_nodes: Some(max_nodes),
            nodes: 0,
            exhausted: false,
        }
    }

    /// A budget that is never exhausted.
    pub fn unlimited() -> Self {
        Self {
            max_nodes: None,
            nodes: 0,
            exhausted: false,
        }
    }

    /// Counts one node. Returns `false` once the budget is used up, after
    /// which it stays exhausted until [`NodeBudget::reset`].
    pub fn consume(&mut self) -> bool {
        if self.exhausted {
            return false;
        }
        self.nodes += 1;
        if self.max_nodes.is_some_and(|max| self.nodes > max) {
            self.exhausted = true;
            return false;
        }
        true
    }

    /// Whether the budget ran out. A search that stopped with an exhausted
    /// budget did not finish, so "no mate found" means "unknown", not "no
    /// mate".
    pub fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    /// Nodes counted so far (including the one that exhausted the budget).
    pub fn nodes(&self) -> u64 {
        self.nodes
    }

    /// The limit this budget was created with, if any.
    pub fn max_nodes(&self) -> Option<u64> {
        self.max_nodes
    }

    /// Puts the counter back to zero, keeping the limit.
    pub fn reset(&mut self) {
        self.nodes = 0;
        self.exhausted = false;
    }

    /// A fresh budget for part of this one's work: `max_nodes`, or what is
    /// left of this one if that is less. Hand what it spent back with
    /// [`Self::absorb`].
    pub fn slice(&self, max_nodes: u64) -> Self {
        let left = self.max_nodes.map(|max| max.saturating_sub(self.nodes));
        Self::new(left.map_or(max_nodes, |left| left.min(max_nodes)))
    }

    /// Counts what `slice`, made by [`Self::slice`], spent. A slice that ran
    /// out is counted at its limit, without the node that exhausted it, and
    /// exhausts this budget too if it had all that was left.
    pub fn absorb(&mut self, slice: &Self) {
        let spent = match slice.max_nodes {
            Some(max) if slice.exhausted => max,
            _ => slice.nodes,
        };
        self.nodes += spent;
        if slice.exhausted && self.max_nodes.is_some_and(|max| self.nodes >= max) {
            self.exhausted = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlimited() {
        let mut budget = NodeBudget::unlimited();
        for _ in 0..1000 {
            assert!(budget.consume());
        }
        assert!(!budget.is_exhausted());
        assert_eq!(budget.nodes(), 1000);
        assert_eq!(budget.max_nodes(), None);
        assert_eq!(NodeBudget::default(), NodeBudget::unlimited());
    }

    #[test]
    fn test_exhaustion_is_sticky() {
        let mut budget = NodeBudget::new(2);
        assert!(budget.consume());
        assert!(budget.consume());
        assert!(!budget.consume());
        assert!(budget.is_exhausted());
        assert!(!budget.consume());
        assert_eq!(budget.nodes(), 3);
    }

    #[test]
    fn test_slices_add_up() {
        let mut budget = NodeBudget::new(5);
        // A slice that finishes is counted node for node.
        let mut slice = budget.slice(3);
        assert!(slice.consume());
        budget.absorb(&slice);
        assert_eq!(budget.nodes(), 1);
        assert!(!budget.is_exhausted());
        // One that runs out is counted at its limit, and leaves the parent
        // going while the parent has more.
        let mut slice = budget.slice(2);
        while slice.consume() {}
        budget.absorb(&slice);
        assert_eq!(budget.nodes(), 3);
        assert!(!budget.is_exhausted());
        // A slice gets no more than what is left, and running it out
        // exhausts the parent.
        let mut slice = budget.slice(10);
        assert_eq!(slice.max_nodes(), Some(2));
        while slice.consume() {}
        budget.absorb(&slice);
        assert_eq!(budget.nodes(), 5);
        assert!(budget.is_exhausted());
        // An unlimited parent hands out the slice as asked.
        assert_eq!(NodeBudget::unlimited().slice(7).max_nodes(), Some(7));
    }

    #[test]
    fn test_reset() {
        let mut budget = NodeBudget::new(1);
        assert!(budget.consume());
        assert!(!budget.consume());
        budget.reset();
        assert!(!budget.is_exhausted());
        assert_eq!(budget.nodes(), 0);
        assert!(budget.consume());
    }
}
