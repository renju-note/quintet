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
                Forced(attack) => state.into_play(attack, |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                }),
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
                return state.into_play(attack, |s| {
                    self.resolve_defences(s).map(|m| m.unshift(attack))
                });
            }
        }

        let vcf_state = &mut vcf::State::new(state.game().clone(), state.limit());
        self.vcf_solver.solve(vcf_state)
    }

    fn resolve_defences(&mut self, state: &mut State) -> Option<Mate> {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(end) => Some(Mate::new(end, vec![])),
                Forced(defence) => state.into_play(defence, |s| {
                    self.resolve_attacks(s).map(|m| m.unshift(defence))
                }),
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
        state.into_play(selected_defence, |s| {
            self.resolve_attacks(s).map(|m| m.unshift(selected_defence))
        })
    }
}
