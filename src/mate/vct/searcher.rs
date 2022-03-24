use super::state::State;
use super::table::*;
use crate::board::*;
use crate::mate::game::*;

pub struct Choice {
    pub current: Node,
    pub next_move: Option<Point>,
    pub next_threshold: Node,
}

pub trait Searcher {
    fn table(&mut self) -> &mut Table;

    fn choose_attack(&self, state: &mut State, attacks: &[Point], threshold: Node) -> Choice;

    fn choose_defence(&self, state: &mut State, defences: &[Point], threshold: Node) -> Choice;

    // default implementations

    fn search(&mut self, state: &mut State) -> bool {
        let root = Node::root(state.limit());
        let result = self.search_limit(state, root);
        result.pn == 0
    }

    fn search_limit(&mut self, state: &mut State, threshold: Node) -> Node {
        if state.limit() == 0 {
            return Node::inf_pn(state.limit());
        }
        self.search_attacks(state, threshold)
    }

    fn search_attacks(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::inf_pn(state.limit()),
                Forced(m) => self.expand_attack(state, m, threshold),
            };
        }

        if state.solve_vcf().is_some() {
            return Node::inf_dn(state.limit());
        }

        let maybe_threat = state.solve_threat();

        let attacks = state.sorted_attacks(maybe_threat);

        loop {
            let choice = self.choose_attack(state, &attacks, threshold);
            let current = choice.current;
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            self.expand_attack(state, choice.next_move.unwrap(), choice.next_threshold);
        }
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) -> Node {
        state.into_play(attack, |s| {
            let result = self.search_defences(s, threshold);
            self.table().insert(s, result.clone());
            result
        })
    }

    fn search_defences(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::inf_dn(state.limit()),
                Forced(m) => self.expand_defence(state, m, threshold),
            };
        }

        let maybe_threat = state.solve_threat();
        if maybe_threat.is_none() {
            return Node::inf_pn(state.limit());
        }

        if state.solve_vcf().is_some() {
            return Node::inf_pn(state.limit());
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());

        loop {
            let choice = self.choose_defence(state, &defences, threshold);
            let current = choice.current;
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            self.expand_defence(state, choice.next_move.unwrap(), choice.next_threshold);
        }
    }

    fn expand_defence(&mut self, state: &mut State, defence: Point, threshold: Node) -> Node {
        state.into_play(defence, |s| {
            let result = self.search_limit(s, threshold);
            self.table().insert(s, result.clone());
            result
        })
    }
}
