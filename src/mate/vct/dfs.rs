use super::resolver::*;
use super::searcher::*;
use super::state::State;
use super::table::*;
use crate::mate::mate::*;

pub struct Solver {
    table: Table,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            table: Table::new(),
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
}
