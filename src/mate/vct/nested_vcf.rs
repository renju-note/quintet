use super::state::VCTState;
use crate::mate::budget::NodeBudget;
use crate::mate::mate::Mate;
use crate::mate::solver::Solver;
use crate::mate::state::State;
use crate::mate::vcf::IDDFSSolver;

/// One side's nested VCF search: a VCF solver, how deep it may look, and
/// which side it looks for.
///
/// The VCT solver keeps two — `attacker_vcf` and `defender_vcf` — and asks
/// each of them two questions:
///
/// | | side to move | question |
/// | --- | --- | --- |
/// | `attacker_vcf.vcf` | attacker | Can the attacker win by fours right now? |
/// | `attacker_vcf.threat` | defender | If the defender passed, would the attacker have a VCF — was the last attack a threat? |
/// | `defender_vcf.vcf` | defender | Can the defender win by fours right now? |
/// | `defender_vcf.threat` | attacker | If the attacker passed, would the defender have a VCF — what must the attack also parry? |
///
/// The solver is an [`IDDFSSolver`] with `limits = [1]`, and its memo lives
/// as long as the VCT solver does.
pub struct NestedVCF {
    solver: IDDFSSolver,
    /// How many attacker moves the nested VCF may take, at most.
    depth: u8,
    /// Whether this is the VCT attacker's VCF or the defender's.
    for_attacker: bool,
}

impl NestedVCF {
    pub fn new(for_attacker: bool, depth: u8, carry_capacity: usize) -> Self {
        Self {
            solver: IDDFSSolver::with_carry_capacity([1].to_vec(), carry_capacity),
            depth,
            for_attacker,
        }
    }

    /// Does this side, to move now, have a VCF?
    pub fn vcf(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        assert_eq!(state.attacking(), self.for_attacker);
        let state = &mut state.vcf_state(self.depth);
        self.solver.search(state, budget)
    }

    /// Would this side have a VCF if the other side, to move now, passed?
    pub fn threat(&mut self, state: &mut VCTState, budget: &mut NodeBudget) -> Option<Mate> {
        assert_ne!(state.attacking(), self.for_attacker);
        let state = &mut state.threat_state(self.depth);
        self.solver.search(state, budget)
    }

    pub fn clear(&mut self) {
        self.solver.clear();
    }

    pub fn advance_generation(&mut self) {
        self.solver.advance_generation();
    }

    pub fn memo_len(&self) -> usize {
        self.solver.memo_len()
    }
}
