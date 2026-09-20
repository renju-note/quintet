use crate::board::StructureKind::*;
use crate::board::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
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
            Unknown => "Unknown".to_string(),
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

    pub fn zobrist_hash(&self, n: u8) -> u64 {
        self.board.zobrist_hash_n(n)
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

    fn check_last_four_eyes(&self) -> (Option<Point>, Option<Point>) {
        if let Some(last_move) = self.last_move() {
            let last_four_eyes = self
                .board
                .structures_on(last_move, self.turn.opponent(), Four)
                .flat_map(|r| r.eyes());
            Self::take_distinct_two(last_four_eyes)
        } else {
            let four_eyes = self
                .board
                .structures(self.turn.opponent(), Four)
                .flat_map(|r| r.eyes());
            Self::take_distinct_two(four_eyes)
        }
    }

    fn take_distinct_two(points: impl Iterator<Item = Point>) -> (Option<Point>, Option<Point>) {
        let mut ret = None;
        for p in points {
            if ret.is_some_and(|e| e != p) {
                return (ret, Some(p));
            }
            ret = Some(p);
        }
        (ret, None)
    }

    #[allow(dead_code)]
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
}
