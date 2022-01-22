use super::fundamentals::*;
use super::point::*;
use super::sequence::SequenceKind::*;
use super::sequence::*;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

use RowKind::*;

impl RowKind {
    pub fn to_sequence(&self, player: Player) -> (SequenceKind, u8, bool) {
        match self {
            Two => (Double, 2, player.is_black()),
            Sword => (Single, 3, player.is_black()),
            Three => (Double, 3, player.is_black()),
            Four => (Single, 4, player.is_black()),
            Five => (Single, 5, player.is_black()),
            Overline => (Double, 5, false),
        }
    }

    pub fn nstones(&self) -> u8 {
        match self {
            Two => 2,
            Sword => 3,
            Three => 3,
            Four => 4,
            Five => 5,
            Overline => 5,
        }
    }

    pub fn potential(&self) -> u8 {
        match self {
            Two => 3,
            Sword => 4,
            Three => 4,
            Four => 5,
            Five => 6,
            Overline => 6,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Row {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eye1: Option<Point>,
    pub eye2: Option<Point>,
}

impl Row {
    pub fn new(
        direction: Direction,
        start: Point,
        end: Point,
        eye1: Option<Point>,
        eye2: Option<Point>,
    ) -> Self {
        Self {
            direction: direction,
            start: start,
            end: end,
            eye1: eye1,
            eye2: eye2,
        }
    }

    pub fn from_sequence(start: Index, sequence: Sequence, kind: RowKind) -> Self {
        let direction = start.direction;

        let row_start = if kind == Overline {
            start.walk(-1).unwrap()
        } else {
            start
        }
        .to_point();
        let row_end = if kind == Two || kind == Three {
            start.walk(3).unwrap()
        } else {
            start.walk(4).unwrap()
        }
        .to_point();

        let mut eyes = start.subsequence(sequence.eyes());
        let row_eye1 = eyes.next().map(|i| i.to_point());
        let row_eye2 = eyes.next().map(|i| i.to_point());

        Self::new(direction, row_start, row_end, row_eye1, row_eye2)
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} start: {} end: {}",
            self.direction, self.start, self.end
        )?;
        for eye1 in self.eye1 {
            write!(f, " eye1: {}", eye1)?;
        }
        for eye2 in self.eye2 {
            write!(f, " eye2: {}", eye2)?;
        }
        Ok(())
    }
}
