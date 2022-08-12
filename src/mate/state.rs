use super::game::*;
use crate::board::*;

pub trait State {
    fn game(&self) -> &Game;
    fn game_mut(&mut self) -> &mut Game;
    fn attacker(&self) -> Player;
    fn limit(&self) -> u8;
    fn set_limit(&mut self, limit: u8);

    fn play(&mut self, next_move: Option<Point>) {
        self.game_mut().play(next_move);
        if self.attacking() {
            self.set_limit(self.limit() - 1)
        }
        self.after_play(next_move);
    }

    fn after_play(&mut self, _next_move: Option<Point>) {}

    fn undo(&mut self) {
        let maybe_last_move = self.game().last_move();
        if self.attacking() {
            self.set_limit(self.limit() + 1)
        }
        self.game_mut().undo();
        self.after_undo(maybe_last_move);
    }

    fn after_undo(&mut self, _maybe_last_move: Option<Point>) {}

    fn into_play<F, T>(&mut self, next_move: Option<Point>, mut f: F) -> T
    where
        F: FnMut(&mut Self) -> T,
    {
        self.play(next_move);
        let result = f(self);
        self.undo();
        result
    }

    fn attacking(&self) -> bool {
        self.game().turn == self.attacker()
    }

    fn zobrist_hash(&self) -> u64 {
        self.game().zobrist_hash(self.limit())
    }
}
