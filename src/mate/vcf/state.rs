use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::state::State;

#[derive(Clone)]
pub struct VCFState {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
}

impl VCFState {
    pub fn new(game: Game, limit: u8) -> Self {
        Self {
            attacker: game.turn,
            game: game,
            limit,
        }
    }

    pub fn init(board: &Board, attacker: Player, limit: u8) -> Self {
        let game = Game::init(board, attacker);
        Self::new(game, limit)
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.game().is_forbidden_move(p)
    }

    pub fn check_event(&self) -> Option<Event> {
        self.game().check_event()
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

impl State for VCFState {
    fn game(&self) -> &Game {
        &self.game
    }

    fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    fn attacker(&self) -> Player {
        self.attacker
    }

    fn limit(&self) -> u8 {
        self.limit
    }

    fn set_limit(&mut self, limit: u8) {
        self.limit = limit
    }
}
