use super::state::State;
use super::table::*;
use crate::mate::game::*;
use crate::mate::mate::*;

pub trait Resolver {
    fn attacker_table(&self) -> &Table;
    fn defender_table(&self) -> &Table;

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

        if let Some(vcf) = state.solve_attacker_vcf() {
            return Some(vcf);
        }

        let maybe_threat = state.solve_defender_threat();
        let attacks = state.sorted_attacks(maybe_threat);
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

        unreachable!()
    }

    fn resolve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Forced(defence) => state.into_play(Some(defence), |s| {
                    self.resolve_attacks(s).map(|m| m.unshift(defence))
                }),
                _ => unreachable!(),
            };
        }

        let maybe_threat = state.solve_attacker_threat();
        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut min_limit = u8::MAX;
        let mut best = None;
        for defence in defences {
            let node = self
                .defender_table()
                .lookup_next(state, Some(defence))
                .unwrap_or_else(|| unreachable!());
            if node.limit < min_limit {
                min_limit = node.limit;
                best.replace(defence);
            }
        }
        let best = best.unwrap();
        state.into_play(Some(best), |s| {
            self.resolve_attacks(s).map(|m| m.unshift(best))
        })
    }
}
