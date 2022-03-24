use super::field::*;
use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub struct State {
    game: Game,
    field: PotentialField,
    pub attacker: Player,
    pub limit: u8,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
}

impl State {
    pub fn new(game: Game, field: PotentialField, limit: u8) -> Self {
        let attacker = game.turn();
        Self {
            attacker: attacker,
            limit: limit,
            game: game,
            field: field,
            attacker_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defender_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
        }
    }

    pub fn init(board: Board, turn: Player, limit: u8) -> Self {
        let field = PotentialField::init(turn, 2, &board);
        let game = Game::init(board, turn);
        Self::new(game, field, limit)
    }

    pub fn play(&mut self, next_move: Point) {
        self.game.play(next_move);
        self.field.update_along(next_move, self.game.board());
        if self.game.turn() == self.attacker {
            self.limit -= 1
        }
    }

    pub fn undo(&mut self, last2_move: Point) {
        if self.game.turn() == self.attacker {
            self.limit += 1
        }
        let last_move = self.game.last_move();
        self.game.undo(last2_move);
        self.field.update_along(last_move, self.game.board());
    }

    pub fn game(&self) -> &'_ Game {
        &self.game
    }

    pub fn turn(&self) -> Player {
        self.game.turn()
    }

    pub fn attacking(&self) -> bool {
        self.game.turn() == self.attacker
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.game.zobrist_hash(self.limit)
    }

    pub fn next_zobrist_hash(&mut self, next_move: Point) -> u64 {
        // Extract game in order not to cause updating state.field (which costs high)
        let last2_move = self.game.last2_move();
        self.game.play(next_move);
        let next_limit = self.limit - if self.attacking() { 1 } else { 0 };
        let result = self.game.zobrist_hash(next_limit);
        self.game.undo(last2_move);
        result
    }

    pub fn solve_vcf(&mut self) -> Option<Mate> {
        let game = self.game().clone();
        if self.attacking() {
            let state = &mut vcf::State::new(game, self.limit);
            self.attacker_vcf_solver.solve(state)
        } else {
            let state = &mut vcf::State::new(game, u8::MAX);
            self.defender_vcf_solver.solve(state)
        }
    }

    pub fn solve_threat(&mut self) -> Option<Mate> {
        let game = self.game().pass();
        if self.attacking() {
            let state = &mut vcf::State::new(game, u8::MAX);
            self.defender_vcf_solver.solve(state)
        } else {
            let state = &mut vcf::State::new(game, self.limit - 1);
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
        potentials.retain(|&(p, _)| !self.game.is_forbidden_move(p));
        potentials.into_iter().map(|t| t.0).collect()
    }

    pub fn sorted_defences(&mut self, threat: Mate) -> Vec<Point> {
        let mut result = self.threat_defences(&threat);
        let field = &self.field;
        result.sort_by(|&a, &b| field.get(b).cmp(&field.get(a)));
        result.dedup();
        result.retain(|&p| !self.game.is_forbidden_move(p));
        result
    }

    fn potentials(&self) -> Vec<(Point, u8)> {
        let min = if self.attacker == Player::Black { 4 } else { 3 };
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
                result.extend(self.game.board().neighbors(p, 5, true));
            }
            _ => (),
        }
        result
    }

    fn counter_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut game = self.game.pass();
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
        self.game
            .board()
            .structures(self.turn(), Sword)
            .flat_map(|s| s.eyes())
            .collect()
    }
}
