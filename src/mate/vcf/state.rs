use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;

#[derive(Clone)]
pub struct State {
    game: Game,
}

impl State {
    pub fn new(game: Game) -> Self {
        Self { game: game }
    }

    pub fn init(board: &Board, attacker: Player, limit: u8) -> Self {
        let game = Game::init(board, attacker, limit);
        Self::new(game)
    }

    pub fn play(&mut self, next_move: Point) {
        self.game.play(Some(next_move));
    }

    pub fn undo(&mut self) {
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

    pub fn set_limit(&mut self, limit: u8) {
        self.game.set_limit(limit)
    }

    pub fn game(&self) -> &Game {
        &self.game
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
        if let Some(last2_move) = self.game.last2_move() {
            self.game
                .board()
                .structures_on(last2_move, self.game.turn, Sword)
                .flat_map(Self::sword_eyes_pairs)
                .collect()
        } else {
            vec![]
        }
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
