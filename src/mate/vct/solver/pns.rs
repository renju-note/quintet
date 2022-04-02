use crate::board::*;
use crate::mate::mate::*;
use crate::mate::vcf;
use crate::mate::vct::generator;
use crate::mate::vct::proof::*;
use crate::mate::vct::resolver::*;
use crate::mate::vct::searcher;
use crate::mate::vct::solver;
use crate::mate::vct::solver2;
use crate::mate::vct::state::State;
use crate::mate::vct::traverser;
use crate::mate::vct::traverser::base::Selection;

pub struct Solver {
    attacker_table: Table,
    defender_table: Table,
    attacker_vcf_depth: u8,
    defender_vcf_depth: u8,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
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
        }
    }

    pub fn solve(&mut self, state: &mut State) -> Option<Mate> {
        solver2::Solver::solve(self, state)
    }
}

impl solver2::Solver for Solver {}

impl solver::Solver for Solver {
    fn attacker_vcf_depth(&self) -> u8 {
        self.attacker_vcf_depth
    }

    fn defender_vcf_depth(&self) -> u8 {
        self.defender_vcf_depth
    }

    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }

    fn attacker_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver {
        &mut self.attacker_vcf_solver
    }

    fn defender_vcf_solver(&mut self) -> &mut vcf::iddfs::Solver {
        &mut self.defender_vcf_solver
    }
}

impl traverser::base::Traverser for Solver {
    fn select_attack(&mut self, state: &mut State, attacks: &[Point]) -> Selection {
        traverser::pns::Traverser::select_attack(self, state, attacks)
    }

    fn select_defence(&mut self, state: &mut State, defences: &[Point]) -> Selection {
        traverser::pns::Traverser::select_defence(self, state, defences)
    }

    fn next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        traverser::pns::Traverser::next_threshold_attack(self, selection, threshold)
    }

    fn next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        traverser::pns::Traverser::next_threshold_defence(self, selection, threshold)
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) {
        searcher::Searcher::expand_attack(self, state, attack, threshold);
    }

    fn expand_defence(&mut self, state: &mut State, attack: Point, threshold: Node) {
        searcher::Searcher::expand_defence(self, state, attack, threshold);
    }
}

impl traverser::pns::Traverser for Solver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl generator::base::Generator for Solver {
    fn find_attacks(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        generator::eager::Generator::find_attacks(self, state, threshold)
    }

    fn find_defences(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node> {
        generator::eager::Generator::find_defences(self, state, threshold)
    }
}

impl generator::eager::Generator for Solver {
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

impl searcher::Searcher for Solver {
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }
}

impl Resolver for Solver {}
