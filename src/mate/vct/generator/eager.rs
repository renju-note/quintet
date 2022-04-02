use super::base;
use crate::board::*;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct::proof::*;
use crate::mate::vct::state::State;

pub trait Generator: base::Generator {
    fn attacker_vcf_depth(&self) -> u8;
    fn defender_vcf_depth(&self) -> u8;

    fn attacker_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver;
    fn defender_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver;

    fn find_attacks(&mut self, state: &mut State, _threshold: Node) -> Result<Vec<Point>, Node> {
        // This is not necessary but improves speed
        if self.solve_attacker_vcf(state).is_some() {
            return Err(Node::zero_pn(state.limit()));
        }

        // This is not necessary but narrows candidates
        let maybe_threat = self.solve_defender_threat(state);
        Ok(state.sorted_attacks(maybe_threat))
    }

    fn find_defences(&mut self, state: &mut State, _threshold: Node) -> Result<Vec<Point>, Node> {
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

    fn solve_attacker_vcf(&mut self, state: &State) -> Option<Mate> {
        if !state.game().attacking() {
            panic!()
        }
        let state = &mut state.vcf_state();
        state.set_limit(state.limit().min(self.attacker_vcf_depth()));
        self.attacker_vcf_solver().solve(state)
    }

    fn solve_attacker_threat(&mut self, state: &State) -> Option<Mate> {
        if state.game().attacking() {
            panic!()
        }
        let state = &mut state.threat_state();
        state.set_limit(state.limit().min(self.attacker_vcf_depth()));
        self.attacker_vcf_solver().solve(state)
    }

    fn solve_defender_vcf(&mut self, state: &State) -> Option<Mate> {
        if state.game().attacking() {
            panic!()
        }
        let state = &mut state.vcf_state();
        state.set_limit(self.defender_vcf_depth());
        self.defender_vcf_solver().solve(state)
    }

    fn solve_defender_threat(&mut self, state: &State) -> Option<Mate> {
        if !state.game().attacking() {
            panic!()
        }
        let state = &mut state.threat_state();
        state.set_limit(self.defender_vcf_depth());
        self.defender_vcf_solver().solve(state)
    }
}
