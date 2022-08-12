use super::generator::Generator;
use super::state::VCTState;
use super::traverser::Traverser;
use crate::mate::game::*;
use crate::mate::vct::proof::*;

// MEMO: Debug printing example is 6e2bace

pub trait Searcher: Generator + Traverser {
    fn search(&mut self, state: &mut VCTState) -> bool {
        if state.limit == 0 {
            return false;
        }
        self.search_attacks(state, Node::inf()).proven()
    }

    fn search_attacks(&mut self, state: &mut VCTState, threshold: Node) -> Node {
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
        if either_attacks.is_err() {
            return either_attacks.unwrap_err();
        }

        let attacks = either_attacks.unwrap();
        if attacks.is_empty() {
            return Node::zero_dn(state.limit);
        }

        self.traverse_attacks(state, &attacks, threshold, Self::search_defences)
            .current
    }

    fn search_defences(&mut self, state: &mut VCTState, threshold: Node) -> Node {
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
        if either_defences.is_err() {
            return either_defences.unwrap_err();
        }

        let defences = either_defences.unwrap();
        if defences.is_empty() {
            return Node::zero_pn(state.limit);
        }

        self.traverse_defences(state, &defences, threshold, Self::search_attacks)
            .current
    }
}
