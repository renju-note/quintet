use super::super::board::*;

#[derive(Clone)]
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

    pub fn last_move(&self) -> Option<Point> {
        self.last_move
    }

    pub fn is_legal_move(&self, p: Point) -> bool {
        self.board.stone(p).is_none()
            && !(self.next_player.is_black() && self.board.forbidden(p).is_some())
    }

    pub fn board_hash(&self) -> u64 {
        self.board.zobrist_hash()
    }
}
