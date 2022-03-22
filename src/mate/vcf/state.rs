use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;

pub struct State {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
}

impl State {
    pub fn new(game: Game, limit: u8) -> Self {
        let attacker = game.turn();
        Self {
            game: game,
            attacker: attacker,
            limit: limit,
        }
    }

    pub fn init(board: Board, turn: Player, limit: u8) -> Self {
        Self::new(Game::init(board, turn), limit)
    }

    pub fn play(&mut self, next_move: Point) {
        self.game.play(next_move);
        if self.game.turn() == self.attacker {
            self.limit -= 1
        }
    }

    pub fn undo(&mut self, last2_move: Point) {
        if self.game.turn() == self.attacker {
            self.limit += 1
        }
        self.game.undo(last2_move);
    }

    pub fn game(&mut self) -> &'_ mut Game {
        &mut self.game
    }

    pub fn set_limit(&mut self, limit: u8) {
        self.limit = limit
    }

    pub fn zobrist_hash(&self) -> u64 {
        self.game.zobrist_hash(self.limit)
    }

    pub fn forced_move_pair(&self, forced_move: Point) -> Option<(Point, Point)> {
        self.game
            .board()
            .structures_on(forced_move, self.game.turn(), Sword)
            .flat_map(Self::sword_eyes_pairs)
            .filter(|&(e1, _)| e1 == forced_move)
            .next()
    }

    pub fn neighbor_move_pairs(&self) -> Vec<(Point, Point)> {
        self.game
            .board()
            .structures_on(self.game.last2_move(), self.game.turn(), Sword)
            .flat_map(Self::sword_eyes_pairs)
            .collect()
    }

    pub fn move_pairs(&self) -> Vec<(Point, Point)> {
        self.game
            .board()
            .structures(self.game.turn(), Sword)
            .flat_map(Self::sword_eyes_pairs)
            .collect()
    }

    fn sword_eyes_pairs(sword: Structure) -> [(Point, Point); 2] {
        let mut eyes = sword.eyes();
        let e1 = eyes.next().unwrap();
        let e2 = eyes.next().unwrap();
        [(e1, e2), (e2, e1)]
    }
}
