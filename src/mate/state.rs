use super::super::board::SequenceKind::*;
use super::super::board::*;

#[derive(Clone)]
pub struct GameState {
    board: Board,
    turn: Player,
    last_move: Point,
    last2_move: Point,
}

impl GameState {
    pub fn new(board: Board, turn: Player, last_move: Point, last2_move: Point) -> Self {
        Self {
            board: board,
            turn: turn,
            last_move: last_move,
            last2_move: last2_move,
        }
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

    pub fn won_by_last(&self) -> bool {
        let (may_last_eye, has_another_eye) = self.inspect_last_four_eyes();
        if has_another_eye {
            return true;
        }
        may_last_eye.map_or(false, |e| self.is_forbidden_move(e))
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.turn.is_black() && self.board.forbidden(p).is_some()
    }

    pub fn turn(&self) -> Player {
        self.turn
    }

    pub fn last(&self) -> Player {
        self.turn.opponent()
    }

    pub fn last_move(&self) -> Point {
        self.last_move
    }

    pub fn last2_move(&self) -> Point {
        self.last2_move
    }

    pub fn board_hash(&self) -> u64 {
        self.board.zobrist_hash()
    }

    pub fn next_sequences(
        &self,
        k: SequenceKind,
        n: u8,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        let r = self.turn();
        self.board.sequences(r, k, n, r.is_black())
    }

    pub fn next_sequences_on(
        &self,
        p: Point,
        k: SequenceKind,
        n: u8,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        let r = self.turn();
        self.board.sequences_on(p, r, k, n, r.is_black())
    }

    pub fn last_sequences_on(
        &self,
        p: Point,
        k: SequenceKind,
        n: u8,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        let r = self.last();
        self.board.sequences_on(p, r, k, n, r.is_black())
    }

    pub fn inspect_last_four_eyes(&self) -> (Option<Point>, bool) {
        let last_four_eyes = self
            .last_sequences_on(self.last_move(), Single, 4)
            .map(|(i, s)| i.walk(s.eyes()[0] as i8).to_point());
        let mut ret = None;
        for eye in last_four_eyes {
            if ret.map_or(false, |e| e != eye) {
                return (ret, true);
            }
            ret = Some(eye);
        }
        (ret, false)
    }
}
