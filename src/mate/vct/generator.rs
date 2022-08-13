use crate::board::Point;
use crate::mate::state::State;
use crate::mate::vct::helper::VCFHelper;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::VCTState;
use lru::LruCache;

pub trait Generator: VCFHelper {
    fn attacks_cache(&mut self) -> &mut LruCache<u64, Result<Vec<Point>, Node>>;
    fn defences_cache(&mut self) -> &mut LruCache<u64, Result<Vec<Point>, Node>>;

    fn generate_attacks(&mut self, state: &mut VCTState) -> Result<Vec<Point>, Node> {
        let key = state.zobrist_hash();
        if let Some(hit) = self.attacks_cache().get(&key) {
            hit.clone()
        } else {
            let result = self.compute_attacks(state);
            self.attacks_cache().put(key, result.clone());
            result
        }
    }

    fn compute_attacks(&mut self, state: &mut VCTState) -> Result<Vec<Point>, Node> {
        // This is not necessary but improves speed
        if self.solve_attacker_vcf(state).is_some() {
            return Err(Node::zero_pn(state.limit));
        }

        // This is not necessary but narrows candidates
        let maybe_threat = self.solve_defender_threat(state);
        let maybe_threat_defences = maybe_threat.map(|t| state.threat_defences(&t));
        let mut result = state.sorted_potentials(3, maybe_threat_defences);
        result.retain(|&x| !state.is_forbidden_move(x.0));

        if result.is_empty() {
            return Err(Node::zero_dn(state.limit));
        }

        let result = result.into_iter().map(|x| x.0).collect();
        Ok(result)
    }

    fn generate_defences(&mut self, state: &mut VCTState) -> Result<Vec<Point>, Node> {
        let key = state.zobrist_hash();
        if let Some(hit) = self.defences_cache().get(&key) {
            hit.clone()
        } else {
            let result = self.compute_defences(state);
            self.defences_cache().put(key, result.clone());
            result
        }
    }

    fn compute_defences(&mut self, state: &mut VCTState) -> Result<Vec<Point>, Node> {
        let maybe_threat = self.solve_attacker_threat(state);
        if maybe_threat.is_none() {
            return Err(Node::zero_dn(state.limit));
        }

        // This is not necessary but improves speed
        if self.solve_defender_vcf(state).is_some() {
            return Err(Node::zero_dn(state.limit));
        }

        let threat = maybe_threat.unwrap();
        let mut result = state.sort_by_potential(state.threat_defences(&threat));
        result.retain(|&x| !state.is_forbidden_move(x.0));

        if result.is_empty() {
            return Err(Node::zero_pn(state.limit));
        }

        let result = result.into_iter().map(|x| x.0).collect();
        Ok(result)
    }
}
