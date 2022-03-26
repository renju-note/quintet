use super::state::State;
use super::table::*;
use crate::board::*;
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
                _ => None,
            };
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

        let vcf_state = &mut vcf::State::new(state.game().clone(), state.limit());
        self.solve_vcf(vcf_state)
    }

    fn resolve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(end) => Some(Mate::new(end, vec![])),
                Forced(defence) => state.into_play(defence, |s| {
                    self.resolve_attacks(s).map(|m| m.unshift(defence))
                }),
            };
        }

        let threat_state = &mut vcf::State::new(state.game().pass(), state.limit() - 1);
        let maybe_threat = self.solve_vcf(threat_state);
        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut min_limit = u8::MAX;
        let mut best = Point(0, 0);
        for defence in defences {
            let node = self
                .table()
                .lookup_next(state, defence)
                .unwrap_or(Node::inf());
            if node.proven() && node.limit < min_limit {
                min_limit = node.limit;
                best = defence;
            }
        }
        state.into_play(best, |s| self.resolve_attacks(s).map(|m| m.unshift(best)))
    }
}
