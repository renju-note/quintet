use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;

pub struct State {
    game: Game,
}

impl State {
    pub fn new(game: Game) -> Self {
        Self { game: game }
    }

    pub fn init(board: Board, turn: Player) -> Self {
        Self::new(Game::init(board, turn))
    }

    pub fn game(&mut self) -> &'_ mut Game {
        &mut self.game
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
