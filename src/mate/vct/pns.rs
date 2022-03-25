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

    fn calc_next_threshold_attack(&self, selection: &Selection, _current_threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        )
    }

    fn calc_next_threshold_defence(&self, selection: &Selection, _current_threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        )
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
