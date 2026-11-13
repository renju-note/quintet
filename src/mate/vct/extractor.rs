use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::board::Point;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::state::State;
use crate::mate::vct::proof::PnDn;

/// Recovering the winning line after `search` has proven the root.
///
/// The search only records proof numbers in the tables; this walks them
/// again, following proven children, to build the `Mate` path.
///
/// A proven node's children may be missing from the tables. A proof carried
/// over from an earlier search on the same solver can outlive the entries
/// under it: a memo drops what the searches before the last one wrote
/// ([`Memo`](crate::mate::memo::Memo)), and a search that only read an entry
/// did not write it again. Such a node is searched again, which puts its
/// children back.
impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn extract(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        self.extract_attacks(state, budget)
    }

    fn extract_attacks(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        let attack = match state.check_event() {
            Some(Forced(attack)) => attack,
            Some(Defeated(_)) => unreachable!("a proven attacker node is not lost"),
            None => match self.attack_proof(state, budget)? {
                AttackProof::Attack(attack) => attack,
                AttackProof::Vcf(mate) => return Some(mate),
            },
        };
        state.with_move(Some(attack), |s| {
            self.extract_defences(s, budget).map(|m| m.prepend(attack))
        })
    }

    fn extract_defences(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        let defence = match state.check_event() {
            Some(Defeated(end)) => return Some(Mate::new(end, vec![])),
            Some(Forced(defence)) => defence,
            None => {
                let threat = self.attacker_vcf.threat(state, budget).unwrap();
                let defences = state.sorted_defences(state.threat_defences(&threat));
                let defence = match self.stubbornest_proven_defence(state, &defences) {
                    Some(defence) => Some(defence),
                    None => {
                        // Proven, but no defence is in the table: searched
                        // again (see above).
                        self.search_defences(state, PnDn::no_threshold(), budget);
                        self.stubbornest_proven_defence(state, &defences)
                    }
                };
                let Some(defence) = defence else {
                    return Some(Mate::new(End::Unknown, vec![]));
                };
                defence
            }
        };
        state.with_move(Some(defence), |s| {
            self.extract_attacks(s, budget).map(|m| m.prepend(defence))
        })
    }

    /// What proved `state`, the attacker to move with nothing forced: the
    /// first attack the table has proven, or else the attacker's VCF (the
    /// shortcut in `compute_attacks`). If the table has neither, the node is
    /// searched again (see above). `None` if that did not prove it, which
    /// only a budget that ran out explains.
    pub(super) fn attack_proof(
        &mut self,
        state: &mut VCTState,
        budget: &mut NodeBudget,
    ) -> Option<AttackProof> {
        if let Some(attack) = self.first_proven_attack(state) {
            return Some(AttackProof::Attack(attack));
        }
        if let Some(mate) = self.attacker_vcf.vcf(state, budget) {
            return Some(AttackProof::Vcf(mate));
        }
        if budget.is_exhausted() {
            return None;
        }
        self.search_attacks(state, PnDn::no_threshold(), budget);
        self.first_proven_attack(state).map(AttackProof::Attack)
    }

    fn first_proven_attack(&self, state: &mut VCTState) -> Option<Point> {
        state.empty_points().into_iter().find(|&attack| {
            self.attacker_table
                .lookup_next(state, Some(attack))
                .is_some_and(|node| node.is_proven())
                // Proven by another order of the moves, see
                // `expand_attacks`.
                && !state.is_forbidden_move(attack)
        })
    }

    /// The proven defence with the smallest `limit`, the first of those: the
    /// one that made the attacker use the most moves.
    fn stubbornest_proven_defence(
        &self,
        state: &mut VCTState,
        defences: &[Point],
    ) -> Option<Point> {
        defences
            .iter()
            .filter_map(|&defence| {
                // Proven by another order of the moves, one that played it
                // while it was not forbidden.
                if state.is_forbidden_move(defence) {
                    return None;
                }
                let node = self.defender_table.lookup_next(state, Some(defence))?;
                node.is_proven().then_some((defence, node.limit))
            })
            .min_by_key(|&(_, limit)| limit)
            .map(|(defence, _)| defence)
    }
}

/// What proved a node with the attacker to move: see
/// [`VCTSolver::attack_proof`].
pub(super) enum AttackProof {
    /// An attack whose child is proven in the table.
    Attack(Point),
    /// The attacker's VCF.
    Vcf(Mate),
}
