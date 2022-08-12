use crate::board::Point;
use crate::mate::game::*;
use crate::mate::state::MateState;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::State;
use crate::mate::vct::traverser::*;
use std::collections::HashMap;

pub trait LazyGenerator: Traverser {
    fn defences_memory(&mut self) -> &mut HashMap<u64, Vec<Point>>;

    fn generate_attacks(
        &mut self,
        state: &mut State,
        _threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
        let mut result = state.sorted_potentials(3, None);
        result.retain(|&(p, _)| !state.is_forbidden_move(p));

        let len = result.len() as u32;
        let limit = state.limit;
        let result = result
            .into_iter()
            .map(|(p, _)| (p, Node::unit_pn(len, limit)))
            .collect();
        Ok(result)
    }

    fn generate_defences(
        &mut self,
        state: &mut State,
        threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
        let result = self.loop_defence_pass(state, threshold);
        if result.pn != 0 {
            return Err(result);
        }

        let mut defences = self.lookup_defences(state.next_zobrist_hash(None));
        defences.extend(state.four_moves());
        let mut result = state.sort_by_potential(defences);
        result.retain(|&(p, _)| !state.is_forbidden_move(p));

        let len = result.len() as u32;
        let limit = state.limit - 1;
        let result = result
            .into_iter()
            .map(|(p, _)| (p, Node::unit_pn(len, limit)))
            .collect();
        Ok(result)
    }

    fn loop_defence_pass(&mut self, state: &mut State, threshold: Node) -> Node {
        loop {
            let current = self
                .defender_table()
                .lookup_next(state, None)
                .unwrap_or(Node::unit_pn(1, state.limit - 1));
            let selection = Selection {
                best: None,
                current: current,
                next1: current,
                next2: Node::zero_pn(state.limit - 1),
            };
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.next_threshold_defence(&selection, threshold);
            state.into_play(selection.best, |child| {
                let result = self.search_limit_passed(child, next_threshold);
                self.defender_table().insert(child, result);
            })
        }
    }

    fn search_limit_passed(&mut self, state: &mut State, threshold: Node) -> Node {
        if state.limit == 0 {
            return Node::zero_dn(state.limit);
        }
        self.search_attacks_passed(state, threshold)
    }

    fn search_attacks_passed(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(_) => Node::zero_dn(state.limit),
                Forced(m) => {
                    if state.is_four_move(m) {
                        self.traverse_attacks_passed(state, &[m], threshold)
                    } else {
                        Node::zero_dn(state.limit)
                    }
                }
            };
        }

        let mut attacks = state.four_moves();
        attacks.retain(|&p| !state.is_forbidden_move(p));

        if attacks.len() == 0 {
            return Node::zero_dn(state.limit);
        }

        self.traverse_attacks_passed(state, &attacks, threshold)
    }

    fn traverse_attacks_passed(
        &mut self,
        state: &mut State,
        attacks: &[Point],
        threshold: Node,
    ) -> Node {
        let attacks: Vec<_> = attacks
            .into_iter()
            .map(|&p| (p, Node::unit_dn(1, state.limit)))
            .collect();
        let selection =
            self.traverse_attacks(state, &attacks, threshold, Self::search_defences_passed);
        if selection.current.pn == 0 {
            let key = state.next_zobrist_hash(selection.best);
            let mut defences = self.lookup_defences(key);
            defences.extend(selection.best);
            self.extend_defences(state.zobrist_hash(), &defences);
        };
        selection.current
    }

    fn search_defences_passed(&mut self, state: &mut State, threshold: Node) -> Node {
        match state.check_event().unwrap() {
            Defeated(e) => {
                let key = state.zobrist_hash();
                let defences = state.end_breakers(e);
                self.extend_defences(key, &defences);
                Node::zero_pn(state.limit)
            }
            Forced(m) => self.traverse_defences_passed(state, &[m], threshold),
        }
    }

    fn traverse_defences_passed(
        &mut self,
        state: &mut State,
        defences: &[Point],
        threshold: Node,
    ) -> Node {
        let defences: Vec<_> = defences
            .into_iter()
            .map(|&p| (p, Node::unit_pn(1, state.limit)))
            .collect();
        let selection =
            self.traverse_defences(state, &defences, threshold, Self::search_limit_passed);
        if selection.current.pn == 0 && state.game().passed {
            let key = state.next_zobrist_hash(selection.best);
            let mut defences = self.lookup_defences(key);
            defences.extend(selection.best);
            let counter_defences = state.next_sword_eyes(selection.best.unwrap());
            defences.extend(counter_defences);
            self.extend_defences(state.zobrist_hash(), &defences);
        };
        selection.current
    }

    fn extend_defences(&mut self, key: u64, points: &[Point]) {
        let current = self.defences_memory().entry(key).or_insert(vec![]);
        current.extend(points);
    }

    fn lookup_defences(&mut self, key: u64) -> Vec<Point> {
        self.defences_memory()
            .get(&key)
            .iter()
            .flat_map(|&r| r.to_vec())
            .collect()
    }
}
