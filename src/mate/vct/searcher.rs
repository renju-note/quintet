use super::generator::Candidates::*;
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
/// threshold handed down by the parent; `select_attack` / `select_defence`
/// pick that child, expanding nothing and only reading the tables.
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

    fn select_attack(&self, state: &mut VCTState, attacks: &[Point]) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(attacks[0]);
        let mut node = Node::disproven(limit);
        let mut best_child = Node::disproven(limit);
        let mut second_child = Node::disproven(limit);
        let init = Node::unexpanded_defence(attacks.len() as u32, limit); // trick
        for &attack in attacks {
            let maybe_child = self.attacker_table.lookup_next(state, Some(attack));
            let child = maybe_child.unwrap_or(init);
            node = node.min_pn_sum_dn(child);
            if child.pn < best_child.pn {
                best.replace(attack);
                second_child = best_child;
                best_child = child;
            } else if child.pn < second_child.pn {
                second_child = child;
            }
            if node.pn == 0 {
                node.dn = INF;
                break;
            }
        }
        Selection {
            best,
            node,
            best_child,
            second_child,
        }
    }

    fn select_defence(&self, state: &mut VCTState, defences: &[Point]) -> Selection {
        let limit = state.limit;
        let mut best: Option<Point> = Some(defences[0]);
        let mut node = Node::proven(limit - 1);
        let mut best_child = Node::proven(limit - 1);
        let mut second_child = Node::proven(limit - 1);
        let init = Node::unexpanded_attack(defences.len() as u32, limit - 1); // trick
        for &defence in defences {
            let maybe_child = self.defender_table.lookup_next(state, Some(defence));
            let child = maybe_child.unwrap_or(init);
            node = node.min_dn_sum_pn(child);
            if child.dn < best_child.dn {
                best.replace(defence);
                second_child = best_child;
                best_child = child;
            } else if child.dn < second_child.dn {
                second_child = child;
            }
            if node.dn == 0 {
                node.pn = INF;
                break;
            }
        }
        Selection {
            best,
            node,
            best_child,
            second_child,
        }
    }

    fn exceeds_threshold(node: Node, threshold: Node) -> bool {
        node.pn >= threshold.pn || node.dn >= threshold.dn
    }
}

/// Result of evaluating a node from its children's table entries.
pub struct Selection {
    /// The most-proving child (the move to search next).
    pub best: Option<Point>,
    /// The parent's own numbers, aggregated from the children.
    pub node: Node,
    /// The numbers of `best`.
    pub best_child: Node,
    /// The numbers of the second-most-proving child.
    pub second_child: Node,
}
