use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::state::State;
use crate::mate::vct::proof::*;

/// Recovering the winning line after `search` has proven the root.
///
/// The search only records proof numbers in the tables; this walks them
/// again, following proven children, to build the `Mate` path.
impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn extract(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        self.extract_attacks(state, budget)
    }

    fn extract_attacks(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Forced(attack) => state.into_play(Some(attack), |s| {
                    self.extract_defences(s, budget).map(|m| m.unshift(attack))
                }),
                _ => unreachable!(),
            };
        }

        for attack in state.empties() {
            let maybe_node = self.attacker_table.lookup_next(state, Some(attack));
            let node = maybe_node.unwrap_or(Node::unknown());
            if node.is_proven() {
                return state.into_play(Some(attack), |s| {
                    self.extract_defences(s, budget).map(|m| m.unshift(attack))
                });
            }
        }

        self.attacker_vcf.vcf(state, budget)
    }

    fn extract_defences(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(end) => return Some(Mate::new(end, vec![])),
                Forced(defence) => state.into_play(Some(defence), |s| {
                    self.extract_attacks(s, budget).map(|m| m.unshift(defence))
                }),
            };
        }

        let threat = self.attacker_vcf.threat(state, budget).unwrap();
        let threat_defences = state.threat_defences(&threat);
        let defences = state.sorted_defences(threat_defences);
        let mut min_limit = u8::MAX;
        let mut best = None;
        for defence in defences {
            let maybe_node = self.defender_table.lookup_next(state, Some(defence));
            let node = maybe_node.unwrap_or(Node::unknown());
            if node.is_proven() && node.limit < min_limit {
                min_limit = node.limit;
                best.replace(defence);
            }
        }
        if best.is_none() {
            return Some(Mate::new(End::Unknown, vec![]));
        };
        state.into_play(best, |s| {
            self.extract_attacks(s, budget)
                .map(|m| m.unshift(best.unwrap()))
        })
    }
}
