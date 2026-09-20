use super::generator::Candidates::*;
use super::selector::Selection;
use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::board::Point;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::state::State;
use crate::mate::vct::proof::*;

// MEMO: Debug printing example is 6e2bace

/// AND/OR search over proof numbers.
///
/// `search_attacks` / `search_defences` evaluate one node (OR / AND
/// respectively); `expand_attacks` / `expand_defences` are the loop that keeps
/// descending into the most-proving child until the node's numbers reach the
/// threshold handed down by the parent.
impl<P: ThresholdPolicy> VCTSolver<P> {
    /// Runs the search and reports whether the attacker wins. `false` also
    /// covers "gave up": the caller tells the two apart by
    /// `budget.is_exhausted()`.
    ///
    /// A proof is never spurious, so a `true` that was reached just before the
    /// budget ran out still holds.
    pub fn search(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> bool {
        if state.limit == 0 {
            return false;
        }
        self.search_attacks(state, Node::no_threshold(), budget)
            .is_proven()
    }

    pub fn search_attacks(
        &mut self,
        state: &mut VCTState,
        threshold: Node,
        budget: &mut NodeBudget,
    ) -> Node {
        if !budget.consume() {
            return Node::unknown();
        }

        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => Node::disproven(state.limit),
                Forced(next_move) => {
                    let attacks = &[next_move];
                    self.expand_attacks(state, attacks, threshold, budget).node
                }
            };
        }

        let attacks = match self.generate_attacks(state, budget) {
            Moves(v) => v,
            Terminal(node) => return node,
        };
        self.expand_attacks(state, &attacks, threshold, budget).node
    }

    pub fn search_defences(
        &mut self,
        state: &mut VCTState,
        threshold: Node,
        budget: &mut NodeBudget,
    ) -> Node {
        if !budget.consume() {
            return Node::unknown();
        }

        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => Node::proven(state.limit),
                Forced(next_move) => {
                    if state.limit <= 1 {
                        Node::disproven(state.limit)
                    } else {
                        let defences = &[next_move];
                        self.expand_defences(state, defences, threshold, budget)
                            .node
                    }
                }
            };
        }

        if state.limit <= 1 {
            return Node::disproven(state.limit);
        }

        let defences = match self.generate_defences(state, budget) {
            Moves(v) => v,
            Terminal(node) => return node,
        };
        self.expand_defences(state, &defences, threshold, budget)
            .node
    }

    fn expand_attacks(
        &mut self,
        state: &mut VCTState,
        attacks: &[Point],
        threshold: Node,
        budget: &mut NodeBudget,
    ) -> Selection {
        loop {
            let selection = self.select_attack(state, attacks);
            if Self::exceeds_threshold(selection.node, threshold) {
                return selection;
            }
            if budget.is_exhausted() {
                return selection;
            }
            let next_threshold = P::next_threshold_attack(&selection, threshold);
            state.into_play(selection.best, |child| {
                let result = self.search_defences(child, next_threshold, budget);
                // A child the search gave up on proves nothing about the
                // position, so it must not enter the table.
                if !budget.is_exhausted() {
                    self.attacker_table.insert(child, result);
                }
            });
        }
    }

    fn expand_defences(
        &mut self,
        state: &mut VCTState,
        defences: &[Point],
        threshold: Node,
        budget: &mut NodeBudget,
    ) -> Selection {
        loop {
            let selection = self.select_defence(state, defences);
            if Self::exceeds_threshold(selection.node, threshold) {
                return selection;
            }
            if budget.is_exhausted() {
                return selection;
            }
            let next_threshold = P::next_threshold_defence(&selection, threshold);
            state.into_play(selection.best, |child| {
                let result = self.search_attacks(child, next_threshold, budget);
                // A child the search gave up on proves nothing about the
                // position, so it must not enter the table.
                if !budget.is_exhausted() {
                    self.defender_table.insert(child, result);
                }
            });
        }
    }

    fn exceeds_threshold(node: Node, threshold: Node) -> bool {
        node.pn >= threshold.pn || node.dn >= threshold.dn
    }
}
