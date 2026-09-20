use super::solver::VCTSolver;
use super::state::VCTState;
use super::threshold::ThresholdPolicy;
use crate::mate::budget::NodeBudget;
use crate::mate::mate::Mate;
use crate::mate::state::State;

/// The four nested VCF questions asked during VCT search:
/// attacker / defender × "has a VCF now" / "would have a VCF after passing"
/// (a threat).
impl<P: ThresholdPolicy> VCTSolver<P> {
    pub fn solve_attacker_vcf(
        &mut self,
        state: &VCTState,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if !state.attacking() {
            panic!()
        }
        let state = &mut state.vcf_state(self.attacker_vcf_depth);
        self.attacker_vcf_solver.solve(state, budget)
    }

    pub fn solve_attacker_threat(
        &mut self,
        state: &VCTState,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if state.attacking() {
            panic!()
        }
        let state = &mut state.threat_state(self.attacker_vcf_depth);
        self.attacker_vcf_solver.solve(state, budget)
    }

    pub fn solve_defender_vcf(
        &mut self,
        state: &VCTState,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if state.attacking() {
            panic!()
        }
        let state = &mut state.vcf_state(self.defender_vcf_depth);
        self.defender_vcf_solver.solve(state, budget)
    }

    pub fn solve_defender_threat(
        &mut self,
        state: &VCTState,
        budget: &mut NodeBudget,
    ) -> Option<Mate> {
        if !state.attacking() {
            panic!()
        }
        let state = &mut state.threat_state(self.defender_vcf_depth);
        self.defender_vcf_solver.solve(state, budget)
    }
}
