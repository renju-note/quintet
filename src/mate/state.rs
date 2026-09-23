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

    fn undo(&mut self) {
        let maybe_last_move = self.game().last_move();
        if self.attacking() {
            self.set_limit(self.limit() + 1)
        }
        self.game_mut().undo();
        self.after_undo(maybe_last_move);
    }

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

    /// What every memo in the solvers is keyed by: the position — the stones
    /// and the turn from [`Game::position_hash`], plus the attacker — and
    /// the remaining limit.
    ///
    /// The attacker is what lets one solver answer questions about both
    /// sides without forgetting what it learned in between: the same
    /// position is a win for one of them and not the other, so an entry made
    /// while attacking as Black must not be read while attacking as White.
    fn key(&self) -> Key {
        let position = apply_attacker(self.game().position_hash(), self.attacker());
        Key::new(position, self.limit())
    }

    /// [`Self::key`] collapsed to one value, for a memo that wants one entry
    /// per limit.
    fn zobrist_hash(&self) -> u64 {
        self.key().hash()
    }

    /// Called at the end of [`Self::play`], for a state that keeps more than
    /// the game in step with the moves.
    fn after_play(&mut self, _next_move: Option<Point>) {}

    /// Called at the end of [`Self::undo`]; see [`Self::after_play`].
    fn after_undo(&mut self, _maybe_last_move: Option<Point>) {}
}

/// What a memo entry is about: a `position` — the stones, whose turn it is
/// and which side the search is for — and the `limit`, how many attacker
/// moves are still allowed.
///
/// The two are kept apart because they are remembered differently. What a
/// search *decides* about a position bounds every other limit: a mate within
/// `limit` is a mate within more, and no mate within `limit` is no mate
/// within less. Proof numbers short of a decision belong to the one limit
/// they were computed at, so those are stored under [`Key::hash`], the two
/// combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub position: u64,
    pub limit: u8,
}

impl Key {
    pub fn new(position: u64, limit: u8) -> Self {
        Self { position, limit }
    }

    /// One entry per limit.
    pub fn hash(&self) -> u64 {
        apply_n(self.position, self.limit)
    }
}
