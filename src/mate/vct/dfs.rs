use super::state::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use std::collections::HashSet;

pub struct Solver {
    deadends: HashSet<u64>,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            deadends: HashSet::new(),
        }
    }

    pub fn solve(&mut self, state: &mut State) -> Option<Mate> {
        if state.limit() == 0 {
            return None;
        }

        let hash = state.zobrist_hash();
        if self.deadends.contains(&hash) {
            return None;
        }
        let result = self.solve_attacks(state);
        if result.is_none() {
            self.deadends.insert(hash);
        }
        result
    }

    fn solve_attacks(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => None,
                Forced(attack) => state.into_play(attack, |s| {
                    self.solve_defences(s).map(|m| m.unshift(attack))
                }),
            };
        }

        if let Some(vcf) = state.solve_vcf() {
            return Some(vcf);
        }

        let maybe_threat = state.solve_threat();

        let attacks = state.sorted_attacks(maybe_threat);

        for attack in attacks {
            let result = state.into_play(attack, |s| {
                self.solve_defences(s).map(|m| m.unshift(attack))
            });
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn solve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(end) => Some(Mate::new(end, vec![])),
                Forced(defence) => {
                    state.into_play(defence, |s| self.solve(s).map(|m| m.unshift(defence)))
                }
            };
        }

        let maybe_threat = state.solve_threat();
        if maybe_threat.is_none() {
            return None;
        }

        if state.solve_vcf().is_some() {
            return None;
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());

        let mut result = Some(Mate::new(Unknown, vec![]));
        for defence in defences {
            let new_result =
                state.into_play(defence, |s| self.solve(s).map(|m| m.unshift(defence)));
            if new_result.is_none() {
                result = None;
                break;
            }
            let new_mate = new_result.unwrap();
            result = result.map(|mate| Mate::preferred(mate, new_mate));
        }
        result
    }
}
