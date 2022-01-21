use std::convert::From;
use std::convert::TryFrom;

pub const BOARD_SIZE: u8 = 15;

pub type Bits = u16;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Player {
    Black,
    White,
}

use Player::*;

impl Player {
    pub fn opponent(&self) -> Self {
        match self {
            Black => White,
            White => Black,
        }
    }

    pub fn is_black(&self) -> bool {
        *self == Black
    }

    pub fn is_white(&self) -> bool {
        *self == White
    }
}

impl From<bool> for Player {
    fn from(value: bool) -> Self {
        if value {
            Black
        } else {
            White
        }
    }
}

impl From<Player> for bool {
    fn from(value: Player) -> Self {
        value.is_black()
    }
}

impl TryFrom<char> for Player {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'o' => Ok(Black),
            'x' => Ok(White),
            _ => Err("Invalid player"),
        }
    }
}

impl From<Player> for char {
    fn from(value: Player) -> Self {
        match value {
            Black => 'o',
            White => 'x',
        }
    }
}
