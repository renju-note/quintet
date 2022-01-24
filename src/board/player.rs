use std::convert::{From, TryFrom};
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Player {
    Black,
    White,
}

pub use Player::*;

impl Player {
    pub fn opponent(&self) -> Self {
        if self.is_black() {
            White
        } else {
            Black
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

impl FromStr for Player {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.trim().chars().next().ok_or("empty")?;
        Self::try_from(c)
    }
}
