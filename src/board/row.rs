use super::bits::Bits;
use super::player::*;
use super::point::*;
use super::segment::*;

/// A row of one player's stones found on the board: where its segment
/// starts, which of the segment's cells hold the player's stones and which
/// are its eyes, the cells to play to take it a step further.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Row {
    start: Index,
    stones: u8,
    eyes: u8,
}

impl Row {
    /// `r`'s row of kind `k` at `segment`, whose cells start at
    /// `start`.
    pub fn new(start: Index, r: Player, k: RowKind, segment: Segment) -> Self {
        let stones = segment.stone_bits(r);
        Self {
            start,
            stones,
            eyes: !stones & k.eye_cells(),
        }
    }

    pub fn start_index(&self) -> Index {
        self.start
    }

    pub fn start(&self) -> Point {
        self.start.to_point()
    }

    pub fn stones(&self) -> impl Iterator<Item = Point> + use<> {
        self.start.mapped(Bits(self.stones)).map(|i| i.to_point())
    }

    pub fn eyes(&self) -> impl Iterator<Item = Point> + use<> {
        self.start.mapped(Bits(self.eyes)).map(|i| i.to_point())
    }
}

/// The rows of the rules, each told by the scores of one or two
/// neighbouring [`Segment`]s.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RowKind {
    /// Two stones that can become a three: see [`Self::Three`].
    Two,
    /// Three stones that can become a straight four: two neighbouring
    /// segments that both score 3, with the stones all in the four cells
    /// they share. The six cells they span have both ends empty.
    Three,
    /// An open four: as [`Self::Three`], with four stones.
    Straight,
    /// Three stones that can become a four: a segment scoring 3.
    Sword,
    /// A segment scoring 4, one stone short of a five.
    Four,
    /// A segment scoring 5.
    Five,
    /// Five stones in six cells, the empty one making six or more in a row:
    /// two neighbouring segments, both free of the opponent, each with four
    /// stones. An open four is such a pair too, but not through an empty
    /// point.
    Overlining,
    /// Six or more stones in a row: two neighbouring segments full of them.
    Overlined,
}

pub use RowKind::*;

impl RowKind {
    /// How many own stones each segment of the row has.
    #[inline]
    pub fn stones(&self) -> u8 {
        match self {
            Two => 2,
            Three | Sword => 3,
            Straight | Four | Overlining => 4,
            Five | Overlined => 5,
        }
    }

    /// Whether the row needs the segment before `cur` as well, the
    /// `prev` of [`Self::matches`].
    pub fn spans_two(&self) -> bool {
        !matches!(self, Sword | Four | Five)
    }

    /// Whether `r` has this row at segment `cur`; `prev` is the segment
    /// starting one cell before it on the same line, if it is looked at.
    pub fn matches(&self, r: Player, prev: Option<Segment>, cur: Segment) -> bool {
        let n = self.stones();
        match self {
            Sword | Four | Five => cur.score(r) == n as i8,
            // Both score `n`, and `cur`'s last cell is empty: the stones are
            // all in the four cells the two share.
            Two | Three | Straight => {
                cur.score(r) == n as i8
                    && cur.stone_bits(r) & 0b10000 == 0
                    && prev.is_some_and(|prev| prev.score(r) == n as i8)
            }
            // What makes a black segment dead: a black stone next to it.
            Overlining | Overlined => {
                let full = |s: Segment| s.free(r) && s.count(r) == n;
                full(cur) && prev.is_some_and(full)
            }
        }
    }

    /// The cells of the (later) segment that can be the row's eyes, as
    /// a bitmask: all five, but only the four shared ones for the open
    /// rows, whose fifth cell is an open end.
    pub fn eye_cells(&self) -> u8 {
        match self {
            Two | Three | Straight => 0b01111,
            _ => 0b11111,
        }
    }
}
