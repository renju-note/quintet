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

    fn calc_next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node;

    fn calc_next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node;

    // default implementations

    fn search(&mut self, state: &mut State) -> bool {
        let result = self.search_limit(state, Node::inf());
        result.proven()
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

        if state.solve_attacker_vcf().is_some() {
            return Node::zero_pn(state.limit());
        }

        let maybe_threat = state.solve_defender_threat();

        let attacks = state.sorted_attacks(maybe_threat);
        self.loop_attacks(state, &attacks, threshold)
    }

    fn loop_attacks(&mut self, state: &mut State, attacks: &[Point], threshold: Node) -> Node {
        loop {
            let selection = self.select_attack(state, &attacks);
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.calc_next_threshold_attack(&selection, threshold);
            self.expand_attack(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn select_attack(&mut self, state: &mut State, attacks: &[Point]) -> Selection {
        let limit = state.limit();
        let mut current = Node::zero_dn(limit);
        let mut best: Option<Point> = None;
        let mut next1 = Node::zero_dn(limit);
        let mut next2 = Node::zero_dn(limit);
        // trick
        let init = Node::init_dn(attacks.len() as u32, limit);
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
                current.dn = INF;
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
                Defeated(_) => Node::zero_pn(state.limit()),
                Forced(m) => self.loop_defences(state, &[m], threshold),
            };
        }

        let maybe_threat = state.solve_attacker_threat();
        if maybe_threat.is_none() {
            return Node::zero_dn(state.limit());
        }

        if state.solve_defender_vcf().is_some() {
            return Node::zero_dn(state.limit());
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());
        self.loop_defences(state, &defences, threshold)
    }

    fn loop_defences(&mut self, state: &mut State, defences: &[Point], threshold: Node) -> Node {
        loop {
            let selection = self.select_defence(state, &defences);
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.calc_next_threshold_defence(&selection, threshold);
            self.expand_defence(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn select_defence(&mut self, state: &mut State, defences: &[Point]) -> Selection {
        let limit = state.limit();
        let mut current = Node::zero_pn(limit);
        let mut best: Option<Point> = None;
        let mut next1 = Node::zero_pn(limit - 1);
        let mut next2 = Node::zero_pn(limit - 1);
        // trick
        let init = Node::init_pn(defences.len() as u32, limit - 1);
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
                current.pn = INF;
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

    fn backoff(&self, current: Node, threshold: Node) -> bool {
        current.pn >= threshold.pn || current.dn >= threshold.dn
    }
}
