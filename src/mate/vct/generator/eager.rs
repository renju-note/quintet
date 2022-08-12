use crate::board::Point;
use crate::mate::vct::helper::VCFHelper;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::State;

pub trait EagerGenerator: VCFHelper {
    fn generate_attacks(
        &mut self,
        state: &mut State,
        _threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
        // This is not necessary but improves speed
        if self.solve_attacker_vcf(state).is_some() {
            return Err(Node::zero_pn(state.limit));
        }

        // This is not necessary but narrows candidates
        let maybe_threat = self.solve_defender_threat(state);
        let maybe_threat_defences = maybe_threat.map(|t| state.threat_defences(&t));
        let mut result = state.sorted_potentials(3, maybe_threat_defences);
        result.retain(|&(p, _)| !state.game().is_forbidden_move(p));

        let len = result.len() as u32;
        let limit = state.limit;
        let result = result
            .into_iter()
            .map(|(p, _)| (p, Node::init_dn(len, limit)))
            .collect();
        Ok(result)
    }

    fn generate_defences(
        &mut self,
        state: &mut State,
        _threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
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
        result.retain(|&(p, _)| !state.game().is_forbidden_move(p));

        let len = result.len() as u32;
        let limit = state.limit - 1;
        let result = result
            .into_iter()
            .map(|(p, _)| (p, Node::init_pn(len, limit)))
            .collect();
        Ok(result)
    }
}
