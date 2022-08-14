use std::vec;

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
            .flat_map(Self::sword_to_movesets)
            .filter(|ms| ms.attack == forced_move)
            .next()
    }

    pub fn four_movesets(&self) -> Vec<Moveset> {
        let mut result = vec![];
        if let Some(last2_move) = self.game.last2_move() {
            result.extend(self.four_movesets_on(last2_move));
        };
        if let Some(last_move) = self.game.last_move() {
            result.extend(self.four_movesets_on(last_move));
        };
        result.sort_by(|a, b| a.attack.cmp(&b.attack));
        // TODO: not dedup but merge (intersect defences)
        result.dedup_by(|a, b| a.attack == b.attack);
        result
    }

    pub fn three_movesets(&self) -> Vec<Moveset> {
        let mut result = vec![];
        if let Some(last2_move) = self.game.last2_move() {
            result.extend(self.three_movesets_on(last2_move));
        };
        if let Some(last_move) = self.game.last_move() {
            result.extend(self.three_movesets_on(last_move));
        };
        result.sort_by(|a, b| a.attack.cmp(&b.attack));
        // TODO: not dedup but merge (intersect defences)
        result.dedup_by(|a, b| a.attack == b.attack);
        result
    }

    // TODO: add trapping movesets
    // 1. get local forbiddens
    // 2. get single two sequences (maybe dagger?) over them

    fn four_movesets_on(&self, p: Point) -> Vec<Moveset> {
        self.game
            .board()
            .structures_on(p, self.game.turn, Sword)
            .flat_map(Self::sword_to_movesets)
            .collect()
    }

    fn three_movesets_on(&self, p: Point) -> Vec<Moveset> {
        self.game
            .board()
            .structures_on(p, self.game.turn, Two)
            .flat_map(Self::two_to_movesets)
            .collect()
    }

    fn sword_to_movesets(sword: Structure) -> [Moveset; 2] {
        let mut eyes = sword.eyes();
        let e1 = eyes.next().unwrap();
        let e2 = eyes.next().unwrap();
        [Moveset::new(e1, vec![e2]), Moveset::new(e2, vec![e1])]
    }

    fn two_to_movesets(two: Structure) -> [Moveset; 2] {
        let mut eyes = two.eyes();
        let e1 = eyes.next().unwrap();
        let e2 = eyes.next().unwrap();
        let ms = two.start_index().walk_checked(-1).unwrap().to_point();
        let me = two.start_index().walk_checked(4).unwrap().to_point();
        [
            Moveset::new(e1, vec![e2, ms, me]),
            Moveset::new(e2, vec![e1, ms, me]),
        ]
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
