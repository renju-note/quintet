use super::generator::Generator;
use super::state::LazyVCTState;
use super::traverser::Traverser;
use crate::mate::game::*;
use crate::mate::vct_lazy::proof::*;

// MEMO: Debug printing example is 6e2bace

pub trait Searcher: Generator + Traverser {
    fn search(&mut self, state: &mut LazyVCTState) -> bool {
        if state.limit == 0 {
            return false;
        }
        self.search_attacks(state, Node::inf()).proven()
    }

    fn search_attacks(&mut self, state: &mut LazyVCTState, threshold: Node) -> Node {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => Node::zero_dn(state.limit),
                Forced(next_move) => {
                    let attacks = &[(next_move, Node::unit_dn(1, state.limit))];
                    self.traverse_attacks(state, attacks, threshold, Self::search_defences)
                        .current
                }
            };
        }

        let either_attacks = self.generate_attacks(state, threshold);
        let attacks = match either_attacks {
            Ok(v) => v,
            Err(node) => return node,
        };
        if attacks.is_empty() {
            return Node::zero_dn(state.limit);
        }

        self.traverse_attacks(state, &attacks, threshold, Self::search_defences)
            .current
    }

    fn search_defences(&mut self, state: &mut LazyVCTState, threshold: Node) -> Node {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => Node::zero_pn(state.limit),
                Forced(next_move) => {
                    if state.limit <= 1 {
                        Node::zero_dn(state.limit)
                    } else {
                        let defences = &[(next_move, Node::unit_pn(1, state.limit - 1))];
                        self.traverse_defences(state, defences, threshold, Self::search_attacks)
                            .current
                    }
                }
            };
        }

        if state.limit <= 1 {
            return Node::zero_dn(state.limit);
        }

        let either_defences = self.generate_defences(state, threshold);
        let defences = match either_defences {
            Ok(v) => v,
            Err(node) => return node,
        };
        if defences.is_empty() {
            return Node::zero_pn(state.limit);
        }

        self.traverse_defences(state, &defences, threshold, Self::search_attacks)
            .current
    }
}
