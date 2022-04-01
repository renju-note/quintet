use super::solver::Solver;
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

pub trait Searcher: Solver {
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

    fn expand_defence(&mut self, state: &mut State, defence: Point, threshold: Node) {
        state.into_play(Some(defence), |s| {
            let result = self.search_limit(s, threshold);
            self.defender_table().insert(s, result);
        })
    }

    fn find_attacks(&mut self, state: &mut State, _threshold: Node) -> Result<Vec<Point>, Node> {
        // This is not necessary but improves speed
        if self.solve_attacker_vcf(state).is_some() {
            return Err(Node::zero_pn(state.limit()));
        }

        // This is not necessary but narrows candidates
        let maybe_threat = self.solve_defender_threat(state);
        Ok(state.sorted_attacks(maybe_threat))
    }

    fn find_defences(&mut self, state: &mut State, _threshold: Node) -> Result<Vec<Point>, Node> {
        let maybe_threat = self.solve_attacker_threat(state);
        if maybe_threat.is_none() {
            return Err(Node::zero_dn(state.limit()));
        }

        // This is not necessary but improves speed
        if self.solve_defender_vcf(state).is_some() {
            return Err(Node::zero_dn(state.limit()));
        }

        Ok(state.sorted_defences(maybe_threat.unwrap()))
    }

    // MEMO: select_attack|select_defence does trick;
    // approximate child node's initial pn|dn by inheriting the number of *current* attacks|defences
    // since we don't know the number of *next* defences|attacks

    fn select_attack(&mut self, state: &mut State, attacks: &[Point]) -> Selection {
        let limit = state.limit();
        let mut best: Option<Point> = Some(attacks[0]);
        let mut current = Node::zero_dn(limit);
        let mut next1 = Node::zero_dn(limit);
        let mut next2 = Node::zero_dn(limit);
        for &attack in attacks {
            let child = self
                .attacker_table()
                .lookup_next(state, Some(attack))
                .unwrap_or(Node::init_dn(attacks.len() as u32, limit)); // trick
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

    fn select_defence(&mut self, state: &mut State, defences: &[Point]) -> Selection {
        let limit = state.limit();
        let mut best: Option<Point> = Some(defences[0]);
        let mut current = Node::zero_pn(limit - 1);
        let mut next1 = Node::zero_pn(limit - 1);
        let mut next2 = Node::zero_pn(limit - 1);
        for &defence in defences {
            let child = self
                .defender_table()
                .lookup_next(state, Some(defence))
                .unwrap_or(Node::init_pn(defences.len() as u32, limit - 1)); // trick
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

    // pn-search

    fn calc_next_threshold_attack(&self, selection: &Selection, _threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        )
    }

    fn calc_next_threshold_defence(&self, selection: &Selection, _threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        )
    }

    fn backoff(&self, current: Node, threshold: Node) -> bool {
        current.pn >= threshold.pn || current.dn >= threshold.dn
    }
}
