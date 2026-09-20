use super::player::*;
use super::point::*;
use super::sequence::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum StructureKind {
    Two,
    Three,
    OpenFour,
    Sword,
    Four,
    Five,
    NextOverFive,
    OverFive,
}

pub use StructureKind::*;

impl StructureKind {
    pub fn to_sequence(&self, r: Player) -> (SequenceKind, u8, bool) {
        // Black's five must be exactly five, so a window next to an own stone is
        // excluded (`strict`). Overline kinds look for exactly such windows, so
        // they are never strict.
        let strict = r.is_black();
        match self {
            Two => (Open, 2, strict),
            Three => (Open, 3, strict),
            OpenFour => (Open, 4, strict),
            Sword => (Single, 3, strict),
            Four => (Single, 4, strict),
            Five => (Single, 5, strict),
            NextOverFive => (Double, 4, false),
            OverFive => (Double, 5, false),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Structure {
    start: Index,
    sequence: Sequence,
}

impl Structure {
    pub fn new(start: Index, sequence: Sequence) -> Self {
        Self { start, sequence }
    }

    pub fn start_index(&self) -> Index {
        self.start
    }

    pub fn start(&self) -> Point {
        self.start.to_point()
    }

    pub fn stones(&self) -> impl Iterator<Item = Point> {
        self.start
            .mapped(self.sequence.stones())
            .map(|i| i.to_point())
    }

    pub fn eyes(&self) -> impl Iterator<Item = Point> {
        self.start
            .mapped(self.sequence.eyes())
            .map(|i| i.to_point())
    }
}
