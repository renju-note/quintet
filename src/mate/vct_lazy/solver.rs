use crate::board::Point;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct_lazy::generator::*;
use crate::mate::vct_lazy::proof::*;
use crate::mate::vct_lazy::resolver::Resolver;
use crate::mate::vct_lazy::searcher::Searcher;
use crate::mate::vct_lazy::state::LazyVCTState;
use crate::mate::vct_lazy::traverser::*;
use std::collections::HashMap;

pub struct LazyVCTSolver {
    attacker_table: Table,
    defender_table: Table,
    defences_memory: HashMap<u64, Vec<Point>>,
    vcf_solver: vcf::IDDFSSolver,
}

impl LazyVCTSolver {
    pub fn init() -> Self {
        Self {
            attacker_table: Table::new(),
            defender_table: Table::new(),
            defences_memory: HashMap::new(),
            vcf_solver: vcf::IDDFSSolver::init((1..u8::MAX).collect()),
        }
    }

    pub fn solve(&mut self, state: &mut LazyVCTState) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}

impl Searcher for LazyVCTSolver {}

impl Generator for LazyVCTSolver {
    fn defences_memory(&mut self) -> &mut HashMap<u64, Vec<Point>> {
        &mut self.defences_memory
    }
}

impl ProofTree for LazyVCTSolver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Traverser for LazyVCTSolver {}

impl Resolver for LazyVCTSolver {
    fn solve_attacker_vcf(&mut self, state: &LazyVCTState) -> Option<Mate> {
        self.vcf_solver.solve(&mut state.vcf_state(state.limit))
    }

    fn solve_attacker_threat(&mut self, state: &LazyVCTState) -> Option<Mate> {
        self.vcf_solver.solve(&mut state.threat_state(state.limit))
    }
}
