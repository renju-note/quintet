use crate::board::StructureKind::*;
use crate::board::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Win {
    Fours(Point, Point),
    Forbidden(Point),
    Unknown,
}

impl fmt::Display for Win {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Fours(p1, p2) => format!("Fours({}, {})", p1, p2),
            Forbidden(p) => format!("Forbidden({})", p),
            Unknown => format!("Unknown"),
        };
        write!(f, "{}", s)
    }
}

pub use Win::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Stage {
    Forced(Point),
    End(Win),
}

pub use Stage::*;

#[derive(Clone)]
pub struct Game {
    board: Board,
    turn: Player,
    last_move: Point,
    last2_move: Point,
}

impl Game {
    pub fn new(board: Board, turn: Player, last_move: Point, last2_move: Point) -> Self {
        Self {
            board: board,
            turn: turn,
            last_move: last_move,
            last2_move: last2_move,
        }
    }

    pub fn init(board: Board, turn: Player) -> Self {
        let (last_move, last2_move) = Self::choose_last_moves(&board, turn);
        Self::new(board, turn, last_move, last2_move)
    }

    pub fn board(&self) -> &'_ Board {
        &self.board
    }

    pub fn turn(&self) -> Player {
        self.turn
    }

    pub fn last_move(&self) -> Point {
        self.last_move
    }

    pub fn last2_move(&self) -> Point {
        self.last2_move
    }

    pub fn last(&self) -> Player {
        self.turn.opponent()
    }

    pub fn zobrist_hash(&self, limit: u8) -> u64 {
        self.board.zobrist_hash_n(limit)
    }

    pub fn play(&mut self, next_move: Point) {
        self.board.put_mut(self.turn, next_move);
        self.turn = self.last();
        self.last2_move = self.last_move;
        self.last_move = next_move;
    }

    pub fn undo(&mut self, last2_move: Point) {
        self.board.remove_mut(self.last_move);
        self.turn = self.last();
        self.last_move = self.last2_move;
        self.last2_move = last2_move;
    }

    pub fn pass(&self) -> Self {
        let last2_move = self.last_move;
        let last_move = self.last2_move;
        Self::new(self.board.clone(), self.last(), last_move, last2_move)
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.turn.is_black() && self.board.forbidden(p).is_some()
    }

    pub fn check_stage(&self) -> Option<Stage> {
        let (maybe_first, maybe_another) = self.check_last_four_eyes();
        if maybe_first.is_some() && maybe_another.is_some() {
            let win = Fours(maybe_first.unwrap(), maybe_another.unwrap());
            Some(End(win))
        } else if maybe_first.map_or(false, |e| self.is_forbidden_move(e)) {
            let win = Forbidden(maybe_first.unwrap());
            Some(End(win))
        } else if maybe_first.is_some() {
            let forced_move = maybe_first.unwrap();
            Some(Forced(forced_move))
        } else {
            None
        }
    }

    fn check_last_four_eyes(&self) -> (Option<Point>, Option<Point>) {
        let last_four_eyes = self
            .board
            .structures_on(self.last_move(), self.last(), Four)
            .flat_map(|r| r.eyes());
        let mut ret = None;
        for eye in last_four_eyes {
            if ret.map_or(false, |e| e != eye) {
                return (ret, Some(eye));
            }
            ret = Some(eye);
        }
        (ret, None)
    }

    fn choose_last_moves(board: &Board, turn: Player) -> (Point, Point) {
        let last = turn.opponent();
        let mut last_fours = board.structures(last, Four);
        let last_move = if let Some(four) = last_fours.next() {
            let stone = four.stones().next().unwrap();
            board.stones(last).find(|&s| s == stone)
        } else {
            board.stones(last).next()
        };
        let last2_move = board.stones(turn).next();
        let default = Point(0, 0);
        (last_move.unwrap_or(default), last2_move.unwrap_or(default))
    }
}
