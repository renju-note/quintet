use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::state::State;

#[derive(Clone)]
pub struct LocalVCTState {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
}

impl LocalVCTState {
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

    pub fn forced_moveset(&self, forced_move: Point) -> Option<Moveset> {
        self.game
            .board()
            .structures_on(forced_move, self.game.turn, Sword)
            .flat_map(Self::sword_eyes_pairs)
            .filter(|&(e1, _)| e1 == forced_move)
            .next()
            .map(Into::into)
    }

    pub fn neighbor_movesets(&self) -> Vec<Moveset> {
        if let Some(last2_move) = self.game.last2_move() {
            self.game
                .board()
                .structures_on(last2_move, self.game.turn, Sword)
                .flat_map(Self::sword_eyes_pairs)
                .map(Into::into)
                .collect()
        } else {
            vec![]
        }
    }

    pub fn movesets(&self) -> Vec<Moveset> {
        self.game
            .board()
            .structures(self.game.turn, Sword)
            .flat_map(Self::sword_eyes_pairs)
            .map(Into::into)
            .collect()
    }

    fn sword_eyes_pairs(sword: Structure) -> [(Point, Point); 2] {
        let mut eyes = sword.eyes();
        let e1 = eyes.next().unwrap();
        let e2 = eyes.next().unwrap();
        [(e1, e2), (e2, e1)]
    }
}

impl State for LocalVCTState {
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

pub struct Moveset {
    pub attack: Point,
    pub defences: Vec<Point>,
}

impl Moveset {
    pub fn new(attack: Point, defences: Vec<Point>) -> Self {
        Self {
            attack: attack,
            defences: defences,
        }
    }
}

impl From<(Point, Point)> for Moveset {
    fn from((a, d): (Point, Point)) -> Self {
        Self {
            attack: a,
            defences: vec![d],
        }
    }
}
