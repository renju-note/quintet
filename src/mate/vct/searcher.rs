use super::state::State;
use super::table::*;
use crate::board::*;
use crate::mate::game::*;

pub struct Selection {
    pub best: Option<Point>,
    pub current: Node,
    pub next1: Node,
    pub next2: Node,
}

// MEMO: select_attack|select_defence does trick;
// approximate child node's initial pn|dn by inheriting the number of *current* attacks|defences
// since we don't know the number of *next* defences|attacks
pub trait Searcher {
    fn table(&mut self) -> &mut Table;

    fn calc_next_threshold_attack(&self, selection: &Selection, current_threshold: Node) -> Node;

    fn calc_next_threshold_defence(&self, selection: &Selection, current_threshold: Node) -> Node;

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

        if state.solve_attacker_vcf().is_some() {
            return Node::inf_dn(state.limit());
        }

        let maybe_threat = state.solve_defender_threat();

        let attacks = state.sorted_attacks(maybe_threat);

        loop {
            let selection = self.select_attack(state, &attacks);
            let current = selection.current;
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            let next_threshold = self.calc_next_threshold_attack(&selection, threshold);
            self.expand_attack(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn select_attack(&mut self, state: &mut State, attacks: &[Point]) -> Selection {
        let limit = state.limit();
        let mut current = Node::inf_pn(limit);
        let mut best: Option<Point> = None;
        let mut next1 = Node::inf_pn(limit);
        let mut next2 = Node::inf_pn(limit);
        // trick
        let init = Node::init_dn(attacks.len(), limit);
        for &attack in attacks {
            let child = self.table().lookup_next(state, attack).unwrap_or(init);
            current = current.min_pn_sum_dn(child);
            if child.pn < next1.pn {
                best.replace(attack);
                next2 = next1;
                next1 = child;
            } else if child.pn < next2.pn {
                next2 = child;
            }
            if current.pn == 0 {
                current = Node::inf_dn(current.limit);
                break;
            }
        }
        Selection {
            best: best,
            current: current,
            next1: next1,
            next2: next2,
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

        let maybe_threat = state.solve_attacker_threat();
        if maybe_threat.is_none() {
            return Node::inf_pn(state.limit());
        }

        if state.solve_defender_vcf().is_some() {
            return Node::inf_pn(state.limit());
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());

        loop {
            let selection = self.select_defence(state, &defences);
            let current = selection.current;
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            let next_threshold = self.calc_next_threshold_defence(&selection, threshold);
            self.expand_defence(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn select_defence(&mut self, state: &mut State, defences: &[Point]) -> Selection {
        let limit = state.limit();
        let mut current = Node::inf_dn(limit);
        let mut best: Option<Point> = None;
        let mut next1 = Node::inf_dn(limit - 1);
        let mut next2 = Node::inf_dn(limit - 1);
        // trick
        let init = Node::init_pn(defences.len(), limit - 1);
        for &defence in defences {
            let child = self.table().lookup_next(state, defence).unwrap_or(init);
            current = current.min_dn_sum_pn(child);
            if child.dn < next1.dn {
                best.replace(defence);
                next2 = next1;
                next1 = child;
            } else if child.dn < next2.dn {
                next2 = child;
            }
            if current.dn == 0 {
                current = Node::inf_pn(current.limit);
                break;
            }
        }
        Selection {
            best: best,
            current: current,
            next1: next1,
            next2: next2,
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
