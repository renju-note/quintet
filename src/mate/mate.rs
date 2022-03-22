use super::game::*;
use crate::board::*;

#[derive(Debug, PartialEq, Eq)]
pub struct Mate {
    pub end: End,
    pub path: Vec<Point>,
}

impl Mate {
    pub fn new(end: End, path: Vec<Point>) -> Self {
        Self {
            end: end,
            path: path,
        }
    }

    pub fn unshift(mut self, m: Point) -> Self {
        let end = self.end;
        let mut path = vec![m];
        path.append(&mut self.path);
        Self::new(end, path)
    }

    pub fn preferred(old: Self, new: Self) -> Self {
        if old.end == Unknown || new.path.len() > old.path.len() {
            new
        } else {
            old
        }
    }

    pub fn n_moves(&self) -> u8 {
        self.path.len() as u8
    }

    pub fn n_times(&self) -> u8 {
        ((self.path.len() + 1) / 2) as u8
    }
}
