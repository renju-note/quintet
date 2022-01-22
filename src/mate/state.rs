use super::super::board::*;

#[derive(Clone)]
pub struct GameState {
    board: Board,
    next_player: Player,
    last_move: Point,
}

impl GameState {
    pub fn new(board: &Board, next_player: Player, last_move: Point) -> Self {
        Self {
            board: board.clone(),
            next_player: next_player,
            last_move: last_move,
        }
    }

    pub fn play_mut(&mut self, next_move: Point) {
        self.board.put_mut(self.next_player, next_move);
        self.next_player = self.next_player().opponent();
        self.last_move = next_move;
    }

    pub fn play(&self, next_move: Point) -> Self {
        let mut result = self.clone();
        result.play_mut(next_move);
        result
    }

    pub fn next_player(&self) -> Player {
        self.next_player
    }

    pub fn last_player(&self) -> Player {
        self.next_player.opponent()
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
        self.board.sequences(player, kind, n)
    }

    pub fn sequences_on(
        &self,
        player: Player,
        kind: SequenceKind,
        n: u8,
        p: Point,
    ) -> impl Iterator<Item = (Index, Sequence)> + '_ {
        self.board.sequences_on(player, kind, n, p)
    }
}
