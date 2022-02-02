use super::super::board::SequenceKind::*;
use super::super::board::*;

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

    pub fn play_mut(&mut self, next_move: Point) {
        self.board.put_mut(self.turn, next_move);
        self.turn = self.last();
        self.last2_move = self.last_move();
        self.last_move = next_move;
    }

    pub fn undo_mut(&mut self, last2_move: Point) {
        self.board.remove_mut(self.last_move);
        self.turn = self.last();
        self.last_move = self.last2_move();
        self.last2_move = last2_move;
    }

    pub fn pass(&self) -> Self {
        let last2_move = self.last_move();
        let last_move = self.board.stones(self.turn()).next().unwrap();
        Self::new(self.board.clone(), self.last(), last_move, last2_move)
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.turn.is_black() && self.board.forbidden(p).is_some()
    }

    pub fn inspect_last_win_or_abs(&self) -> Option<Result<Win, Point>> {
        let (may_first_eye, may_another_eye) = self.inspect_last_four_eyes();
        if may_first_eye.is_some() && may_another_eye.is_some() {
            let win = Win::Fours(may_first_eye.unwrap(), may_another_eye.unwrap());
            Some(Ok(win))
        } else if may_first_eye.map_or(false, |e| self.is_forbidden_move(e)) {
            let win = Win::Forbidden(may_first_eye.unwrap());
            Some(Ok(win))
        } else if may_first_eye.is_some() {
            Some(Err(may_first_eye.unwrap()))
        } else {
            None
        }
    }

    pub fn inspect_last_four_eyes(&self) -> (Option<Point>, Option<Point>) {
        let last = self.last();
        let last_four_eyes = self
            .board
            .sequences_on(self.last_move(), last, Single, 4, last.is_black())
            .map(|(i, s)| i.walk(s.eyes()[0]).to_point());
        let mut ret = None;
        for eye in last_four_eyes {
            if ret.map_or(false, |e| e != eye) {
                return (ret, Some(eye));
            }
            ret = Some(eye);
        }
        (ret, None)
    }

    pub fn won_by_last(&self) -> Option<Win> {
        let (may_first_eye, may_another_eye) = self.inspect_last_four_eyes();
        if may_first_eye.is_some() && may_another_eye.is_some() {
            Some(Win::Fours(may_first_eye.unwrap(), may_another_eye.unwrap()))
        } else if may_first_eye.map_or(false, |e| self.is_forbidden_move(e)) {
            Some(Win::Forbidden(may_first_eye.unwrap()))
        } else {
            None
        }
    }

    fn choose_last_moves(board: &Board, turn: Player) -> (Point, Point) {
        let last = turn.opponent();
        let mut last_fours = board.sequences(last, Single, 4, last.is_black());
        let last_move = if let Some((index, _)) = last_fours.next() {
            let start = index.to_point();
            let next = index.walk(1).to_point();
            board.stones(last).find(|&s| s == start || s == next)
        } else {
            board.stones(last).next()
        };
        let last2_move = board.stones(turn).next();
        let default = Point(0, 0);
        (last_move.unwrap_or(default), last2_move.unwrap_or(default))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Win {
    Fours(Point, Point),
    Forbidden(Point),
    Unknown(),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Mate {
    pub win: Win,
    pub path: Vec<Point>,
}

impl Mate {
    pub fn new(win: Win, path: Vec<Point>) -> Self {
        Self {
            win: win,
            path: path,
        }
    }

    pub fn prepend(mut self, m: Point) -> Self {
        let win = self.win;
        let mut path = vec![m];
        path.append(&mut self.path);
        Self::new(win, path)
    }
}
