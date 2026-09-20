use super::generator::Candidates::*;
use super::selector::Selection;
use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::board::Point;
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
    pub fn search(&mut self, state: &mut VCTState) -> bool {
        if state.limit == 0 {
            return false;
        }
        self.search_attacks(state, Node::no_threshold()).is_proven()
    }

    pub fn search_attacks(&mut self, state: &mut VCTState, threshold: Node) -> Node {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => Node::disproven(state.limit),
                Forced(next_move) => {
                    let attacks = &[next_move];
                    self.expand_attacks(state, attacks, threshold).node
                }
            };
        }

        let attacks = match self.generate_attacks(state) {
            Moves(v) => v,
            Terminal(node) => return node,
        };
        self.expand_attacks(state, &attacks, threshold).node
    }

    pub fn search_defences(&mut self, state: &mut VCTState, threshold: Node) -> Node {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => Node::proven(state.limit),
                Forced(next_move) => {
                    if state.limit <= 1 {
                        Node::disproven(state.limit)
                    } else {
                        let defences = &[next_move];
                        self.expand_defences(state, defences, threshold).node
                    }
                }
            };
        }

        if state.limit <= 1 {
            return Node::disproven(state.limit);
        }

        let defences = match self.generate_defences(state) {
            Moves(v) => v,
            Terminal(node) => return node,
        };
        self.expand_defences(state, &defences, threshold).node
    }

    fn expand_attacks(
        &mut self,
        state: &mut VCTState,
        attacks: &[Point],
        threshold: Node,
    ) -> Selection {
        loop {
            let selection = self.select_attack(state, attacks);
            if Self::exceeds_threshold(selection.node, threshold) {
                return selection;
            }
            let next_threshold = P::next_threshold_attack(&selection, threshold);
            state.into_play(selection.best, |child| {
                let result = self.search_defences(child, next_threshold);
                self.attacker_table.insert(child, result);
            });
        }
    }

    fn expand_defences(
        &mut self,
        state: &mut VCTState,
        defences: &[Point],
        threshold: Node,
    ) -> Selection {
        loop {
            let selection = self.select_defence(state, defences);
            if Self::exceeds_threshold(selection.node, threshold) {
                return selection;
            }
            let next_threshold = P::next_threshold_defence(&selection, threshold);
            state.into_play(selection.best, |child| {
                let result = self.search_attacks(child, next_threshold);
                self.defender_table.insert(child, result);
            });
        }
    }

    fn exceeds_threshold(node: Node, threshold: Node) -> bool {
        node.pn >= threshold.pn || node.dn >= threshold.dn
    }
}
