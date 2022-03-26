use super::resolver::*;
use super::searcher::*;
use super::state::State;
use super::table::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub struct Solver {
    table: Table,
    vcf_solver: vcf::iddfs::Solver,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            table: Table::new(),
            vcf_solver: vcf::iddfs::Solver::init((1..u8::MAX).collect()),
        }
    }

    pub fn solve(&mut self, state: &mut State) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}

impl Searcher for Solver {
    fn table(&mut self) -> &mut Table {
        &mut self.table
    }

    fn calc_next_threshold_attack(&self, _selection: &Selection, _threshold: Node) -> Node {
        Node::inf()
    }

    fn calc_next_threshold_defence(&self, _selection: &Selection, _threshold: Node) -> Node {
        Node::inf()
    }
}

impl Resolver for Solver {
    fn table(&self) -> &Table {
        &self.table
    }

    fn solve_vcf(&mut self, state: &mut vcf::State) -> Option<Mate> {
        self.vcf_solver.solve(state)
    }
}
