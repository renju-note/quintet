use std::convert::From;

pub const BOARD_SIZE: u8 = 15;

pub type Bits = u16;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Player {
    Black,
    White,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    pub fn is_black(self) -> bool {
        self == Player::Black
    }

    pub fn is_white(self) -> bool {
        self == Player::White
    }
}

impl From<bool> for Player {
    fn from(value: bool) -> Player {
        if value {
            Player::Black
        } else {
            Player::White
        }
    }
}
