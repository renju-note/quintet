use crate::board::Point;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct::generator::*;
use crate::mate::vct::helper::VCFHelper;
use crate::mate::vct::proof::*;
use crate::mate::vct::resolver::Resolver;
use crate::mate::vct::searcher::Searcher;
use crate::mate::vct::solver::Solver;
use crate::mate::vct::state::State;
use crate::mate::vct::traverser::*;

pub struct EagerPNSSolver {
    attacker_table: Table,
    defender_table: Table,
    attacker_vcf_depth: u8,
    defender_vcf_depth: u8,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
}

impl EagerPNSSolver {
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

impl Solver for EagerPNSSolver {}

impl Searcher for EagerPNSSolver {}

impl ProofTree for EagerPNSSolver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Generator for EagerPNSSolver {
    fn generate_attacks(
        &mut self,
        state: &mut State,
        threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
        EagerGenerator::generate_attacks(self, state, threshold)
    }

    fn generate_defences(
        &mut self,
        state: &mut State,
        threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node> {
        EagerGenerator::generate_defences(self, state, threshold)
    }
}

impl EagerGenerator for EagerPNSSolver {}

impl Traverser for EagerPNSSolver {
    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        PNSTraverser::next_threshold_attack(self, selection, threshold)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        PNSTraverser::next_threshold_defence(self, selection, threshold)
    }
}

impl PNSTraverser for EagerPNSSolver {}

impl Resolver for EagerPNSSolver {
    fn solve_attacker_vcf(&mut self, state: &State) -> Option<Mate> {
        VCFHelper::solve_attacker_vcf(self, state)
    }

    fn solve_attacker_threat(&mut self, state: &State) -> Option<Mate> {
        VCFHelper::solve_attacker_threat(self, state)
    }
}

impl VCFHelper for EagerPNSSolver {
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
