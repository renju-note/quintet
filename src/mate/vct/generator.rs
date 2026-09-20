use super::solver::VCTSolver;
use super::threshold::ThresholdPolicy;
use crate::board::Point;
use crate::mate::state::State;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::VCTState;

/// What move generation found for a node.
#[derive(Clone)]
pub enum Candidates {
    /// The moves to expand, best first.
    Moves(Vec<Point>),
    /// The node is decided without expansion (e.g. the attacker has a VCF, or
    /// there is no move at all); this is its value.
    Terminal(Node),
}

use Candidates::*;

/// Candidate move generation, cached by position.
impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn generate_attacks(&mut self, state: &mut VCTState) -> Candidates {
        let key = state.zobrist_hash();
        if let Some(hit) = self.attacks_cache.get(&key) {
            hit.clone()
        } else {
            let result = self.compute_attacks(state);
            self.attacks_cache.put(key, result.clone());
            result
        }
    }

    fn compute_attacks(&mut self, state: &mut VCTState) -> Candidates {
        // This is not necessary but improves speed
        if self.solve_attacker_vcf(state).is_some() {
            return Terminal(Node::proven(state.limit));
        }

        // This is not necessary but narrows candidates
        let maybe_threat = self.solve_defender_threat(state);
        let maybe_threat_defences = maybe_threat.map(|t| state.threat_defences(&t));
        let mut result = state.sorted_potentials(3, maybe_threat_defences);
        result.retain(|&x| !state.is_forbidden_move(x.0));

        if result.is_empty() {
            return Terminal(Node::disproven(state.limit));
        }

        Moves(result.into_iter().map(|x| x.0).collect())
    }

    pub fn generate_defences(&mut self, state: &mut VCTState) -> Candidates {
        let key = state.zobrist_hash();
        if let Some(hit) = self.defences_cache.get(&key) {
            hit.clone()
        } else {
            let result = self.compute_defences(state);
            self.defences_cache.put(key, result.clone());
            result
        }
    }

    fn compute_defences(&mut self, state: &mut VCTState) -> Candidates {
        let maybe_threat = self.solve_attacker_threat(state);
        if maybe_threat.is_none() {
            return Terminal(Node::disproven(state.limit));
        }

        // This is not necessary but improves speed
        if self.solve_defender_vcf(state).is_some() {
            return Terminal(Node::disproven(state.limit));
        }

        let threat = maybe_threat.unwrap();
        let mut result = state.sort_by_potential(state.threat_defences(&threat));
        result.retain(|&x| !state.is_forbidden_move(x.0));

        if result.is_empty() {
            return Terminal(Node::proven(state.limit));
        }

        Moves(result.into_iter().map(|x| x.0).collect())
    }
}
