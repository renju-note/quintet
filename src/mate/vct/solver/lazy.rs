use crate::board::Point;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct::generator::*;
use crate::mate::vct::proof::*;
use crate::mate::vct::resolver::*;
use crate::mate::vct::searcher::Searcher;
use crate::mate::vct::solver::Solver;
use crate::mate::vct::state::State;
use crate::mate::vct::traverser::*;
use std::collections::HashMap;

pub struct LazyDFPNSolver {
    attacker_table: Table,
    defender_table: Table,
    defences_memory: HashMap<u64, Vec<Point>>,
    vcf_solver: vcf::iddfs::Solver,
}

impl LazyDFPNSolver {
    pub fn init() -> Self {
        Self {
            attacker_table: Table::new(),
            defender_table: Table::new(),
            defences_memory: HashMap::new(),
            vcf_solver: vcf::iddfs::Solver::init((1..u8::MAX).collect()),
        }
    }
}

impl Solver for LazyDFPNSolver {}

impl Searcher for LazyDFPNSolver {}

impl ProofTree for LazyDFPNSolver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Generator for LazyDFPNSolver {
    fn find_attacks(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        LazyGenerator::find_attacks(self, state, threshold)
    }

    fn find_defences(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        LazyGenerator::find_defences(self, state, threshold)
    }
}

impl LazyGenerator for LazyDFPNSolver {
    fn defences_memory(&mut self) -> &mut HashMap<u64, Vec<Point>> {
        &mut self.defences_memory
    }
}

impl Traverser for LazyDFPNSolver {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        DFPNSTraverser::next_threshold_attack(self, selection, threshold)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        DFPNSTraverser::next_threshold_defence(self, selection, threshold)
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) {
        Searcher::expand_attack(self, state, attack, threshold);
    }

    fn expand_defence(&mut self, state: &mut State, attack: Point, threshold: Node) {
        Searcher::expand_defence(self, state, attack, threshold);
    }
}

impl DFPNSTraverser for LazyDFPNSolver {}

impl Resolver for LazyDFPNSolver {
    fn solve_attacker_vcf(&mut self, state: &State) -> Option<Mate> {
        self.vcf_solver.solve(&mut state.vcf_state())
    }

    fn solve_attacker_threat(&mut self, state: &State) -> Option<Mate> {
        self.vcf_solver.solve(&mut state.threat_state())
    }
}
