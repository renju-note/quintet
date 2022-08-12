use crate::board::Point;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct::generator::*;
use crate::mate::vct::proof::*;
use crate::mate::vct::resolver::Resolver;
use crate::mate::vct::searcher::Searcher;
use crate::mate::vct::solver::Solver;
use crate::mate::vct::state::VCTState;
use crate::mate::vct::traverser::*;
use std::collections::HashMap;

pub struct LazyDFPNSSolver {
    attacker_table: Table,
    defender_table: Table,
    defences_memory: HashMap<u64, Vec<Point>>,
    vcf_solver: vcf::IDDFSSolver,
}

impl LazyDFPNSSolver {
    pub fn init() -> Self {
        Self {
            attacker_table: Table::new(),
            defender_table: Table::new(),
            defences_memory: HashMap::new(),
            vcf_solver: vcf::IDDFSSolver::init((1..u8::MAX).collect()),
        }
    }
}

impl Solver for LazyDFPNSSolver {}

impl Searcher for LazyDFPNSSolver {}

impl ProofTree for LazyDFPNSSolver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Generator for LazyDFPNSSolver {
    fn generate_attacks(
        &mut self,
        state: &mut VCTState,
        threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
        LazyGenerator::generate_attacks(self, state, threshold)
    }

    fn generate_defences(
        &mut self,
        state: &mut VCTState,
        threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
        LazyGenerator::generate_defences(self, state, threshold)
    }
}

impl LazyGenerator for LazyDFPNSSolver {
    fn defences_memory(&mut self) -> &mut HashMap<u64, Vec<Point>> {
        &mut self.defences_memory
    }
}

impl Traverser for LazyDFPNSSolver {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        DFPNSTraverser::next_threshold_attack(self, selection, threshold)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        DFPNSTraverser::next_threshold_defence(self, selection, threshold)
    }
}

impl DFPNSTraverser for LazyDFPNSSolver {}

impl Resolver for LazyDFPNSSolver {
    fn solve_attacker_vcf(&mut self, state: &VCTState) -> Option<Mate> {
        self.vcf_solver.solve(&mut state.vcf_state(state.limit))
    }

    fn solve_attacker_threat(&mut self, state: &VCTState) -> Option<Mate> {
        self.vcf_solver.solve(&mut state.threat_state(state.limit))
    }
}
