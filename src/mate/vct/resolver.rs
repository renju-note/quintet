use super::solver::Solver;
use super::state::State;
use super::table::*;
use crate::mate::game::*;
use crate::mate::mate::*;

pub trait Resolver: Solver {
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

        if let Some(vcf) = self.solve_attacker_vcf(state) {
            return Some(vcf);
        }

        let maybe_threat = self.solve_defender_threat(state);
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
