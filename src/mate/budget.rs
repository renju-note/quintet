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
/// use quintet::mate::{DFSSolver, NodeBudget, VCFState};
///
/// let board = Board::new();
/// let mut budget = NodeBudget::new(1_000);
/// let mut solver = DFSSolver::init();
/// let state = &mut VCFState::init(&board, Player::Black, 5);
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
    /// which it stays exhausted until [`NodeBudget::restart`].
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
    pub fn restart(&mut self) {
        self.nodes = 0;
        self.exhausted = false;
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
    fn test_restart() {
        let mut budget = NodeBudget::new(1);
        assert!(budget.consume());
        assert!(!budget.consume());
        budget.restart();
        assert!(!budget.is_exhausted());
        assert_eq!(budget.nodes(), 0);
        assert!(budget.consume());
    }

    #[test]
    fn test_default_is_unlimited() {
        assert_eq!(NodeBudget::default(), NodeBudget::unlimited());
    }
}
