use super::resolver::*;
use super::searcher::*;
use super::solver;
use super::state::State;
use super::table::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub struct Solver {
    threat_limit: u8,
    attacker_table: Table,
    defender_table: Table,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
}

impl Solver {
    pub fn init(threat_limit: u8) -> Self {
        Self {
            threat_limit: threat_limit,
            attacker_table: Table::new(),
            defender_table: Table::new(),
            attacker_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defender_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
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

impl solver::Solver for Solver {
    fn threat_limit(&self) -> u8 {
        self.threat_limit
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

impl Searcher for Solver {
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

impl Resolver for Solver {}
