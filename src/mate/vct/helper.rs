use super::state::VCTState;
use crate::mate::mate::Mate;
use crate::mate::state::State;
use crate::mate::vcf;

pub trait VCFHelper {
    fn attacker_vcf_depth(&self) -> u8;
    fn defender_vcf_depth(&self) -> u8;

    fn attacker_vcf_solver(&mut self) -> &mut vcf::IDDFSSolver;
    fn defender_vcf_solver(&mut self) -> &mut vcf::IDDFSSolver;

    fn solve_attacker_vcf(&mut self, state: &VCTState) -> Option<Mate> {
        if !state.attacking() {
            panic!()
        }
        let state = &mut state.vcf_state(self.attacker_vcf_depth());
        self.attacker_vcf_solver().solve(state)
    }

    fn solve_attacker_threat(&mut self, state: &VCTState) -> Option<Mate> {
        if state.attacking() {
            panic!()
        }
        let state = &mut state.threat_state(self.attacker_vcf_depth());
        self.attacker_vcf_solver().solve(state)
    }

    fn solve_defender_vcf(&mut self, state: &VCTState) -> Option<Mate> {
        if state.attacking() {
            panic!()
        }
        let state = &mut state.vcf_state(self.defender_vcf_depth());
        self.defender_vcf_solver().solve(state)
    }

    fn solve_defender_threat(&mut self, state: &VCTState) -> Option<Mate> {
        if !state.attacking() {
            panic!()
        }
        let state = &mut state.threat_state(self.defender_vcf_depth());
        self.defender_vcf_solver().solve(state)
    }
}
