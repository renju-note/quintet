use crate::board::RowKind::*;
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

    /// A copy to search from, for a nested search that never undoes past
    /// where it starts: it keeps only the last two moves, all that is read
    /// of the history ([`Self::last_move`], [`Self::last2_move`]), and room
    /// for `plies` more. A plain `clone` copied the whole history, and the
    /// first move played then had to grow it.
    pub fn fork(&self, plies: usize) -> Self {
        let kept = &self.moves[self.moves.len().saturating_sub(2)..];
        let mut moves = Vec::with_capacity(kept.len() + plies);
        moves.extend_from_slice(kept);
        Self {
            board: self.board.clone(),
            moves,
            turn: self.turn,
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

    /// The last move, `None` if it was a pass or there is none.
    pub fn last_move(&self) -> Option<Point> {
        self.moves.last().copied().flatten()
    }

    /// The move before the last, as [`Self::last_move`].
    pub fn last2_move(&self) -> Option<Point> {
        self.moves.iter().rev().nth(1).copied().flatten()
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
            Self::take_distinct_two(self.four_eyes_on(last_move, opponent))
        } else {
            Self::take_distinct_two(self.board.rows(opponent, Four).flat_map(|r| r.eyes()))
        }
    }

    /// The eyes of `r`'s fours through `p`, line by line: those of
    /// `rows_on(p, r, Four)`, read off each line's bitmasks without building
    /// the rows. This runs at every node of every search.
    ///
    /// On one line, two fours through `p` give their eyes in the order the
    /// rows come, as the rows start: were the later row's eye before the
    /// earlier one's, both would be empty cells of the earlier row, which
    /// then would have two and be no four.
    fn four_eyes_on(&self, p: Point, r: Player) -> impl Iterator<Item = Point> + '_ {
        self.board
            .lines_on(p)
            .filter(move |(_, _, l)| l.potential_cap(r) > Four.stones())
            .flat_map(move |(d, i, l)| {
                let starts = l.row_starts_on(p.to_index(d).j, r, Four);
                Bits(l.eyes_of(starts, Four.eye_cells()))
                    .map(move |j| Index::new(d, i, j).to_point())
            })
    }

    /// The first two distinct points of `eyes`, in the order they come.
    fn take_distinct_two(eyes: impl Iterator<Item = Point>) -> (Option<Point>, Option<Point>) {
        let mut ret = None;
        for p in eyes {
            if ret.is_some_and(|e| e != p) {
                return (ret, Some(p));
            }
            ret = Some(p);
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
    fn test_position_hash_separates_the_turn() -> Result<(), String> {
        let board = "H8,J9/I9".parse::<Board>()?;
        let mut game = Game::init(&board, Black);
        let hash = game.position_hash();

        // The same stones with the other side to move is another position.
        assert_ne!(Game::init(&board, White).position_hash(), hash);
        // ... which is exactly what a pass makes, leaving the board alone.
        game.play(None);
        assert_ne!(game.position_hash(), hash);
        assert_eq!(
            game.position_hash(),
            Game::init(&board, White).position_hash()
        );
        game.undo();
        assert_eq!(game.position_hash(), hash);

        Ok(())
    }

    #[test]
    fn test_last_moves() {
        let mut game = Game::init(&Board::new(), Black);
        assert_eq!((game.last_move(), game.last2_move()), (None, None));
        game.play(Some(point("H8")));
        assert_eq!(
            (game.last_move(), game.last2_move()),
            (Some(point("H8")), None)
        );
        game.play(None);
        assert_eq!(
            (game.last_move(), game.last2_move()),
            (None, Some(point("H8")))
        );
        game.play(Some(point("I9")));
        assert_eq!(
            (game.last_move(), game.last2_move()),
            (Some(point("I9")), None)
        );
    }

    /// A fork keeps what is read of the history and the position itself.
    #[test]
    fn test_fork_keeps_the_last_moves() {
        let mut game = Game::init(&Board::new(), Black);
        for m in ["H8", "I9", "J10"] {
            game.play(Some(point(m)));
        }
        let mut fork = game.fork(4);
        assert_eq!(fork.last_move(), game.last_move());
        assert_eq!(fork.last2_move(), game.last2_move());
        assert_eq!(fork.position_hash(), game.position_hash());

        // Moves played on the fork can be undone, down to where it began.
        fork.play(None);
        fork.play(Some(point("K11")));
        assert_eq!(fork.last2_move(), None);
        fork.undo();
        fork.undo();
        assert_eq!(fork.last_move(), game.last_move());
        assert_eq!(fork.position_hash(), game.position_hash());
    }

    fn point(s: &str) -> Point {
        s.parse().unwrap()
    }

    /// The event after `turn` plays `m` on `board`.
    fn event_after(board: &str, turn: Player, m: &str) -> Option<Event> {
        let mut game = Game::init(&board.parse::<Board>().unwrap(), turn);
        game.play(Some(point(m)));
        game.check_event()
    }

    #[test]
    fn test_check_event() {
        // A four with one way to five: the reply is forced.
        let event = event_after("H8,I8,J8/G8", Black, "K8");
        assert_eq!(event, Some(Forced(point("L8"))));

        // Two ways: the defender cannot stop both.
        let event = event_after("H8,I8,J8/A1", Black, "K8");
        assert_eq!(event, Some(Defeated(Fours(point("G8"), point("L8")))));

        // Two fours, D8-H8 and E8-I8, but one eye G8 blocks both.
        let event = event_after("A1/E8,F8,H8,I8", White, "D8");
        assert_eq!(event, Some(Forced(point("G8"))));

        // The only block, H8, is a double-three, so Black may not play it.
        let event = event_after("H9,G8,I8,H7,C3/E5,F6,G7", White, "D4");
        assert_eq!(event, Some(Defeated(Forbidden(point("H8")))));

        // No four, nothing forced.
        assert_eq!(event_after("H8,I8/A1", Black, "J8"), None);
    }

    #[test]
    fn test_check_event_without_a_last_move() -> Result<(), String> {
        // At the root there is no last move to look around, so every four
        // of the side that just moved counts.
        let board = "H8,I8,J8,K8/G8".parse::<Board>()?;
        let mut game = Game::init(&board, White);
        assert_eq!(game.check_event(), Some(Forced(point("L8"))));

        // The same after a pass, which leaves no last move either.
        game.play(Some(point("A1")));
        game.play(None);
        assert_eq!(game.check_event(), Some(Forced(point("L8"))));
        Ok(())
    }
}
