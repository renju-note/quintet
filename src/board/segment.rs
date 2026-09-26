use super::player::*;
use std::fmt;

/// The number of stones in a row that wins.
pub const VICTORY: u8 = 5;

/// The five cells of a segment, bits 1-5.
const CELLS: u8 = 0b0111110;
/// The cells just before and just after the five, bits 0 and 6.
const MARGINS: u8 = 0b1000001;

/// Five consecutive cells of a line: one place where a five can be made.
///
/// Along with the five cells it holds the cell just before them and the one
/// just after, since whether Black may fill the five in depends on them: a
/// black stone next to the five would make it an overline. The stones are
/// two bitmasks, bit 0 for the cell before, bits 1-5 for cells 0-4 of the
/// segment and bit 6 for the cell after. A cell off the edge of the line is
/// empty.
///
/// Every question about one place for a five is answered here, for either
/// player: can `r` still make a five here ([`Self::alive`]), how close is it
/// ([`Self::score`]), and where are the stones still to be played
/// ([`Self::eyes`]). A [`Line`](super::Line) is the segments along it, and
/// the sequences of the rules ([`SequenceKind`](super::SequenceKind)) are
/// one or two segments with the right scores.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct Segment {
    blacks: u8,
    whites: u8,
}

impl Segment {
    /// A segment from its stones, laid out as described on [`Segment`].
    pub fn new(blacks: u8, whites: u8) -> Self {
        Self { blacks, whites }
    }

    /// Whether `r` can still make a five on the five cells: none of them
    /// holds an opponent stone, and, for Black, filling them in would not
    /// make an overline.
    #[inline]
    pub fn alive(&self, r: Player) -> bool {
        let (my, op) = self.my_op(r);
        op & CELLS == 0 && (r.is_white() || my & MARGINS == 0)
    }

    /// How many of the five cells `r` already has (0 to 5), or `-1` if the
    /// segment is not [`Self::alive`] for `r`.
    #[inline]
    pub fn score(&self, r: Player) -> i8 {
        if self.alive(r) {
            self.count(r) as i8
        } else {
            -1
        }
    }

    /// How many of the five cells hold `r`'s stones, whether or not the
    /// segment is alive.
    #[inline]
    pub fn count(&self, r: Player) -> u8 {
        self.stone_bits(r).count_ones() as u8
    }

    /// Whether none of the five cells holds a stone of `r`'s opponent:
    /// [`Self::alive`] without the overline condition.
    #[inline]
    pub fn free(&self, r: Player) -> bool {
        let (_, op) = self.my_op(r);
        op & CELLS == 0
    }

    /// The cells (0-4) of the five holding `r`'s stones.
    #[inline]
    pub fn stones(&self, r: Player) -> &'static [u8] {
        BITS[self.stone_bits(r) as usize]
    }

    /// The cells (0-4) where `r` still has to play to make a five here, or
    /// none if the segment is not [`Self::alive`] for `r`.
    #[inline]
    pub fn eyes(&self, r: Player) -> &'static [u8] {
        BITS[self.eye_bits(r) as usize]
    }

    /// [`Self::stones`] as a bitmask, bit `k` for cell `k`.
    #[inline]
    pub fn stone_bits(&self, r: Player) -> u8 {
        let (my, _) = self.my_op(r);
        (my & CELLS) >> 1
    }

    /// [`Self::eyes`] as a bitmask, bit `k` for cell `k`.
    #[inline]
    pub fn eye_bits(&self, r: Player) -> u8 {
        if self.alive(r) {
            !self.stone_bits(r) & 0b11111
        } else {
            0
        }
    }

    /// `r`'s stones and the opponent's, in that order.
    #[inline]
    fn my_op(&self, r: Player) -> (u8, u8) {
        match r {
            Black => (self.blacks, self.whites),
            White => (self.whites, self.blacks),
        }
    }
}

