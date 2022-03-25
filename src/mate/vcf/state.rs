use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;

#[derive(Clone)]
pub struct State {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
}

impl State {
    pub fn new(game: Game, limit: u8) -> Self {
        let attacker = game.turn;
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
        if self.attacking() {
            self.limit -= 1
        }
    }

    pub fn undo(&mut self) {
        if self.attacking() {
            self.limit += 1
        }
        self.game.undo();
    }

    pub fn into_play<F, T>(&mut self, next_move: Point, mut f: F) -> T
    where
        F: FnMut(&mut Self) -> T,
    {
        self.play(next_move);
        let result = f(self);
        self.undo();
        result
    }

    pub fn pass(&self) -> Self {
        let game = self.game.pass();
        Self::new(game, self.limit)
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn board(&self) -> &Board {
        self.game.board()
    }

    pub fn attacking(&self) -> bool {
        self.game.turn == self.attacker
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
            .structures_on(forced_move, self.game.turn, Sword)
            .flat_map(Self::sword_eyes_pairs)
            .filter(|&(e1, _)| e1 == forced_move)
            .next()
    }

    pub fn neighbor_move_pairs(&self) -> Vec<(Point, Point)> {
        self.game
            .board()
            .structures_on(self.game.last2_move(), self.game.turn, Sword)
            .flat_map(Self::sword_eyes_pairs)
            .collect()
    }

    pub fn move_pairs(&self) -> Vec<(Point, Point)> {
        self.game
            .board()
            .structures(self.game.turn, Sword)
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
