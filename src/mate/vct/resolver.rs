use super::state::State;
use super::table::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub struct Resolver {
    table: Table,
    vcf_solver: vcf::iddfs::Solver,
}

impl Resolver {
    pub fn init(table: Table) -> Self {
        Self {
            table: table,
            vcf_solver: vcf::iddfs::Solver::init((1..u8::MAX).collect()),
        }
    }

    pub fn resolve(&mut self, state: &mut State) -> Option<Mate> {
        self.resolve_attacks(state)
    }

    fn resolve_attacks(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Forced(m) => self.resolve_attack(state, m),
                _ => None,
            };
        }

        let attacks = state.sorted_attacks(None);
        for attack in attacks {
            let node = self
                .table
                .lookup_next(state, attack)
                .unwrap_or(Node::dummy());
            if node.pn == 0 {
                return self.resolve_attack(state, attack);
            }
        }

        let vcf_state = &mut vcf::State::new(state.game().clone(), state.limit());
        self.vcf_solver.solve(vcf_state)
    }

    fn resolve_attack(&mut self, state: &mut State, attack: Point) -> Option<Mate> {
        state.play(attack);
        let result = self.resolve_defences(state).map(|m| m.unshift(attack));
        state.undo();
        result
    }

    fn resolve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(end) => Some(Mate::new(end, vec![])),
                Forced(m) => self.resolve_defence(state, m),
            };
        }

        let maybe_threat = state.solve_threat();
        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut min_limit = u8::MAX;
        let mut selected_defence = Point(0, 0);
        for defence in defences {
            let node = self
                .table
                .lookup_next(state, defence)
                .unwrap_or(Node::dummy());
            if node.pn == 0 && node.limit < min_limit {
                min_limit = node.limit;
                selected_defence = defence;
            }
        }
        self.resolve_defence(state, selected_defence)
    }

    fn resolve_defence(&mut self, state: &mut State, defence: Point) -> Option<Mate> {
        state.play(defence);
        let result = self.resolve_attacks(state).map(|m| m.unshift(defence));
        state.undo();
        result
    }
}
