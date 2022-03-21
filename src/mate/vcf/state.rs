use super::super::super::board::StructureKind::*;
use super::super::super::board::*;
use super::super::game::*;

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

    pub fn check_win(&self) -> Option<Win> {
        let (maybe_first, maybe_another) = self.game.check_last_four_eyes();
        if maybe_first.is_some() && maybe_another.is_some() {
            Some(Win::Fours(maybe_first.unwrap(), maybe_another.unwrap()))
        } else if maybe_first.map_or(false, |e| self.game.is_forbidden_move(e)) {
            Some(Win::Forbidden(maybe_first.unwrap()))
        } else {
            None
        }
    }

    pub fn check_mandatory_move_pair(&self) -> Option<Option<(Point, Point)>> {
        let (maybe_first, maybe_another) = self.game.check_last_four_eyes();
        if maybe_another.is_some() {
            return Some(None);
        }
        if maybe_first.is_none() {
            return None;
        }
        let mandatory_move = maybe_first.unwrap();
        let mandatory_move_pair = self
            .game
            .board()
            .structures_on(mandatory_move, self.game.turn(), Sword)
            .flat_map(Self::sword_eyes_pairs)
            .filter(|&(e1, _)| e1 == mandatory_move)
            .next();
        Some(mandatory_move_pair)
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
