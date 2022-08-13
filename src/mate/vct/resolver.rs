use super::helper::VCFHelper;
use super::state::VCTState;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::state::State;
use crate::mate::vct::proof::*;

pub trait Resolver: ProofTree + VCFHelper {
    fn resolve(&mut self, state: &mut VCTState) -> Option<Mate> {
        self.resolve_attacks(state)
    }

    fn resolve_attacks(&mut self, state: &mut VCTState) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Forced(attack) => state.into_play(Some(attack), |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                }),
                _ => unreachable!(),
            };
        }

        for attack in state.empties() {
            let maybe_node = self.attacker_table().lookup_next(state, Some(attack));
            let node = maybe_node.unwrap_or(Node::inf());
            if node.proven() {
                return state.into_play(Some(attack), |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                });
            }
        }

        self.solve_attacker_vcf(state)
    }

    fn resolve_defences(&mut self, state: &mut VCTState) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(end) => return Some(Mate::new(end, vec![])),
                Forced(defence) => state.into_play(Some(defence), |s| {
                    self.resolve_attacks(s).map(|m| m.unshift(defence))
                }),
            };
        }

        let threat = self.solve_attacker_threat(state).unwrap();
        let defences = state.sort_by_potential(state.threat_defences(&threat));
        let mut min_limit = u8::MAX;
        let mut best = None;
        for (defence, _) in defences {
            let maybe_node = self.defender_table().lookup_next(state, Some(defence));
            let node = maybe_node.unwrap_or(Node::inf());
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
