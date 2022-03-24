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
    moves: Vec<Point>,
}

impl Game {
    pub fn new(board: Board, turn: Player, moves: Vec<Point>) -> Self {
        Self {
            turn: turn,
            board: board,
            moves: moves,
        }
    }

    pub fn init(board: Board, turn: Player) -> Self {
        let (last_move, last2_move) = Self::choose_last_moves(&board, turn);
        Self::new(board, turn, vec![last2_move, last_move])
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn last_move(&self) -> Point {
        self.moves[self.moves.len() - 1]
    }

    pub fn last2_move(&self) -> Point {
        self.moves[self.moves.len() - 2]
    }

    pub fn zobrist_hash(&self, limit: u8) -> u64 {
        self.board.zobrist_hash_n(limit)
    }

    pub fn play(&mut self, next_move: Point) {
        self.board.put_mut(self.turn, next_move);
        self.turn = self.turn.opponent();
        self.moves.push(next_move);
    }

    pub fn undo(&mut self) {
        self.board.remove_mut(self.last_move());
        self.turn = self.turn.opponent();
        self.moves.pop();
    }

    pub fn pass(&self) -> Self {
        let moves = vec![self.last_move(), self.last2_move()];
        Self::new(self.board.clone(), self.turn.opponent(), moves)
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
        let last_four_eyes = self
            .board
            .structures_on(self.last_move(), self.turn.opponent(), Four)
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
