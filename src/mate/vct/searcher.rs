use super::generator::Generator;
use super::state::State;
use super::traverser::Traverser;
use crate::mate::game::*;
use crate::mate::vct::proof::*;

// MEMO: Debug printing example is 6e2bace

pub trait Searcher: ProofTree + Generator + Traverser {
    fn search(&mut self, state: &mut State) -> bool {
        self.search_limit(state, Node::inf()).proven()
    }

    fn search_limit(&mut self, state: &mut State, threshold: Node) -> Node {
        if state.limit() == 0 {
            return Node::zero_dn(state.limit());
        }
        self.search_attacks(state, threshold)
    }

    fn search_attacks(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::zero_dn(state.limit()),
                Forced(next_move) => {
                    let attacks = &[(next_move, Node::new(1, 1, state.limit()))];
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
            return Node::zero_dn(state.limit());
        }

        self.traverse_attacks(state, &attacks, threshold, Self::search_defences)
            .current
    }

    fn search_defences(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::zero_pn(state.limit()),
                Forced(next_move) => {
                    let defences = &[(next_move, Node::new(1, 1, state.limit() - 1))];
                    self.traverse_defences(state, defences, threshold, Self::search_limit)
                        .current
                }
            };
        }

        let either_defences = self.generate_defences(state, threshold);
        if either_defences.is_err() {
            return either_defences.unwrap_err();
        }

        let defences = either_defences.unwrap();
        if defences.is_empty() {
            return Node::zero_pn(state.limit());
        }

        self.traverse_defences(state, &defences, threshold, Self::search_limit)
            .current
    }
}
