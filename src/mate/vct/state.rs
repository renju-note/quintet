use super::field::*;
use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub struct State {
    game: Game,
    field: PotentialField,
    threat_limit: u8,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
}

impl State {
    pub fn new(game: Game, field: PotentialField, threat_limit: u8) -> Self {
        Self {
            game: game,
            field: field,
            threat_limit: threat_limit,
            attacker_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defender_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
        }
    }

    pub fn init(board: Board, attacker: Player, limit: u8, threat_limit: u8) -> Self {
        let field = PotentialField::init(attacker, 2, &board);
        let game = Game::new(board, attacker, limit);
        Self::new(game, field, threat_limit)
    }

    pub fn play(&mut self, next_move: Option<Point>) {
        self.game.play(next_move);
        if let Some(next_move) = next_move {
            self.field.update_along(next_move, self.game.board());
        }
    }

    pub fn undo(&mut self) {
        let maybe_last_move = self.game().last_move();
        self.game.undo();
        if let Some(last_move) = maybe_last_move {
            self.field.update_along(last_move, self.game.board());
        }
    }

    pub fn into_play<F, T>(&mut self, next_move: Option<Point>, mut f: F) -> T
    where
        F: FnMut(&mut Self) -> T,
    {
        self.play(next_move);
        let result = f(self);
        self.undo();
        result
    }

    pub fn vcf_state(&self) -> vcf::State {
        vcf::State::new(self.game.clone())
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn next_zobrist_hash(&mut self, next_move: Option<Point>) -> u64 {
        // Update only state in order not to cause updating state.field (which costs high)
        self.game.into_play(next_move, |g| g.zobrist_hash())
    }

    pub fn solve_attacker_vcf(&mut self) -> Option<Mate> {
        if !self.game.attacking() {
            panic!()
        }
        let state = &mut self.vcf_state();
        // this limit can be changed dynamically
        self.attacker_vcf_solver.solve(state)
    }

    pub fn solve_defender_vcf(&mut self) -> Option<Mate> {
        if self.game.attacking() {
            panic!()
        }
        let state = &mut self.vcf_state();
        state.set_limit(u8::MAX);
        self.defender_vcf_solver.solve(state)
    }

    pub fn solve_attacker_threat(&mut self) -> Option<Mate> {
        if self.game.attacking() {
            panic!()
        }
        let state = &mut self.vcf_state();
        state.play(None);
        state.set_limit(state.game().limit.min(self.threat_limit));
        self.attacker_vcf_solver.solve(state)
    }

    pub fn solve_defender_threat(&mut self) -> Option<Mate> {
        if !self.game.attacking() {
            panic!()
        }
        let state = &mut self.vcf_state();
        state.play(None);
        // this limit can be changed dynamically
        state.set_limit(u8::MAX);
        self.defender_vcf_solver.solve(state)
    }

    pub fn sorted_attacks(&mut self, maybe_threat: Option<Mate>) -> Vec<Point> {
        let mut potentials = self.potentials();
        if let Some(threat) = maybe_threat {
            let threat_defences = self.threat_defences(&threat);
            potentials.retain(|(p, _)| threat_defences.contains(p));
        }
        potentials.sort_by(|a, b| b.1.cmp(&a.1));
        potentials.retain(|&(p, _)| !self.game().is_forbidden_move(p));
        potentials.into_iter().map(|t| t.0).collect()
    }

    pub fn sorted_defences(&mut self, threat: Mate) -> Vec<Point> {
        let mut result = self.threat_defences(&threat);
        let field = &self.field;
        result.sort_by(|&a, &b| field.get(b).cmp(&field.get(a)));
        result.dedup();
        result.retain(|&p| !self.game().is_forbidden_move(p));
        result
    }

    fn potentials(&self) -> Vec<(Point, u8)> {
        let min = if self.game.attacker == Player::Black {
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
                result.extend(self.game.board().neighbors(p, 5, true));
            }
            _ => (),
        }
        result
    }

    fn counter_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut game = self.game().clone();
        game.play(None);
        let threater = game.turn;
        let mut result = vec![];
        for &p in &threat.path {
            let turn = game.turn;
            game.play(Some(p));
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
        self.game()
            .board()
            .structures(self.game().turn, Sword)
            .flat_map(|s| s.eyes())
            .collect()
    }
}
