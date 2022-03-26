use super::state::State;
use super::table::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub trait Resolver {
    fn table(&self) -> &Table;

    fn solve_vcf(&mut self, state: &mut vcf::State) -> Option<Mate>;

    // default implementations

    fn resolve(&mut self, state: &mut State) -> Option<Mate> {
        self.resolve_attacks(state)
    }

    fn resolve_attacks(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Forced(attack) => state.into_play(attack, |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                }),
                _ => unreachable!(),
            };
        }

        let vcf_state = &mut vcf::State::new(state.game().clone(), state.limit());
        if let Some(vcf) = self.solve_vcf(vcf_state) {
            return Some(vcf);
        }

        let attacks = state.sorted_attacks(None);
        for attack in attacks {
            let node = self
                .table()
                .lookup_next(state, attack)
                .unwrap_or(Node::inf());
            if node.proven() {
                return state.into_play(attack, |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                });
            }
        }

        unreachable!()
    }

    fn resolve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Forced(defence) => state.into_play(defence, |s| {
                    self.resolve_attacks(s).map(|m| m.unshift(defence))
                }),
                _ => unreachable!(),
            };
        }

        let threat_state = &mut vcf::State::new(state.game().pass(), state.limit() - 1);
        let maybe_threat = self.solve_vcf(threat_state);
        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut min_limit = u8::MAX;
        let mut best = None;
        for defence in defences {
            let node = self
                .table()
                .lookup_next(state, defence)
                .unwrap_or_else(|| unreachable!());
            if node.limit < min_limit {
                min_limit = node.limit;
                best.replace(defence);
            }
        }
        let best = best.unwrap();
        state.into_play(best, |s| self.resolve_attacks(s).map(|m| m.unshift(best)))
    }
}
