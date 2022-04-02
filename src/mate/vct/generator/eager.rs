use crate::board::Point;
use crate::mate::vct::helper::VCFHelper;
use crate::mate::vct::proof::Node;
use crate::mate::vct::state::State;

pub trait EagerGenerator: VCFHelper {
    fn generate_attacks(
        &mut self,
        state: &mut State,
        _threshold: Node,
    ) -> Result<Vec<Point>, Node> {
        // This is not necessary but improves speed
        if self.solve_attacker_vcf(state).is_some() {
            return Err(Node::zero_pn(state.limit()));
        }

        // This is not necessary but narrows candidates
        let maybe_threat = self.solve_defender_threat(state);
        Ok(state.sorted_attacks(maybe_threat))
    }

    fn generate_defences(
        &mut self,
        state: &mut State,
        _threshold: Node,
    ) -> Result<Vec<Point>, Node> {
        let maybe_threat = self.solve_attacker_threat(state);
        if maybe_threat.is_none() {
            return Err(Node::zero_dn(state.limit()));
        }

        // This is not necessary but improves speed
        if self.solve_defender_vcf(state).is_some() {
            return Err(Node::zero_dn(state.limit()));
        }

        Ok(state.sorted_defences(maybe_threat.unwrap()))
    }
}
