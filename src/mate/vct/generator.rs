use super::solver::VCTSolver;
use super::threshold::ThresholdPolicy;
use crate::board::Point;
use crate::mate::budget::NodeBudget;
use crate::mate::state::State;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::VCTState;

use Candidates::*;

/// Candidate move generation, cached by position.
impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn generate_attacks(
        &mut self,
        state: &mut VCTState,
        budget: &mut NodeBudget,
    ) -> Candidates {
        let key = state.zobrist_hash();
        if let Some(hit) = self.attacks_cache.get(&key) {
            hit.clone()
        } else {
            let result = self.compute_attacks(state, budget);
            // A nested search that gave up may have missed a VCF, so what it
            // produced must not be cached.
            if !budget.is_exhausted() {
                self.attacks_cache.put(key, result.clone());
            }
            result
        }
    }

    pub fn generate_defences(
        &mut self,
        state: &mut VCTState,
        budget: &mut NodeBudget,
    ) -> Candidates {
        let key = state.zobrist_hash();
        if let Some(hit) = self.defences_cache.get(&key) {
            hit.clone()
        } else {
            let result = self.compute_defences(state, budget);
            // A nested search that gave up may have missed a threat, so what
            // it produced must not be cached.
            if !budget.is_exhausted() {
                self.defences_cache.put(key, result.clone());
            }
            result
        }
    }

    fn compute_attacks(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Candidates {
        // This is not necessary but improves speed
        let zone = match self.attacker_vcf.vcf_or_zone(state, budget) {
            Ok(_) => return Terminal(Node::proven(state.limit)),
            Err(zone) => zone,
        };

        // This is not necessary but narrows candidates
        let maybe_threat = self.defender_vcf.threat(state, budget);
        let maybe_threat_defences = maybe_threat.map(|t| state.threat_defences(&t));
        let mut result = state.sorted_attacks(maybe_threat_defences);
        let width = result.len() as u32;

        // An attack must be a threat, and a move the zone rules out is not
        // one: this is not necessary either, but saves a nested VCF search
        // for each. A zone the budget cut short proves nothing, but then
        // nothing generated here is kept anyway.
        if !budget.is_exhausted() {
            result.retain(|&p| state.may_threaten(p, &zone));
        }
        result.retain(|&p| !state.is_forbidden_move(p));

        if result.is_empty() {
            return Terminal(Node::disproven(state.limit));
        }

        Moves {
            moves: result,
            width,
        }
    }

    fn compute_defences(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Candidates {
        let maybe_threat = self.attacker_vcf.threat(state, budget);
        if maybe_threat.is_none() {
            return Terminal(Node::disproven(state.limit));
        }

        // This is not necessary but improves speed
        if self.defender_vcf.vcf(state, budget).is_some() {
            return Terminal(Node::disproven(state.limit));
        }

        let threat = maybe_threat.unwrap();
        let threat_defences = state.threat_defences(&threat);
        let mut result = state.sorted_defences(threat_defences);
        result.retain(|&p| !state.is_forbidden_move(p));

        if result.is_empty() {
            return Terminal(Node::proven(state.limit));
        }

        let width = result.len() as u32;
        Moves {
            moves: result,
            width,
        }
    }
}

/// What move generation found for a node.
#[derive(Clone)]
pub enum Candidates {
    /// An inner node.
    Moves {
        /// The moves to expand, best first.
        moves: Vec<Point>,
        /// How many moves there were before those that cannot be threats
        /// were ruled out (see `compute_attacks`), which is what an
        /// unexpanded child's proof number is guessed from. Leaving out
        /// moves that would only be disproven at once does not make the
        /// others any easier to prove.
        width: u32,
    },
    /// The node is decided without expansion (e.g. the attacker has a VCF, or
    /// there is no move at all); this is its value.
    Terminal(Node),
}
