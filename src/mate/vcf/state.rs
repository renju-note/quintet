use crate::analysis::sword::SwordEyeCache;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::state::State;

#[derive(Clone)]
pub struct VCFState {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
    /// Where the attacker can make a four. Every generator below reads it
    /// instead of the board; `after_play` / `after_undo` keep it in step.
    ///
    /// It is the attacker's alone, and that is enough: the three generators
    /// are only ever asked at an attacker node (`DFSSolver::search` recurses
    /// with the attacker to move), and the attacker of a `VCFState` never
    /// changes.
    sword_eyes: SwordEyeCache,
}

impl VCFState {
    pub fn new(game: Game, limit: u8) -> Self {
        let sword_eyes = SwordEyeCache::new(game.turn);
        Self {
            attacker: game.turn,
            game,
            limit,
            sword_eyes,
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

    pub fn forced_move_pair(&mut self, forced_move: Point) -> Option<(Point, Point)> {
        self.sword_eyes
            .eye_partner(forced_move, self.game.board())
            .map(|defence| (forced_move, defence))
    }

    pub fn neighbor_move_pairs(&mut self) -> Vec<(Point, Point)> {
        let mut result = vec![];
        if let Some(last2_move) = self.game.last2_move() {
            self.sword_eyes
                .extend_eye_pairs_on(last2_move, self.game.board(), &mut result);
        }
        result
    }

    pub fn move_pairs(&mut self) -> Vec<(Point, Point)> {
        let mut result = vec![];
        self.sword_eyes
            .extend_eye_pairs(self.game.board(), &mut result);
        result
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

    fn after_play(&mut self, next_move: Option<Point>) {
        if let Some(next_move) = next_move {
            self.sword_eyes.play_along(next_move);
        }
    }

    fn after_undo(&mut self, maybe_last_move: Option<Point>) {
        if let Some(last_move) = maybe_last_move {
            self.sword_eyes.play_along(last_move);
        }
    }
}
