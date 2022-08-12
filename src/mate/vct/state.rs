use super::field::*;
use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::vcf;

pub struct State {
    game: Game,
    pub limit: u8,
    field: PotentialField,
}

impl State {
    pub fn new(game: Game, limit: u8, field: PotentialField) -> Self {
        Self {
            game: game,
            limit: limit,
            field: field,
        }
    }

    pub fn init(board: &Board, attacker: Player, limit: u8) -> Self {
        let game = Game::init(board, attacker);
        let field = PotentialField::init(attacker, 2, board);
        Self::new(game, limit, field)
    }

    pub fn play(&mut self, next_move: Option<Point>) {
        self.game.play(next_move);
        if self.game.attacking() {
            self.limit -= 1
        }
        if let Some(next_move) = next_move {
            self.field.update_along(next_move, self.game.board());
        }
    }

    pub fn undo(&mut self) {
        if self.game.attacking() {
            self.limit += 1
        }
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

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.game.zobrist_hash(self.limit)
    }

    pub fn vcf_state(&self, max_limit: u8) -> vcf::State {
        let game = self.game.clone();
        let limit = self.limit.min(max_limit);
        vcf::State::new(game, limit)
    }

    pub fn threat_state(&self, max_limit: u8) -> vcf::State {
        let mut game = self.game.clone();
        let limit = if game.attacking() {
            self.limit - 1
        } else {
            self.limit
        };
        game.play(None);
        let limit = limit.min(max_limit);
        vcf::State::new(game, limit)
    }

    pub fn next_zobrist_hash(&mut self, next_move: Option<Point>) -> u64 {
        // Update only game in order not to cause updating state.field (which costs high)
        let limit = self.limit;
        self.game.into_play(next_move, |g| {
            let next_limit = if g.attacking() { limit - 1 } else { limit };
            g.zobrist_hash(next_limit)
        })
    }

    pub fn is_four_move(&self, forced_move: Point) -> bool {
        self.game
            .board()
            .structures_on(forced_move, self.game.turn, Sword)
            .flat_map(|s| s.eyes())
            .any(|e| e == forced_move)
    }

    pub fn sorted_potentials(&self, min: u8, only: Option<Vec<Point>>) -> Vec<(Point, u8)> {
        let mut result = if let Some(only) = only {
            let potentials = only.into_iter().map(|p| (p, self.field.get(p)));
            potentials.filter(|&(_, o)| o >= min).collect()
        } else {
            self.field.collect(min)
        };
        result.sort_by(|&a, &b| b.1.cmp(&a.1));
        result.dedup();
        result
    }

    pub fn sort_by_potential(&self, points: Vec<Point>) -> Vec<(Point, u8)> {
        let mut result: Vec<_> = points.into_iter().map(|p| (p, self.field.get(p))).collect();
        result.sort_by(|&a, &b| b.1.cmp(&a.1));
        result.dedup();
        result
    }

    pub fn threat_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = threat.path().clone();
        result.extend(self.end_breakers(threat.end().clone()));
        result.extend(self.counter_defences(threat));
        result.extend(self.four_moves());
        result
    }

    pub fn end_breakers(&self, end: End) -> Vec<Point> {
        match end {
            Fours(p1, p2) => {
                vec![p1, p2]
            }
            Forbidden(p) => {
                let mut ds = vec![p];
                ds.extend(self.game().board().neighbors(p, 5, true));
                ds
            }
            _ => vec![],
        }
    }

    pub fn next_sword_eyes(&mut self, p: Point) -> Vec<Point> {
        self.game.into_play(Some(p), |g| {
            g.board()
                .structures_on(g.last_move().unwrap(), g.turn.opponent(), Sword)
                .map(|s| s.eyes())
                .flatten()
                .collect()
        })
    }

    pub fn four_moves(&self) -> Vec<Point> {
        self.game()
            .board()
            .structures(self.game().turn, Sword)
            .flat_map(|s| s.eyes())
            .collect()
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
}
