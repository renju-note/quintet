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

    #[allow(clippy::wrong_self_convention)]
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

    /// The key every memo in the solvers is stored under: the position, the
    /// turn and the remaining limit (all from [`Game::zobrist_hash`]), plus
    /// the attacker.
    ///
    /// The attacker is what lets one solver answer questions about both
    /// sides without forgetting what it learned in between: the same
    /// position is a win for one of them and not the other, so an entry made
    /// while attacking as Black must not be read while attacking as White.
    fn zobrist_hash(&self) -> u64 {
        apply_attacker(self.game().zobrist_hash(self.limit()), self.attacker())
    }
}
