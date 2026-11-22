/// How much work a search may do, bounded by nodes and/or a caller stop signal.
///
/// A budget is the only way to interrupt a solver. The crate must keep
/// compiling for `wasm32-unknown-unknown`, so there is no clock here: a caller
/// with a time control or cancellation flag passes a stop predicate with
/// [`NodeBudget::with_stop`], which [`NodeBudget::consume`] polls every
/// `STOP_POLL` nodes.
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
/// assert!(solver.solve(state, &mut budget).is_disproven());
/// assert!(!budget.is_exhausted());
/// ```
#[derive(Clone, Copy)]
pub struct NodeBudget<'a> {
    max_nodes: Option<u64>,
    nodes: u64,
    exhausted: bool,
    stop: Option<&'a dyn Fn() -> bool>,
    stopped: bool,
}

/// How often (in nodes) a stop signal is polled. Must be a power of two.
const STOP_POLL: u64 = 256;

impl Default for NodeBudget<'static> {
    fn default() -> Self {
        Self::unlimited()
    }
}

impl std::fmt::Debug for NodeBudget<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeBudget")
            .field("max_nodes", &self.max_nodes)
            .field("nodes", &self.nodes)
            .field("exhausted", &self.exhausted)
            .field("stop", &self.stop.is_some())
            .finish()
    }
}

impl NodeBudget<'static> {
    /// A budget of `max_nodes` nodes.
    pub fn new(max_nodes: u64) -> Self {
        Self {
            max_nodes: Some(max_nodes),
            nodes: 0,
            exhausted: false,
            stop: None,
            stopped: false,
        }
    }

    /// A budget with no node limit. Pair with [`NodeBudget::with_stop`] to
    /// bound a search by time or an external cancellation flag alone.
    pub fn unlimited() -> Self {
        Self {
            max_nodes: None,
            nodes: 0,
            exhausted: false,
            stop: None,
            stopped: false,
        }
    }
}

impl<'a> NodeBudget<'a> {
    /// Adds a caller stop signal, polled on the first node and every 256 nodes
    /// thereafter. Once `stop()` returns `true`, the budget becomes exhausted.
    ///
    /// ```
    /// use quintet::mate::NodeBudget;
    /// use std::time::{Duration, Instant};
    ///
    /// let deadline = Instant::now() + Duration::from_millis(50);
    /// let stop = || Instant::now() >= deadline;
    /// let mut budget = NodeBudget::unlimited().with_stop(&stop);
    /// assert!(budget.consume());
    /// ```
    pub fn with_stop<'b>(self, stop: &'b dyn Fn() -> bool) -> NodeBudget<'b> {
        NodeBudget {
            max_nodes: self.max_nodes,
            nodes: self.nodes,
            exhausted: self.exhausted,
            stop: Some(stop),
            stopped: self.stopped,
        }
    }

    /// Counts one node. Returns `false` once the node limit is reached or the
    /// stop signal triggers, after which the budget stays exhausted until
    /// [`NodeBudget::reset`].
    pub fn consume(&mut self) -> bool {
        if self.exhausted {
            return false;
        }
        self.nodes += 1;
        if self.max_nodes.is_some_and(|max| self.nodes > max) {
            self.exhausted = true;
            return false;
        }
        if (self.nodes - 1) & (STOP_POLL - 1) == 0 && self.stop.is_some_and(|f| f()) {
            self.exhausted = true;
            self.stopped = true;
            return false;
        }
        true
    }

    /// Whether the budget ran out (by node limit or stop signal). A search that
    /// stopped with an exhausted budget did not finish, so "no mate found"
    /// means "unknown", not "no mate".
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

    /// Puts the counter back to zero, keeping the limit and stop signal.
    pub fn reset(&mut self) {
        self.nodes = 0;
        self.exhausted = false;
        self.stopped = false;
    }

    /// A fresh budget for part of this one's work: `max_nodes`, or what is
    /// left of this one if that is less, sharing this budget's stop signal.
    /// Hand what it spent back with [`Self::absorb`].
    pub fn slice(&self, max_nodes: u64) -> Self {
        let left = self.max_nodes.map(|max| max.saturating_sub(self.nodes));
        Self {
            max_nodes: Some(left.map_or(max_nodes, |left| left.min(max_nodes))),
            nodes: 0,
            exhausted: self.exhausted,
            stop: self.stop,
            stopped: self.stopped,
        }
    }

    /// Counts what `slice`, made by [`Self::slice`], spent. A slice that ran
    /// out of nodes is counted at its limit, without the node that exhausted
    /// it, and exhausts this budget too if it had all that was left. A slice
    /// stopped by the stop signal exhausts this budget immediately.
    pub fn absorb(&mut self, slice: &Self) {
        if slice.stopped {
            self.nodes += slice.nodes;
            self.exhausted = true;
            self.stopped = true;
            return;
        }
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
    use std::cell::Cell;

    #[test]
    fn test_unlimited() {
        let mut budget = NodeBudget::unlimited();
        for _ in 0..1000 {
            assert!(budget.consume());
        }
        assert!(!budget.is_exhausted());
        assert_eq!(budget.nodes(), 1000);
        assert_eq!(budget.max_nodes(), None);
        let default = NodeBudget::default();
        assert_eq!(default.max_nodes(), None);
        assert_eq!(default.nodes(), 0);
        assert!(!default.is_exhausted());
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

    #[test]
    fn test_stop_signal_immediate() {
        let stop = || true;
        let mut budget = NodeBudget::unlimited().with_stop(&stop);
        assert!(!budget.consume());
        assert!(budget.is_exhausted());
        assert_eq!(budget.nodes(), 1);
    }

    #[test]
    fn test_stop_signal_polls_and_propagates_through_slice() {
        let polls = Cell::new(0u64);
        let stop = || {
            let n = polls.get() + 1;
            polls.set(n);
            n >= 3
        };
        let mut budget = NodeBudget::unlimited().with_stop(&stop);
        let mut slice = budget.slice(10_000);
        while slice.consume() {}
        assert!(slice.is_exhausted());
        // Polled at nodes 1 (n=1), 257 (n=2), 513 (n=3 -> stop).
        assert_eq!(slice.nodes(), 513);
        assert_eq!(polls.get(), 3);
        // Absorbing a stopped slice exhausts the parent even though the parent
        // had no node limit.
        budget.absorb(&slice);
        assert!(budget.is_exhausted());
        assert_eq!(budget.nodes(), 513);
    }
}
