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
        let depths = (1..=u8::MAX).into_iter().collect::<Vec<_>>();
        Resolver {
            table: table,
            vcf_solver: vcf::iddfs::Solver::init(depths),
        }
    }

    pub fn resolve(&mut self, state: &mut State, max_depth: u8) -> Option<Mate> {
        self.resolve_attacks(state, max_depth)
    }

    fn resolve_attacks(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        if let Some(stage) = state.game().check_stage() {
            return match stage {
                Forced(m) => self.resolve_attack(state, limit, m),
                _ => None,
            };
        }

        let attacks = state.sorted_attacks(None);
        let mut game = state.game().clone();
        for attack in attacks {
            let node = self.table.lookup_child(&mut game, attack, limit);
            if node.pn == 0 {
                return self.resolve_attack(state, limit, attack);
            }
        }

        let vcf_state = &mut state.as_vcf();
        vcf_state.set_limit(limit);
        self.vcf_solver.solve(vcf_state)
    }

    fn resolve_attack(&mut self, state: &mut State, limit: u8, attack: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(attack);
        let result = self
            .resolve_defences(state, limit)
            .map(|m| m.unshift(attack));
        state.undo(last2_move);
        result
    }

    fn resolve_defences(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        if let Some(stage) = state.game().check_stage() {
            return match stage {
                End(w) => Some(Mate::new(w, vec![])),
                Forced(m) => self.resolve_defence(state, limit, m),
            };
        }

        let threat_state = &mut state.as_threat();
        threat_state.set_limit(limit - 1);
        let maybe_threat = self.vcf_solver.solve(threat_state);
        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut game = state.game().clone();
        let mut min_limit = u8::MAX;
        let mut selected_defence = Point(0, 0);
        for defence in defences {
            let node = self.table.lookup_child(&mut game, defence, limit);
            if node.pn == 0 && node.limit < min_limit {
                min_limit = node.limit;
                selected_defence = defence;
            }
        }
        self.resolve_defence(state, limit, selected_defence)
    }

    fn resolve_defence(&mut self, state: &mut State, limit: u8, defence: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(defence);
        let limit = limit - 1;
        let result = self
            .resolve_attacks(state, limit)
            .map(|m| m.unshift(defence));
        state.undo(last2_move);
        result
    }
}
