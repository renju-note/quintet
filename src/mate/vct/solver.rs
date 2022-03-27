use super::state::State;
use super::table::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub trait Solver {
    fn threat_limit(&self) -> u8;

    fn attacker_table(&mut self) -> &mut Table;
    fn defender_table(&mut self) -> &mut Table;

    fn attacker_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver;
    fn defender_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver;

    fn solve_attacker_vcf(&mut self, state: &State) -> Option<Mate> {
        if !state.game().attacking() {
            panic!()
        }
        let state = &mut state.vcf_state();
        // this limit can be changed dynamically
        state.set_limit(state.limit().min(self.threat_limit()));
        self.attacker_vcf_solver().solve(state)
    }

    fn solve_attacker_threat(&mut self, state: &State) -> Option<Mate> {
        if state.game().attacking() {
            panic!()
        }
        let state = &mut state.threat_state();
        state.set_limit(state.limit().min(self.threat_limit()));
        self.attacker_vcf_solver().solve(state)
    }

    fn solve_defender_vcf(&mut self, state: &State) -> Option<Mate> {
        if state.game().attacking() {
            panic!()
        }
        let state = &mut state.vcf_state();
        // this limit can be changed dynamically
        state.set_limit(2);
        self.defender_vcf_solver().solve(state)
    }

    fn solve_defender_threat(&mut self, state: &State) -> Option<Mate> {
        if !state.game().attacking() {
            panic!()
        }
        let state = &mut state.threat_state();
        // this limit can be changed dynamically
        state.set_limit(2);
        self.defender_vcf_solver().solve(state)
    }
}
