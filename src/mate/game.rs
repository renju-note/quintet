use crate::board::StructureKind::*;
use crate::board::*;
use std::fmt;

#[derive(Clone)]
pub struct Game {
    board: Board,
    moves: Vec<Option<Point>>,
    pub turn: Player,
}

impl Game {
    pub fn init(board: &Board, turn: Player) -> Self {
        Self {
            board: board.clone(),
            moves: vec![],
            turn,
        }
    }

    pub fn play(&mut self, next_move: Option<Point>) {
        if let Some(next_move) = next_move {
            self.board.put_mut(self.turn, next_move);
        }
        self.moves.push(next_move);
        self.turn = self.turn.opponent();
    }

    pub fn undo(&mut self) {
        self.turn = self.turn.opponent();
        if let Some(last_move) = self.moves.pop().unwrap() {
            self.board.remove_mut(last_move);
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn into_play<F, T>(&mut self, next_move: Option<Point>, mut f: F) -> T
    where
        F: FnMut(&mut Self) -> T,
    {
        self.play(next_move);
        let result = f(self);
        self.undo();
        result
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The stones and whose turn it is. The turn has to be in it because a
    /// pass changes it without touching the board.
    pub fn position_hash(&self) -> u64 {
        apply_turn(self.board.zobrist_hash(), self.turn)
    }

    /// [`Self::position_hash`] combined with a remaining limit `n`.
    pub fn zobrist_hash(&self, n: u8) -> u64 {
        apply_n(self.position_hash(), n)
    }

    pub fn last_move(&self) -> Option<Point> {
        if !self.moves.is_empty() {
            self.moves[self.moves.len() - 1]
        } else {
            None
        }
    }

    pub fn last2_move(&self) -> Option<Point> {
        if self.moves.len() >= 2 {
            self.moves[self.moves.len() - 2]
        } else {
            None
        }
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.turn.is_black() && self.board.forbidden(p).is_some()
    }

    pub fn check_event(&self) -> Option<Event> {
        match self.check_last_four_eyes() {
            (Some(first), Some(another)) => Some(Defeated(Fours(first, another))),
            (Some(first), None) if self.is_forbidden_move(first) => {
                Some(Defeated(Forbidden(first)))
            }
            (Some(first), None) => Some(Forced(first)),
            (None, _) => None,
        }
    }

    pub fn moves_to_string(&self) -> String {
        self.moves
            .iter()
            .map(|m| match m {
                Some(p) => p.to_string(),
                None => "PASS".to_string(),
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn check_last_four_eyes(&self) -> (Option<Point>, Option<Point>) {
        let opponent = self.turn.opponent();
        if let Some(last_move) = self.last_move() {
            Self::take_distinct_two(self.board.structures_on(last_move, opponent, Four))
        } else {
            Self::take_distinct_two(self.board.structures(opponent, Four))
        }
    }

    /// The first two distinct eyes of `fours`, in the order they come.
    ///
    /// The eyes are walked with a loop rather than `flat_map(|r| r.eyes())`:
    /// this runs at every node of every search, and a `Four` has exactly one
    /// eye, so the flattening was all overhead and no flattening.
    fn take_distinct_two(fours: impl Iterator<Item = Structure>) -> (Option<Point>, Option<Point>) {
        let mut ret = None;
        for four in fours {
            for p in four.eyes() {
                if ret.is_some_and(|e| e != p) {
                    return (ret, Some(p));
                }
                ret = Some(p);
            }
        }
        (ret, None)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Forced(Point),
    Defeated(End),
}

pub use Event::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum End {
    Fours(Point, Point),
    Forbidden(Point),
    Unknown,
}

pub use End::*;

impl fmt::Display for End {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Fours(p1, p2) => format!("Fours({}, {})", p1, p2),
            Forbidden(p) => format!("Forbidden({})", p),
            Unknown => "Unknown".to_string(),
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::{Black, White};

    #[test]
    fn test_zobrist_hash_separates_the_turn_and_the_limit() -> Result<(), String> {
        let board = "H8,J9/I9".parse::<Board>()?;
        let mut game = Game::init(&board, Black);
        let hash = game.zobrist_hash(5);

        // The same stones with the other side to move is another position.
        assert_ne!(Game::init(&board, White).zobrist_hash(5), hash);
        // ... which is exactly what a pass makes, leaving the board alone.
        game.play(None);
        assert_ne!(game.zobrist_hash(5), hash);
        assert_eq!(
            game.zobrist_hash(5),
            Game::init(&board, White).zobrist_hash(5)
        );
        game.undo();
        assert_eq!(game.zobrist_hash(5), hash);

        // The remaining limit still separates otherwise identical positions.
        assert_ne!(game.zobrist_hash(4), hash);

        Ok(())
    }
}
