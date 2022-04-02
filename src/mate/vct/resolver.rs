use super::state::State;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::vct::proof::*;

pub trait Resolver: ProofTree {
    fn solve_attacker_vcf(&mut self, state: &State) -> Option<Mate>;
    fn solve_attacker_threat(&mut self, state: &State) -> Option<Mate>;

    fn resolve(&mut self, state: &mut State) -> Option<Mate> {
        self.resolve_attacks(state)
    }

    fn resolve_attacks(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Forced(attack) => state.into_play(Some(attack), |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                }),
                _ => unreachable!(),
            };
        }

        let attacks: Vec<_> = state.game().board().empties().collect();
        for attack in attacks {
            let node = self
                .attacker_table()
                .lookup_next(state, Some(attack))
                .unwrap_or(Node::inf());
            if node.proven() {
                return state.into_play(Some(attack), |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                });
            }
        }

        self.solve_attacker_vcf(state)
    }

    fn resolve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(end) => return Some(Mate::new(end, vec![])),
                Forced(defence) => state.into_play(Some(defence), |s| {
                    self.resolve_attacks(s).map(|m| m.unshift(defence))
                }),
            };
        }

        let maybe_threat = self.solve_attacker_threat(state);
        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut min_limit = u8::MAX;
        let mut best = None;
        for defence in defences {
            let node = self
                .defender_table()
                .lookup_next(state, Some(defence))
                .unwrap_or_else(|| Node::inf());
            if node.proven() && node.limit < min_limit {
                min_limit = node.limit;
                best.replace(defence);
            }
        }
        if best.is_none() {
            return Some(Mate::new(End::Unknown, vec![]));
        };
        state.into_play(best, |s| {
            self.resolve_attacks(s).map(|m| m.unshift(best.unwrap()))
        })
    }
}
