use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::state::State;

/// Recovering the winning line after `search` has proven the root.
///
/// The search only records proof numbers in the tables; this walks them
/// again, following proven children, to build the `Mate` path.
impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn extract(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        self.extract_attacks(state, budget)
    }

    fn extract_attacks(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        let attack = match state.check_event() {
            Some(Forced(attack)) => Some(attack),
            Some(Defeated(_)) => unreachable!("a proven attacker node is not lost"),
            None => state.empties().into_iter().find(|&attack| {
                self.attacker_table
                    .lookup_next(state, Some(attack))
                    .is_some_and(|node| node.is_proven())
            }),
        };
        // No proven child: the node was proven by the attacker's VCF.
        let Some(attack) = attack else {
            return self.attacker_vcf.vcf(state, budget);
        };
        state.into_play(Some(attack), |s| {
            self.extract_defences(s, budget).map(|m| m.unshift(attack))
        })
    }

    fn extract_defences(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        let defence = match state.check_event() {
            Some(Defeated(end)) => return Some(Mate::new(end, vec![])),
            Some(Forced(defence)) => defence,
            None => {
                // The defence that is lost soonest, the first of those.
                let threat = self.attacker_vcf.threat(state, budget).unwrap();
                let threat_defences = state.threat_defences(&threat);
                let best = state
                    .sorted_defences(threat_defences)
                    .into_iter()
                    .filter_map(|defence| {
                        let node = self.defender_table.lookup_next(state, Some(defence))?;
                        node.is_proven().then_some((defence, node.limit))
                    })
                    .min_by_key(|&(_, limit)| limit);
                let Some((defence, _)) = best else {
                    return Some(Mate::new(End::Unknown, vec![]));
                };
                defence
            }
        };
        state.into_play(Some(defence), |s| {
            self.extract_attacks(s, budget).map(|m| m.unshift(defence))
        })
    }
}
