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
        Self {
            table: table,
            vcf_solver: vcf::iddfs::Solver::init(depths),
        }
    }

    pub fn resolve(&mut self, state: &mut State) -> Option<Mate> {
        self.resolve_attacks(state)
    }

    fn resolve_attacks(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(stage) = state.game().check_stage() {
            return match stage {
                Forced(m) => self.resolve_attack(state, m),
                _ => None,
            };
        }

        let attacks = state.sorted_attacks(None);
        for attack in attacks {
            let node = self.table.lookup_child(state, attack);
            if node.pn == 0 {
                return self.resolve_attack(state, attack);
            }
        }

        let vcf_state = &mut state.as_vcf();
        self.vcf_solver.solve(vcf_state)
    }

    fn resolve_attack(&mut self, state: &mut State, attack: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(attack);
        let result = self.resolve_defences(state).map(|m| m.unshift(attack));
        state.undo(last2_move);
        result
    }

    fn resolve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(stage) = state.game().check_stage() {
            return match stage {
                End(w) => Some(Mate::new(w, vec![])),
                Forced(m) => self.resolve_defence(state, m),
            };
        }

        let threat_state = &mut state.as_threat();
        let maybe_threat = self.vcf_solver.solve(threat_state);
        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut min_limit = u8::MAX;
        let mut selected_defence = Point(0, 0);
        for defence in defences {
            let node = self.table.lookup_child(state, defence);
            if node.pn == 0 && node.limit < min_limit {
                min_limit = node.limit;
                selected_defence = defence;
            }
        }
        self.resolve_defence(state, selected_defence)
    }

    fn resolve_defence(&mut self, state: &mut State, defence: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(defence);
        let result = self.resolve_attacks(state).map(|m| m.unshift(defence));
        state.undo(last2_move);
        result
    }
}