impl fmt::Display for Segment {
    /// The seven cells as `o`, `x` and `-`, the margins set apart:
    /// `-|oo-o-|x`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for k in 0..7 {
            if k == 1 || k == 6 {
                f.write_str("|")?;
            }
            let c = if self.blacks & (1 << k) != 0 {
                'o'
            } else if self.whites & (1 << k) != 0 {
                'x'
            } else {
                '-'
            };
            write!(f, "{c}")?;
        }
        Ok(())
    }
}

/// The positions of the set bits of each 5-bit number, lowest first.
pub(super) const BITS: [&[u8]; 32] = [
    &[],
    &[0],
    &[1],
    &[0, 1],
    &[2],
    &[0, 2],
    &[1, 2],
    &[0, 1, 2],
    &[3],
    &[0, 3],
    &[1, 3],
    &[0, 1, 3],
    &[2, 3],
    &[0, 2, 3],
    &[1, 2, 3],
    &[0, 1, 2, 3],
    &[4],
    &[0, 4],
    &[1, 4],
    &[0, 1, 4],
    &[2, 4],
    &[0, 2, 4],
    &[1, 2, 4],
    &[0, 1, 2, 4],
    &[3, 4],
    &[0, 3, 4],
    &[1, 3, 4],
    &[0, 1, 3, 4],
    &[2, 3, 4],
    &[0, 2, 3, 4],
    &[1, 2, 3, 4],
    &[0, 1, 2, 3, 4],
];

#[cfg(test)]
mod tests {
    use super::*;

    /// A segment written like `"-|oo-o-|x"`: the cell before, the five,
    /// the cell after.
    fn segment(s: &str) -> Segment {
        let cells: Vec<char> = s.chars().filter(|&c| c != '|').collect();
        assert_eq!(cells.len(), 7);
        let (mut blacks, mut whites) = (0, 0);
        for (k, c) in cells.into_iter().enumerate() {
            match c {
                'o' => blacks |= 1 << k,
                'x' => whites |= 1 << k,
                _ => {}
            }
        }
        Segment::new(blacks, whites)
    }

    #[test]
    fn test_score_and_eyes() {
        let s = segment("-|o-o--|-");
        assert!(s.free(Black));
        assert!(s.alive(Black));
        assert_eq!(s.score(Black), 2);
        assert_eq!(s.stones(Black), [0, 2]);
        assert_eq!(s.eyes(Black), [1, 3, 4]);
        // Black's stones are in White's way.
        assert!(!s.free(White));
        assert!(!s.alive(White));
        assert_eq!(s.score(White), -1);
        assert_eq!(s.eyes(White), []);

        let s = segment("-|-----|-");
        assert_eq!(s.score(Black), 0);
        assert_eq!(s.score(White), 0);
        assert_eq!(s.eyes(White), [0, 1, 2, 3, 4]);

        let s = segment("-|ooooo|-");
        assert_eq!(s.score(Black), 5);
        assert_eq!(s.eyes(Black), []);
    }

    #[test]
    fn test_margins() {
        // A black stone just outside the five would make filling them in an
        // overline, so the segment is dead for Black, but not for White.
        let s = segment("o|-ooo-|-");
        assert!(s.free(Black));
        assert!(!s.alive(Black));
        assert_eq!(s.score(Black), -1);
        assert_eq!(s.count(Black), 3);
        assert_eq!(s.stones(Black), [1, 2, 3]);
        assert_eq!(s.eyes(Black), []);

        let s = segment("x|-xxx-|x");
        assert!(s.alive(White));
        assert_eq!(s.score(White), 3);
        assert_eq!(s.eyes(White), [0, 4]);
        // White's own stones there are no business of Black's either.
        assert_eq!(segment("x|-----|x").score(Black), 0);
    }

    #[test]
    fn test_display() {
        let s = segment("o|-ox--|x");
        assert_eq!(s.to_string(), "o|-ox--|x");
    }
}
