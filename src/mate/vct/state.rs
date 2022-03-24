use super::field::*;
use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub struct State {
    state: vcf::State,
    field: PotentialField,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
}

impl State {
    pub fn new(state: vcf::State, field: PotentialField) -> Self {
        Self {
            state: state,
            field: field,
            attacker_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defender_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
        }
    }

    pub fn init(board: Board, turn: Player, limit: u8) -> Self {
        let field = PotentialField::init(turn, 2, &board);
        let state = vcf::State::init(board, turn, limit);
        Self::new(state, field)
    }

    pub fn play(&mut self, next_move: Point) {
        self.state.play(next_move);
        self.field.update_along(next_move, self.state.board());
    }

    pub fn undo(&mut self) {
        let last_move = self.state.game().last_move();
        self.state.undo();
        self.field.update_along(last_move, self.state.board());
    }

    pub fn game(&self) -> &Game {
        self.state.game()
    }

    pub fn limit(&self) -> u8 {
        self.state.limit
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.state.zobrist_hash()
    }

    pub fn next_zobrist_hash(&mut self, next_move: Point) -> u64 {
        // Update only state in order not to cause updating state.field (which costs high)
        self.state.play(next_move);
        let result = self.state.zobrist_hash();
        self.state.undo();
        result
    }

    pub fn solve_vcf(&mut self) -> Option<Mate> {
        if self.state.attacking() {
            let state = &mut self.state.clone();
            self.attacker_vcf_solver.solve(state)
        } else {
            let state = &mut self.state.clone();
            state.set_limit(u8::MAX);
            self.defender_vcf_solver.solve(state)
        }
    }

    pub fn solve_threat(&mut self) -> Option<Mate> {
        if self.state.attacking() {
            let state = &mut self.state.pass();
            state.set_limit(u8::MAX);
            self.defender_vcf_solver.solve(state)
        } else {
            let state = &mut self.state.pass();
            self.attacker_vcf_solver.solve(state)
        }
    }

    pub fn sorted_attacks(&mut self, maybe_threat: Option<Mate>) -> Vec<Point> {
        let mut potentials = self.potentials();
        if let Some(threat) = maybe_threat {
            let threat_defences = self.threat_defences(&threat);
            potentials.retain(|(p, _)| threat_defences.contains(p));
        }
        potentials.sort_by(|a, b| b.1.cmp(&a.1));
        potentials.retain(|&(p, _)| !self.state.game().is_forbidden_move(p));
        potentials.into_iter().map(|t| t.0).collect()
    }

    pub fn sorted_defences(&mut self, threat: Mate) -> Vec<Point> {
        let mut result = self.threat_defences(&threat);
        let field = &self.field;
        result.sort_by(|&a, &b| field.get(b).cmp(&field.get(a)));
        result.dedup();
        result.retain(|&p| !self.state.game().is_forbidden_move(p));
        result
    }

    fn potentials(&self) -> Vec<(Point, u8)> {
        let min = if self.state.attacker == Player::Black {
            4
        } else {
            3
        };
        self.field.collect(min)
    }

    fn threat_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = self.direct_defences(threat);
        result.extend(self.counter_defences(threat));
        result.extend(self.four_moves());
        result
    }

    fn direct_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = threat.path.clone();
        match threat.end {
            Fours(p1, p2) => {
                result.extend([p1, p2]);
            }
            Forbidden(p) => {
                result.push(p);
                result.extend(self.state.board().neighbors(p, 5, true));
            }
            _ => (),
        }
        result
    }

    fn counter_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut game = self.state.game().pass();
        let threater = game.turn();
        let mut result = vec![];
        for &p in &threat.path {
            let turn = game.turn();
            game.play(p);
            if turn == threater {
                continue;
            }
            let swords = game.board().structures_on(p, turn, Sword);
            for s in swords {
                result.extend(s.eyes());
            }
        }
        result
    }

    fn four_moves(&self) -> Vec<Point> {
        self.state
            .game()
            .board()
            .structures(self.state.game().turn(), Sword)
            .flat_map(|s| s.eyes())
            .collect()
    }
}
