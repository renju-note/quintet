use super::generator::Candidate;
use super::generator::Candidates::*;
use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::state::State;
use crate::mate::vct::proof::*;

/// AND/OR search over proof numbers.
///
/// `search_attacks` / `search_defences` evaluate one node (OR / AND
/// respectively); `expand_attacks` / `expand_defences` are the loop that keeps
/// descending into the most-proving child until the node's numbers reach the
/// threshold handed down by the parent; `select_attack` / `select_defence`
/// pick that child, expanding nothing.
///
/// An expansion reads its children's numbers from the tables once and
/// then keeps them itself, writing back only the child it has just
/// searched, rather than looking every child up again at each step.
/// Nothing else can change them in the meantime: every position under
/// that child holds its stone, which none of its siblings does (the search
/// never passes).
impl<P: ThresholdPolicy> VCTSolver<P> {
    /// Runs the search and reports whether the attacker wins. `false` also
    /// covers "gave up": the caller tells the two apart by
    /// `budget.is_exhausted()`.
    ///
    /// A proof is never spurious, so a `true` that was reached just before the
    /// budget ran out still holds.
    pub fn search(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> bool {
        if state.limit() == 0 {
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
                Defeated(_) => Node::disproven(state.limit()),
                Forced(next_move) => {
                    let attacks = &[Candidate::new(next_move, 1)];
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
                Defeated(_) => Node::proven(state.limit()),
                Forced(next_move) => {
                    if state.limit() <= 1 {
                        Node::disproven(state.limit())
                    } else {
                        let defences = &[Candidate::new(next_move, 1)];
                        self.expand_defences(state, defences, threshold, budget)
                            .node
                    }
                }
            };
        }

        if state.limit() <= 1 {
            return Node::disproven(state.limit());
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
        attacks: &[Candidate],
        threshold: Node,
        budget: &mut NodeBudget,
    ) -> Selection {
        let mut children = self.attacker_table.lookup_children(state, attacks);
        loop {
            let selection = Self::select_attack(state.limit(), attacks, &children);
            if Self::exceeds_threshold(selection.node, threshold) {
                return selection;
            }
            if budget.is_exhausted() {
                return selection;
            }
            let best = attacks[selection.best].point;
            // Few attacks are forbidden, so each is only asked about when
            // it is first searched rather than when it is generated
            // (`compute_attacks`). A forbidden one is disproven: the
            // attacker cannot play it.
            if selection.fresh && state.is_forbidden_move(best) {
                let result = state.into_play(Some(best), |child| {
                    let result = Node::disproven(child.limit());
                    self.attacker_table.insert(child, result);
                    result
                });
                children[selection.best] = Some(result);
                continue;
            }
            let next_threshold = P::next_threshold_attack(&selection, threshold);
            let result = state.into_play(Some(best), |child| {
                let result = self.search_defences(child, next_threshold, budget);
                // A child the search gave up on proves nothing about the
                // position, so it must not enter the table.
                if !budget.is_exhausted() {
                    self.attacker_table.insert(child, result);
                }
                result
            });
            // The loop ends at the next check if the budget ran out, so the
            // result is only read back when it is also in the table.
            children[selection.best] = Some(result);
        }
    }

    fn expand_defences(
        &mut self,
        state: &mut VCTState,
        defences: &[Candidate],
        threshold: Node,
        budget: &mut NodeBudget,
    ) -> Selection {
        let mut children = self.defender_table.lookup_children(state, defences);
        loop {
            let selection = Self::select_defence(state.limit(), defences, &children);
            if Self::exceeds_threshold(selection.node, threshold) {
                return selection;
            }
            if budget.is_exhausted() {
                return selection;
            }
            let best = defences[selection.best].point;
            let next_threshold = P::next_threshold_defence(&selection, threshold);
            let result = state.into_play(Some(best), |child| {
                let result = self.search_attacks(child, next_threshold, budget);
                // A child the search gave up on proves nothing about the
                // position, so it must not enter the table.
                if !budget.is_exhausted() {
                    self.defender_table.insert(child, result);
                }
                result
            });
            children[selection.best] = Some(result);
        }
    }

    /// Chooses among `attacks` by their numbers in `children`, as the
    /// attacker table had them (`None` for a child not in it).
    fn select_attack(limit: u8, attacks: &[Candidate], children: &[Option<Node>]) -> Selection {
        let mut best = 0;
        let mut fresh = false;
        let mut node = Node::disproven(limit);
        let mut best_child = Node::disproven(limit);
        let mut second_child = Node::disproven(limit);
        for (i, (attack, &maybe_child)) in attacks.iter().zip(children).enumerate() {
            let child = maybe_child.unwrap_or(Node::unexpanded_defence(attack.estimate, limit));
            node = node.min_pn_sum_dn(child);
            if child.pn < best_child.pn {
                best = i;
                fresh = maybe_child.is_none();
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
            fresh,
            node,
            best_child,
            second_child,
        }
    }

    /// [`Self::select_attack`] for the defender, with `children` as the
    /// defender table had them.
    fn select_defence(limit: u8, defences: &[Candidate], children: &[Option<Node>]) -> Selection {
        let mut best = 0;
        let mut fresh = false;
        let mut node = Node::proven(limit - 1);
        let mut best_child = Node::proven(limit - 1);
        let mut second_child = Node::proven(limit - 1);
        for (i, (defence, &maybe_child)) in defences.iter().zip(children).enumerate() {
            let child = maybe_child.unwrap_or(Node::unexpanded_attack(defence.estimate, limit - 1));
            node = node.min_dn_sum_pn(child);
            if child.dn < best_child.dn {
                best = i;
                fresh = maybe_child.is_none();
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
            fresh,
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
    /// The most-proving child (the move to search next), as an index into
    /// the candidates.
    pub best: usize,
    /// Whether the tables knew nothing of `best`: it has not been
    /// searched.
    pub fresh: bool,
    /// The parent's own numbers, aggregated from the children.
    pub node: Node,
    /// The numbers of `best`.
    pub best_child: Node,
    /// The numbers of the second-most-proving child.
    pub second_child: Node,
}
