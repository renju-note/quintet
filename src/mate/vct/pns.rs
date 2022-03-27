use super::resolver::*;
use super::searcher::*;
use super::state::State;
use super::table::*;
use crate::mate::mate::*;

pub struct Solver {
    attacker_table: Table,
    defender_table: Table,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            attacker_table: Table::new(),
            defender_table: Table::new(),
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
    fn attacker_table(&mut self) -> &mut Table {
        &mut self.attacker_table
    }

    fn defender_table(&mut self) -> &mut Table {
        &mut self.defender_table
    }

    fn calc_next_threshold_attack(&self, selection: &Selection, _threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        )
    }

    fn calc_next_threshold_defence(&self, selection: &Selection, _threshold: Node) -> Node {
        let next = selection.next1;
        Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        )
    }
}

impl Resolver for Solver {
    fn attacker_table(&self) -> &Table {
        &self.attacker_table
    }

    fn defender_table(&self) -> &Table {
        &self.defender_table
    }
}
