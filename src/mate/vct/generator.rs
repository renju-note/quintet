use super::solver::VCTSolver;
use super::threshold::ThresholdPolicy;
use crate::board::Point;
use crate::feature::shape::Shape;
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
            Ok(_) => return Terminal(PnDn::proven(state.limit())),
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

        if result.is_empty() {
            return Terminal(PnDn::disproven(state.limit()));
        }

        // An attack is expanded in the order of its estimated proof number
        // (`select_attack`), so these decide how far the search follows the
        // forcing moves before it tries the others. They are the trick of
        // seeding with the number of siblings, so that narrow nodes look
        // easier, lowered for a move that narrows the defender's replies: a
        // four to one, a three to a few. The siblings are counted before the
        // moves that cannot be threats were ruled out (`width`): leaving out
        // moves that would only be disproven at once does not make the
        // others any easier to prove.
        let moves = result
            .into_iter()
            .map(|p| {
                let estimate = match state.best_shape(p) {
                    Shape::Five | Shape::Four => width / 4,
                    Shape::Three => width / 2,
                    _ => width,
                };
                Candidate::new(p, estimate)
            })
            .collect();
        Moves(moves)
    }

    fn compute_defences(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Candidates {
        let Some(threat) = self.attacker_vcf.threat(state, budget) else {
            return Terminal(PnDn::disproven(state.limit()));
        };

        // This is not necessary but improves speed
        if self.defender_vcf.vcf(state, budget).is_some() {
            return Terminal(PnDn::disproven(state.limit()));
        }

        let threat_defences = state.threat_defences(&threat);
        let mut result = state.sorted_defences(threat_defences);
        result.retain(|&p| !state.is_forbidden_move(p));

        if result.is_empty() {
            return Terminal(PnDn::proven(state.limit()));
        }

        let width = result.len() as u32;
        Moves(
            result
                .into_iter()
                .map(|p| Candidate::new(p, width))
                .collect(),
        )
    }
}

/// What move generation found for a node.
#[derive(Clone)]
pub enum Candidates {
    /// The moves to expand, best first.
    Moves(Vec<Candidate>),
    /// The node is decided without expansion (e.g. the attacker has a VCF, or
    /// there is no move at all); this is its value.
    Terminal(PnDn),
}

/// A move to expand, with the number its child starts from while the tables
/// know nothing about it: the proof number of an attack, the disproof number
/// of a defence.
#[derive(Clone, Copy)]
pub struct Candidate {
    pub point: Point,
    pub estimate: u32,
}

impl Candidate {
    pub fn new(point: Point, estimate: u32) -> Self {
        Self {
            point,
            estimate: estimate.max(1),
        }
    }
}
