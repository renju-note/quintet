use super::base;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::vct::state::State;
use crate::mate::vct::table::*;
use crate::mate::vct::traverser;
use crate::mate::vct::traverser::base::Selection;
use std::collections::HashMap;

pub trait Generator: base::Generator + traverser::base::Traverser {
    fn attacker_table(&mut self) -> &mut Table;
    fn defender_table(&mut self) -> &mut Table;

    fn defences_memory(&mut self) -> &mut HashMap<u64, Vec<Point>>;

    fn find_attacks(&mut self, state: &mut State, _threshold: Node) -> Result<Vec<Point>, Node> {
        Ok(state.sorted_attacks(None))
    }

    fn find_defences(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        let result = self.loop_defence_pass(state, threshold);
        if result.pn != 0 {
            return Err(result);
        }

        let mut defences = self.lookup_defences(state.next_zobrist_hash(None));
        defences.extend(state.four_moves());
        defences.retain(|&d| !state.game().is_forbidden_move(d));
        Ok(defences)
    }

    fn loop_defence_pass(&mut self, state: &mut State, threshold: Node) -> Node {
        loop {
            let current = self
                .defender_table()
                .lookup_next(state, None)
                .unwrap_or(Node::init_pn(1, state.limit() - 1));
            let selection = Selection {
                best: None,
                current: current,
                next1: current,
                next2: Node::zero_pn(state.limit() - 1),
            };
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.next_threshold_defence(&selection, threshold);
            self.expand_defence_after_pass(state, selection.best, next_threshold);
        }
    }

    fn search_limit_after_pass(&mut self, state: &mut State, threshold: Node) -> Node {
        if state.limit() == 0 {
            return Node::zero_dn(state.limit());
        }
        self.search_attacks_after_pass(state, threshold)
    }

    fn search_attacks_after_pass(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::zero_dn(state.limit()),
                Forced(m) => {
                    if state.is_four_move(m) {
                        self.loop_attacks_after_pass(state, &[m], threshold)
                    } else {
                        Node::zero_dn(state.limit())
                    }
                }
            };
        }

        let attacks = state.sorted_four_moves();

        if attacks.len() == 0 {
            return Node::zero_dn(state.limit());
        }

        self.loop_attacks_after_pass(state, &attacks, threshold)
    }

    fn loop_attacks_after_pass(
        &mut self,
        state: &mut State,
        attacks: &[Point],
        threshold: Node,
    ) -> Node {
        let selection = loop {
            let selection = self.select_attack(state, &attacks);
            if self.backoff(selection.current, threshold) {
                break selection;
            }
            let next_threshold = self.next_threshold_attack(&selection, threshold);
            self.expand_attack_after_pass(state, selection.best.unwrap(), next_threshold);
        };
        if selection.current.pn == 0 {
            let key = state.next_zobrist_hash(selection.best);
            let mut defences = self.lookup_defences(key);
            defences.extend(selection.best);
            self.extend_defences(state.game().zobrist_hash(), &defences);
        };
        selection.current
    }

    fn expand_attack_after_pass(
        &mut self,
        state: &mut State,
        attack: Point,
        threshold: Node,
    ) -> Node {
        state.into_play(Some(attack), |s| {
            let result = self.search_defences_after_pass(s, threshold);
            self.attacker_table().insert(s, result.clone());
            result
        })
    }

    fn search_defences_after_pass(&mut self, state: &mut State, threshold: Node) -> Node {
        match state.game().check_event().unwrap() {
            Defeated(e) => {
                let key = state.game().zobrist_hash();
                let defences = state.end_breakers(e);
                self.extend_defences(key, &defences);
                Node::zero_pn(state.limit())
            }
            Forced(m) => self.loop_defences_after_pass(state, &[m], threshold),
        }
    }

    fn loop_defences_after_pass(
        &mut self,
        state: &mut State,
        defences: &[Point],
        threshold: Node,
    ) -> Node {
        let selection = loop {
            let selection = self.select_defence(state, &defences);
            if self.backoff(selection.current, threshold) {
                break selection;
            }
            let next_threshold = self.next_threshold_defence(&selection, threshold);
            self.expand_defence_after_pass(state, selection.best, next_threshold);
        };
        if selection.current.pn == 0 && state.game().passed {
            let key = state.next_zobrist_hash(selection.best);
            let mut defences = self.lookup_defences(key);
            defences.extend(selection.best);
            let counter_defences = state.next_sword_eyes(selection.best.unwrap());
            defences.extend(counter_defences);
            self.extend_defences(state.game().zobrist_hash(), &defences);
        };
        selection.current
    }

    fn expand_defence_after_pass(
        &mut self,
        state: &mut State,
        defence: Option<Point>,
        threshold: Node,
    ) -> Node {
        state.into_play(defence, |s| {
            let result = self.search_limit_after_pass(s, threshold);
            self.defender_table().insert(s, result.clone());
            result
        })
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
