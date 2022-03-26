use crate::board::StructureKind::*;
use crate::board::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum End {
    Fours(Point, Point),
    Forbidden(Point),
    Unknown,
}

impl fmt::Display for End {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Fours(p1, p2) => format!("Fours({}, {})", p1, p2),
            Forbidden(p) => format!("Forbidden({})", p),
            Unknown => format!("Unknown"),
        };
        write!(f, "{}", s)
    }
}

pub use End::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Forced(Point),
    Defeated(End),
}

pub use Event::*;

#[derive(Clone)]
pub struct Game {
    pub turn: Player,
    board: Board,
    moves: Vec<Option<Point>>,
}

impl Game {
    pub fn new(board: Board, turn: Player) -> Self {
        Self {
            turn: turn,
            board: board,
            moves: vec![],
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn last_move(&self) -> Option<Point> {
        if self.moves.len() >= 1 {
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

    pub fn zobrist_hash(&self, limit: u8) -> u64 {
        self.board.zobrist_hash_n(limit)
    }

    pub fn play(&mut self, next_move: Option<Point>) {
        if let Some(next_move) = next_move {
            self.board.put_mut(self.turn, next_move);
        }
        self.moves.push(next_move);
        self.turn = self.turn.opponent();
    }

    pub fn undo(&mut self) {
        if let Some(last_move) = self.last_move() {
            self.board.remove_mut(last_move);
        }
        self.moves.pop();
        self.turn = self.turn.opponent();
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.turn.is_black() && self.board.forbidden(p).is_some()
    }

    pub fn check_event(&self) -> Option<Event> {
        let (maybe_first, maybe_another) = self.check_last_four_eyes();
        if maybe_first.is_some() && maybe_another.is_some() {
            let end = Fours(maybe_first.unwrap(), maybe_another.unwrap());
            Some(Defeated(end))
        } else if maybe_first.map_or(false, |e| self.is_forbidden_move(e)) {
            let end = Forbidden(maybe_first.unwrap());
            Some(Defeated(end))
        } else if maybe_first.is_some() {
            let forced_move = maybe_first.unwrap();
            Some(Forced(forced_move))
        } else {
            None
        }
    }

    fn check_last_four_eyes(&self) -> (Option<Point>, Option<Point>) {
        if let Some(last_move) = self.last_move() {
            let last_four_eyes = self
                .board
                .structures_on(last_move, self.turn.opponent(), Four)
                .flat_map(|r| r.eyes());
            Self::take_distinct_two(last_four_eyes)
        } else {
            (None, None)
        }
    }

    fn take_distinct_two(points: impl Iterator<Item = Point>) -> (Option<Point>, Option<Point>) {
        let mut ret = None;
        for p in points {
            if ret.map_or(false, |e| e != p) {
                return (ret, Some(p));
            }
            ret = Some(p);
        }
        (ret, None)
    }
}
