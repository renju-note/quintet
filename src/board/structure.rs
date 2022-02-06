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
        match self {
            Two => (Compact, 2, r.is_black()),
            Three => (Compact, 3, r.is_black()),
            OpenFour => (Compact, 4, r.is_black()),
            Sword => (Single, 3, r.is_black()),
            Four => (Single, 4, r.is_black()),
            Five => (Single, 5, r.is_black()),
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
        Self {
            start: start,
            sequence: sequence,
        }
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
