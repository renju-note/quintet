use super::resolver::*;
use super::searcher::*;
use super::state::State;
use super::table::*;
use crate::mate::mate::*;
use crate::mate::vcf;

// MEMO: Debug printing example is 6e2bace

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

/*
Df-pn algorithm is proposed in the following paper:

Nagai, Ayumu, and Hiroshi Imai.
"Proof for the equivalence between some best-first algorithms and depth-first algorithms for AND/OR trees."
IEICE TRANSACTIONS on Information and Systems 85.10 (2002): 1645-1653.
*/
impl Searcher for Solver {
    fn table(&mut self) -> &mut Table {
        &mut self.table
    }

    fn calc_next_threshold_attack(&self, selection: &Selection, current_threshold: Node) -> Node {
        let pn = current_threshold
            .pn
            .min(selection.next2.pn.checked_add(1).unwrap_or(INF));
        let dn = (current_threshold.dn - selection.current.dn)
            .checked_add(selection.next1.dn)
            .unwrap_or(INF);
        Node::new(pn, dn, selection.next1.limit)
    }

    fn calc_next_threshold_defence(&self, selection: &Selection, current_threshold: Node) -> Node {
        let pn = (current_threshold.pn - selection.current.pn)
            .checked_add(selection.next1.pn)
            .unwrap_or(INF);
        let dn = current_threshold
            .dn
            .min(selection.next2.dn.checked_add(1).unwrap_or(INF));
        Node::new(pn, dn, selection.next1.limit)
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
