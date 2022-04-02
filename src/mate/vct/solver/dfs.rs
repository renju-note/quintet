use crate::board::Point;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct::generator::*;
use crate::mate::vct::helper;
use crate::mate::vct::proof::*;
use crate::mate::vct::resolver::*;
use crate::mate::vct::searcher::Searcher;
use crate::mate::vct::solver::Solver;
use crate::mate::vct::state::State;
use crate::mate::vct::traverser::*;

pub struct EagerDFSSolver {
    attacker_table: Table,
    defender_table: Table,
    attacker_vcf_depth: u8,
    defender_vcf_depth: u8,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
}

impl EagerDFSSolver {
    pub fn init(attacker_vcf_depth: u8, defender_vcf_depth: u8) -> Self {
        Self {
            attacker_table: Table::new(),
            defender_table: Table::new(),
            attacker_vcf_depth: attacker_vcf_depth,
            defender_vcf_depth: defender_vcf_depth,
            attacker_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defender_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
        }
    }
}

impl Solver for EagerDFSSolver {}

impl Searcher for EagerDFSSolver {}

impl ProofTree for EagerDFSSolver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Generator for EagerDFSSolver {
    fn generate_attacks(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        EagerGenerator::generate_attacks(self, state, threshold)
    }

    fn generate_defences(
        &mut self,
        state: &mut State,
        threshold: Node,
    ) -> Result<Vec<Point>, Node> {
        EagerGenerator::generate_defences(self, state, threshold)
    }
}

impl EagerGenerator for EagerDFSSolver {}

impl Traverser for EagerDFSSolver {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        DFSTraverser::next_threshold_attack(self, selection, threshold)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        DFSTraverser::next_threshold_defence(self, selection, threshold)
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) {
        Searcher::expand_attack(self, state, attack, threshold);
    }

    fn expand_defence(&mut self, state: &mut State, attack: Point, threshold: Node) {
        Searcher::expand_defence(self, state, attack, threshold);
    }
}

impl DFSTraverser for EagerDFSSolver {}

impl Resolver for EagerDFSSolver {
    fn solve_attacker_vcf(&mut self, state: &State) -> Option<Mate> {
        helper::VCFHelper::solve_attacker_vcf(self, state)
    }

    fn solve_attacker_threat(&mut self, state: &State) -> Option<Mate> {
        helper::VCFHelper::solve_attacker_threat(self, state)
    }
}

impl helper::VCFHelper for EagerDFSSolver {
    fn attacker_vcf_depth(&self) -> u8 {
        self.attacker_vcf_depth
    }

    fn defender_vcf_depth(&self) -> u8 {
        self.defender_vcf_depth
    }

    fn attacker_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver {
        &mut self.attacker_vcf_solver
    }

    fn defender_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver {
        &mut self.defender_vcf_solver
    }
}
