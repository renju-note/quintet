use super::super::board::*;

pub struct GameState {
    pub board: Board,
    next_player: Player,
    last_move: Option<Point>,
}

impl GameState {
    pub fn from_board(board: &Board, next_player: Player) -> Self {
        Self {
            board: board.clone(),
            next_player: next_player,
            last_move: None,
        }
    }

    pub fn play(&self, next_move: Point) -> Self {
        Self {
            board: self.board.put(self.next_player, next_move),
            next_player: self.next_player.opponent(),
            last_move: Some(next_move),
        }
    }

    pub fn next_player(&self) -> Player {
        self.next_player
    }

    pub fn last_player(&self) -> Player {
        self.next_player.opponent()
    }

    pub fn is_legal_move(&self, p: Point) -> bool {
        self.board.stone(p).is_none()
            && !(self.next_player.is_black() && self.board.forbidden(p).is_some())
    }

    pub fn latest_row_eyes(&self, kind: RowKind) -> Vec<Point> {
        match self.last_move {
            Some(p) => self.board.row_eyes_along(self.last_player(), kind, p),
            None => self.board.row_eyes(self.last_player(), kind),
        }
    }

    pub fn board_hash(&self) -> u64 {
        self.board.zobrist_hash()
    }
}
