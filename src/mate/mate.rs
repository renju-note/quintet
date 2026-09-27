use super::game::*;
use crate::board::*;

/// A winning line: the moves from the root, attacker first, and how it
/// ends.
#[derive(Debug, PartialEq, Eq)]
pub struct Mate {
    pub end: End,
    pub path: Vec<Point>,
}

impl Mate {
    pub fn new(end: End, path: Vec<Point>) -> Self {
        Self { end, path }
    }

    /// The line with `m` played before it.
    pub fn prepend(mut self, m: Point) -> Self {
        self.path.insert(0, m);
        self
    }

    pub fn n_moves(&self) -> u8 {
        self.path.len() as u8
    }

    pub fn n_attacks(&self) -> u8 {
        self.path.len().div_ceil(2) as u8
    }
}
