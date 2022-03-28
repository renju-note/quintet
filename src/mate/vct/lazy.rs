use super::resolver::*;
use super::searcher::*;
use super::solver;
use super::state::State;
use super::table::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::vcf;

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
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }

    fn calc_next_threshold_attack(&self, selection: &Selection, threshold: Node) -> Node {
        let pn = threshold
            .pn
            .min(selection.next2.pn.checked_add(1).unwrap_or(INF));
        let dn = (threshold.dn - selection.current.dn)
            .checked_add(selection.next1.dn)
            .unwrap_or(INF);
        Node::new(pn, dn, selection.next1.limit)
    }

    fn calc_next_threshold_defence(&self, selection: &Selection, threshold: Node) -> Node {
        let pn = (threshold.pn - selection.current.pn)
            .checked_add(selection.next1.pn)
            .unwrap_or(INF);
        let dn = threshold
            .dn
            .min(selection.next2.dn.checked_add(1).unwrap_or(INF));
        Node::new(pn, dn, selection.next1.limit)
    }

    fn search(&mut self, state: &mut State) -> bool {
        let result = self.search_limit(state, Node::inf());
        result.proven()
    }

    fn search_limit(&mut self, state: &mut State, threshold: Node) -> Node {
        if state.limit() == 0 {
            return Node::zero_dn(state.limit());
        }
        self.search_attacks(state, threshold)
    }

    fn search_attacks(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::zero_dn(state.limit()),
                Forced(m) => self.loop_attacks(state, &[m], threshold),
            };
        }

        let attacks = state.sorted_attacks(None);

        self.loop_attacks(state, &attacks, threshold)
    }

    fn loop_attacks(&mut self, state: &mut State, attacks: &[Point], threshold: Node) -> Node {
        loop {
            let selection = self.select_attack(state, &attacks);
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.calc_next_threshold_attack(&selection, threshold);
            self.expand_attack(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn select_attack(&mut self, state: &mut State, attacks: &[Point]) -> Selection {
        let limit = state.limit();
        let mut best: Option<Point> = None;
        let mut current = Node::zero_dn(limit);
        let mut next1 = Node::zero_dn(limit);
        let mut next2 = Node::zero_dn(limit);
        for &attack in attacks {
            let child = self
                .attacker_table
                .lookup_next(state, Some(attack))
                .unwrap_or(Node::init_dn(attacks.len() as u32, limit)); // trick
            current = current.min_pn_sum_dn(child);
            if child.pn < next1.pn {
                best.replace(attack);
                next2 = next1;
                next1 = child;
            } else if child.pn < next2.pn {
                next2 = child;
            }
            if current.pn == 0 {
                current.dn = INF;
                break;
            }
        }
        Selection {
            best: best,
            current: current,
            next1: next1,
            next2: next2,
        }
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) -> Node {
        state.into_play(Some(attack), |s| {
            let result = self.search_defences(s, threshold);
            self.attacker_table.insert(s, result.clone());
            result
        })
    }

    fn search_defences(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::zero_pn(state.limit()),
                Forced(m) => self.loop_defences(state, &[m], threshold),
            };
        }

        let maybe_threat = self.attacker_vcf_solver.solve(&mut state.threat_state());
        if maybe_threat.is_none() {
            return Node::zero_dn(state.limit());
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());

        self.loop_defences(state, &defences, threshold)
    }

    fn loop_defences(&mut self, state: &mut State, defences: &[Point], threshold: Node) -> Node {
        loop {
            let selection = self.select_defence(state, &defences);
            if self.backoff(selection.current, threshold) {
                return selection.current;
            }
            let next_threshold = self.calc_next_threshold_defence(&selection, threshold);
            self.expand_defence(state, selection.best.unwrap(), next_threshold);
        }
    }

    fn select_defence(&mut self, state: &mut State, defences: &[Point]) -> Selection {
        let limit = state.limit();
        let mut best: Option<Point> = None;
        let mut current = Node::zero_pn(limit - 1);
        let mut next1 = Node::zero_pn(limit - 1);
        let mut next2 = Node::zero_pn(limit - 1);
        for &defence in defences {
            let child = self
                .defender_table
                .lookup_next(state, Some(defence))
                .unwrap_or(Node::init_pn(defences.len() as u32, limit - 1)); // trick
            current = current.min_dn_sum_pn(child);
            if child.dn < next1.dn {
                best.replace(defence);
                next2 = next1;
                next1 = child;
            } else if child.dn < next2.dn {
                next2 = child;
            }
            if current.dn == 0 {
                current.pn = INF;
                break;
            }
        }
        Selection {
            best: best,
            current: current,
            next1: next1,
            next2: next2,
        }
    }

    fn expand_defence(&mut self, state: &mut State, defence: Point, threshold: Node) -> Node {
        state.into_play(Some(defence), |s| {
            let result = self.search_limit(s, threshold);
            self.defender_table.insert(s, result.clone());
            result
        })
    }

    fn backoff(&self, current: Node, threshold: Node) -> bool {
        current.pn >= threshold.pn || current.dn >= threshold.dn
    }
}

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

impl Resolver for Solver {}
