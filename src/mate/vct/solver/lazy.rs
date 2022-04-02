use crate::board::Point;
use crate::mate::mate::Mate;
use crate::mate::vcf;
use crate::mate::vct::generator::*;
use crate::mate::vct::helper;
use crate::mate::vct::proof::*;
use crate::mate::vct::resolver::*;
use crate::mate::vct::searcher;
use crate::mate::vct::solver;
use crate::mate::vct::state::State;
use crate::mate::vct::traverser::*;
use std::collections::HashMap;

pub struct Solver {
    attacker_table: Table,
    defender_table: Table,
    attacker_vcf_depth: u8,
    defender_vcf_depth: u8,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
    defences_memory: HashMap<u64, Vec<Point>>,
}

impl ProofTree for Solver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Solver {
    pub fn init(attacker_vcf_depth: u8, defender_vcf_depth: u8) -> Self {
        Self {
            attacker_table: Table::new(),
            defender_table: Table::new(),
            attacker_vcf_depth: attacker_vcf_depth,
            defender_vcf_depth: defender_vcf_depth,
            attacker_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defender_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defences_memory: HashMap::new(),
        }
    }

    pub fn solve(&mut self, state: &mut State) -> Option<Mate> {
        solver::Solver::solve(self, state)
    }
}

impl solver::Solver for Solver {}

impl helper::VCFHelper for Solver {
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

impl Traverser for Solver {
    fn select_attack(&mut self, state: &mut State, attacks: &[Point]) -> Selection {
        DFPNSTraverser::select_attack(self, state, attacks)
    }

    fn select_defence(&mut self, state: &mut State, defences: &[Point]) -> Selection {
        DFPNSTraverser::select_defence(self, state, defences)
    }

    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        DFPNSTraverser::next_threshold_attack(self, selection, threshold)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        DFPNSTraverser::next_threshold_defence(self, selection, threshold)
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) {
        searcher::Searcher::expand_attack(self, state, attack, threshold);
    }

    fn expand_defence(&mut self, state: &mut State, attack: Point, threshold: Node) {
        searcher::Searcher::expand_defence(self, state, attack, threshold);
    }
}

impl DFPNSTraverser for Solver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Generator for Solver {
    fn find_attacks(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        LazyGenerator::find_attacks(self, state, threshold)
    }

    fn find_defences(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        LazyGenerator::find_defences(self, state, threshold)
    }
}

impl LazyGenerator for Solver {
    fn defences_memory(&mut self) -> &mut HashMap<u64, Vec<Point>> {
        &mut self.defences_memory
    }
}

impl searcher::Searcher for Solver {}

impl Resolver for Solver {
    fn solve_attacker_vcf(&mut self, state: &State) -> Option<Mate> {
        helper::VCFHelper::solve_attacker_vcf(self, state)
    }

    fn solve_attacker_threat(&mut self, state: &State) -> Option<Mate> {
        helper::VCFHelper::solve_attacker_threat(self, state)
    }
}
