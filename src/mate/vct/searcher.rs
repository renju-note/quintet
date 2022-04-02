use super::generator::base::Generator;
use super::state::State;
use super::traverser::base::Traverser;
use crate::board::Point;
use crate::mate::game::*;
use crate::mate::vct::proof::{Node, Table};

pub trait Searcher: Generator + Traverser {
    fn attacker_table(&mut self) -> &mut Table;

    fn defender_table(&mut self) -> &mut Table;

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
                Forced(m) => self.loop_attacks(state, &[m], threshold),
            };
        }

        let either_attacks = self.find_attacks(state, threshold);
        if either_attacks.is_err() {
            return either_attacks.unwrap_err();
        }

        let attacks = either_attacks.unwrap();
        if attacks.is_empty() {
            return Node::zero_dn(state.limit());
        }

        self.loop_attacks(state, &attacks, threshold)
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) {
        state.into_play(Some(attack), |s| {
            let result = self.search_defences(s, threshold);
            self.attacker_table().insert(s, result);
        })
    }

    fn search_defences(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::zero_pn(state.limit()),
                Forced(m) => self.loop_defences(state, &[m], threshold),
            };
        }

        let either_defences = self.find_defences(state, threshold);
        if either_defences.is_err() {
            return either_defences.unwrap_err();
        }

        let defences = either_defences.unwrap();
        if defences.is_empty() {
            return Node::zero_pn(state.limit());
        }

        self.loop_defences(state, &defences, threshold)
    }

    fn expand_defence(&mut self, state: &mut State, defence: Point, threshold: Node) {
        state.into_play(Some(defence), |s| {
            let result = self.search_limit(s, threshold);
            self.defender_table().insert(s, result);
        })
    }
}
