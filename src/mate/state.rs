use super::super::board::SequenceKind::*;
use super::super::board::*;

#[derive(Clone)]
pub struct GameState {
    board: Board,
    next_player: Player,
    last_move: Point,
}

impl GameState {
    pub fn new(board: Board, next_player: Player, last_move: Point) -> Self {
        Self {
            board: board,
            next_player: next_player,
            last_move: last_move,
        }
    }

    pub fn play_mut(&mut self, next_move: Point) {
        self.board.put_mut(self.next_player, next_move);
        self.next_player = self.last_player();
        self.last_move = next_move;
    }

    pub fn undo_mut(&mut self, last2_move: Point) {
        self.board.remove_mut(self.last_move);
        self.next_player = self.last_player();
        self.last_move = last2_move;
    }

    pub fn next_player(&self) -> Player {
        self.next_player
    }

    pub fn last_move(&self) -> Point {
        self.last_move
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.next_player.is_black() && self.board.forbidden(p).is_some()
    }

    pub fn board_hash(&self) -> u64 {
        self.board.zobrist_hash()
    }

    pub fn sequences(
        &self,
        player: Player,
        kind: SequenceKind,
        n: u8,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        self.board.sequences(player, kind, n, player.is_black())
    }

    pub fn sequences_on(
        &self,
        p: Point,
        player: Player,
        kind: SequenceKind,
        n: u8,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        self.board
            .sequences_on(p, player, kind, n, player.is_black())
    }

    pub fn last_four_eyes(&self) -> impl Iterator<Item = Point> + '_ {
        self.sequences_on(self.last_move(), self.last_player(), Single, 4)
            .map(|(i, s)| i.walk(s.eyes()[0] as i8).to_point())
    }

    pub fn won_by_last(&self) -> bool {
        let mut last_fours = self.sequences_on(self.last_move(), self.last_player(), Single, 4);
        let last_four = last_fours.next();
        if last_four.is_none() {
            return false;
        }
        if last_fours.next().is_some() {
            return true;
        }
        let (index, sequence) = last_four.unwrap();
        let last_four_eye = index.walk(sequence.eyes()[0] as i8).to_point();
        self.is_forbidden_move(last_four_eye)
    }

    fn last_player(&self) -> Player {
        self.next_player.opponent()
    }
}
