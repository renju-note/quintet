use super::player::*;
use super::segment::*;
use super::sequence::SequenceKind::{self, *};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

const MAX_SIZE: u8 = 16 - 1;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Line {
    blacks: u16,
    whites: u16,
    pub size: u8,
}

impl Line {
    pub fn new(size: u8) -> Self {
        Self {
            blacks: 0b0,
            whites: 0b0,
            size,
        }
    }

    pub fn put_mut(&mut self, r: Player, i: u8) {
        let stones = 0b1 << i;
        let (blacks, whites) = match r {
            Black => (self.blacks | stones, self.whites & !stones),
            White => (self.blacks & !stones, self.whites | stones),
        };
        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn remove_mut(&mut self, i: u8) {
        let stones = 0b1 << i;
        self.blacks &= !stones;
        self.whites &= !stones;
    }

    pub fn stone(&self, i: u8) -> Option<Player> {
        let pat = 0b1 << i;
        if self.blacks & pat != 0b0 {
            Some(Black)
        } else if self.whites & pat != 0b0 {
            Some(White)
        } else {
            None
        }
    }

    pub fn stones(&self, r: Player) -> impl Iterator<Item = u8> {
        let target = match r {
            Black => self.blacks,
            White => self.whites,
        };
        (0..self.size).filter(move |i| target & (0b1 << i) != 0b0)
    }

    pub fn empties(&self) -> impl Iterator<Item = u8> {
        let blacks = self.blacks;
        let whites = self.whites;
        (0..self.size).filter(move |i| blacks & 0b1 << i == 0b0 && whites & 0b1 << i == 0b0)
    }

    /// The segment whose five cells start at cell `j`.
    #[inline]
    pub fn segment(&self, j: u8) -> Segment {
        // Shifted up one so that bit 0 is the cell before, cell -1 at `j = 0`.
        let cells = |stones: u16| ((stones << 1) >> j) as u8 & 0b1111111;
        Segment::new(cells(self.blacks), cells(self.whites))
    }

    /// Every segment of the line, as `(j, segment)`, `j` being where its
    /// five cells start.
    pub fn segments(&self) -> impl Iterator<Item = (u8, Segment)> + '_ {
        (0..=self.size - VICTORY).map(|j| (j, self.segment(j)))
    }

    /// `r`'s sequences of kind `k` in the line, as `(j, segment)`, `j`
    /// being where the segment starts: the segments where
    /// [`SequenceKind::matches`] holds, the one before each as its `prev`.
    pub fn sequences(&self, r: Player, k: SequenceKind) -> Sequences {
        Sequences {
            line: *self,
            starts: self.sequence_starts(r, k),
        }
    }

    /// [`Self::sequences`], only those through cell `i`. For a pattern of
    /// two segments, both have to be through it.
    pub fn sequences_on(&self, i: u8, r: Player, k: SequenceKind) -> Sequences {
        let first = i.saturating_sub(if k.spans_two() { 3 } else { 4 });
        let through = ((1u32 << (i + 1)) - (1u32 << first)) as u16;
        Sequences {
            line: *self,
            starts: self.sequence_starts(r, k) & through,
        }
    }

    /// The potential of each empty cell for `r`, as `(i, potential)`, the
    /// cells below `min` left out.
    ///
    /// A segment is worth its [`Segment::score`] + 1, so 0 if it is dead,
    /// and nothing below `min`. A cell is worth the best segment through
    /// it, times how many segments through it are that good.
    pub fn potentials(&self, r: Player, min: u8) -> impl Iterator<Item = (u8, u8)> + '_ {
        let mut values = [0u8; MAX_SIZE as usize];
        for (j, s) in self.segments() {
            let v = (s.score(r) + 1) as u8;
            values[j as usize] = if v >= min { v } else { 0 };
        }
        self.empties().filter_map(move |i| {
            let first = i.saturating_sub(VICTORY - 1);
            let last = i.min(self.size - VICTORY);
            let through = &values[first as usize..=last as usize];
            let max = through.iter().copied().max().unwrap_or(0);
            let potential = max * through.iter().filter(|&&v| v == max).count() as u8;
            (potential >= min).then_some((i, potential))
        })
    }

    pub fn potential_cap(&self, r: Player) -> u8 {
        let (my, op) = self.my_op(r);
        if self.size < op.count_ones() as u8 + 5 {
            return 0;
        }
        my.count_ones() as u8 + 1
    }

    /// Bit `j` is set if `r` has a sequence of kind `k` at the segment
    /// starting at cell `j`: the starts [`Self::sequences`] gives.
    #[inline]
    pub fn sequence_starts(&self, r: Player, k: SequenceKind) -> u16 {
        let n = k.stones();
        match k {
            Sword | Four | Five => self.scoring(r, n),
            Two | Three | Straight => {
                let (my, _) = self.my_op(r);
                let s = self.scoring(r, n);
                s & s << 1 & !(my >> 4)
            }
            Overlining | Overlined => {
                let s = self.counting(r, n);
                s & s << 1
            }
        }
    }

    /// Bit `j` is set if the segment starting at cell `j` scores `n` for
    /// `r` ([`Segment::score`]), for all the segments at once.
    #[inline]
    pub fn scoring(&self, r: Player, n: u8) -> u16 {
        let (my, _) = self.my_op(r);
        // A black stone just before or just after the five.
        let overline = if r.is_black() { my << 1 | my >> 5 } else { 0 };
        self.counting(r, n) & !overline
    }

    /// Bit `j` is set if the segment starting at cell `j` is free for `r`
    /// and has `n` of its stones ([`Segment::free`], [`Segment::count`]).
    ///
    /// Bit `j` of `x >> k` is cell `j + k`, so each condition is checked
    /// for every segment in parallel: no opponent stone in the five cells,
    /// and the five added up bit-sliced, a full adder on the first three,
    /// a half adder on the last two, then the carries.
    #[inline]
    pub fn counting(&self, r: Player, n: u8) -> u16 {
        let (my, op) = self.my_op(r);
        let segments = (1u16 << (self.size + 1 - VICTORY)) - 1;
        let blocked = op | op >> 1 | op >> 2 | op >> 3 | op >> 4;
        let (a, b, c, d, e) = (my, my >> 1, my >> 2, my >> 3, my >> 4);
        let (s1, c1) = (a ^ b ^ c, a & b | c & (a ^ b));
        let (s2, c2) = (d ^ e, d & e);
        let (ones, c3) = (s1 ^ s2, s1 & s2);
        let twos = c1 ^ c2 ^ c3;
        let fours = c1 & c2 | c3 & (c1 ^ c2);
        let digit = |bits: u16, k: u8| if n & k != 0 { bits } else { !bits };
        digit(ones, 1) & digit(twos, 2) & digit(fours, 4) & !blocked & segments
    }

    /// `r`'s stones and the opponent's, in that order.
    fn my_op(&self, r: Player) -> (u16, u16) {
        match r {
            Black => (self.blacks, self.whites),
            White => (self.whites, self.blacks),
        }
    }
}

/// The segments of a line where one player has a sequence of one kind:
/// [`Line::sequences`].
pub struct Sequences {
    line: Line,
    /// Where the segments not given yet start, bit `j` for cell `j`.
    starts: u16,
}

impl Iterator for Sequences {
    type Item = (u8, Segment);

    fn next(&mut self) -> Option<Self::Item> {
        if self.starts == 0 {
            return None;
        }
        let j = self.starts.trailing_zeros() as u8;
        self.starts &= self.starts - 1;
        Some((j, self.line.segment(j)))
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ss = (0..self.size).map(|i| match self.stone(i) {
            Some(player) => format!(" {}", char::from(player)),
            None => " .".to_string(),
        });
        f.write_str(&ss.collect::<String>())
    }
}

impl FromStr for Line {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        let size = chars.len();
        if size > MAX_SIZE as usize {
            return Err("Wrong length.");
        }
        let mut line = Self::new(size as u8);
        for (i, c) in chars.into_iter().enumerate() {
            if let Ok(p) = Player::try_from(c) {
                line.put_mut(p, i as u8);
            }
        }
        Ok(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KINDS: [SequenceKind; 8] = [
        Two, Three, Straight, Sword, Four, Five, Overlining, Overlined,
    ];

    /// What `sequences` is defined as: the segments, one at a time, where
    /// `SequenceKind::matches` holds.
    fn sequence_starts_by_segments(line: &Line, r: Player, k: SequenceKind) -> u16 {
        let mut starts = 0;
        let mut prev = None;
        for (j, cur) in line.segments() {
            if k.matches(r, prev, cur) {
                starts |= 1 << j;
            }
            prev = Some(cur);
        }
        starts
    }

    /// Where `sequences` / `sequences_on` start, as a list.
    fn starts(found: Sequences) -> Vec<u8> {
        found.map(|(j, _)| j).collect()
    }

    #[test]
    fn test_sequence_starts_matches_segments() {
        let check = |size: u8, cells: &[u8]| {
            let mut line = Line::new(size);
            for (i, &c) in cells.iter().enumerate() {
                match c {
                    1 => line.put_mut(Black, i as u8),
                    2 => line.put_mut(White, i as u8),
                    _ => {}
                }
            }
            for r in [Black, White] {
                for k in KINDS {
                    let expected = sequence_starts_by_segments(&line, r, k);
                    assert_eq!(line.sequence_starts(r, k), expected, "{r:?} {k:?} {line}");
                    // Through cell `i`: the segment, and for a pattern of
                    // two its predecessor too, has `i` among its cells.
                    for i in 0..size {
                        let first = i.saturating_sub(if k.spans_two() { 3 } else { 4 });
                        let on: Vec<_> = (first..=i).filter(|j| expected & 1 << j != 0).collect();
                        assert_eq!(
                            starts(line.sequences_on(i, r, k)),
                            on,
                            "{r:?} {k:?} {line} on {i}"
                        );
                    }
                }
            }
        };
        // Every line up to nine cells, which covers the short diagonals and
        // every pair of segments with their margins.
        for size in 5..=9u8 {
            for code in 0..3u32.pow(size as u32) {
                let cells: Vec<u8> = (0..size)
                    .map(|i| (code / 3u32.pow(i as u32) % 3) as u8)
                    .collect();
                check(size, &cells);
            }
        }
        // Full-length lines, pseudo-randomly.
        let mut x: u64 = 0x2545f4914f6cdd1d;
        for _ in 0..20_000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let cells: Vec<u8> = (0..15).map(|i| ((x >> (2 * i)) % 3) as u8).collect();
            check(15, &cells);
        }
    }

    #[test]
    fn test_segment() -> Result<(), String> {
        let line = "o-ox---xo".parse::<Line>()?;
        // Cells off the line are empty.
        assert_eq!(line.segment(0).to_string(), "-|o-ox-|-");
        assert_eq!(line.segment(1).to_string(), "o|-ox--|-");
        assert_eq!(line.segment(4).to_string(), "x|---xo|-");
        assert_eq!(line.segments().count(), 5);
        Ok(())
    }

    #[test]
    fn test_sequences() -> Result<(), String> {
        // Three stones in a segment free of the opponent. For Black, not
        // next to another black stone: segments 1 and 2 would make an
        // overline.
        let line = "-o-oo-o--------".parse::<Line>()?;
        assert_eq!(starts(line.sequences(Black, Sword)), [0, 3]);
        let line = "-x-xx-x--------".parse::<Line>()?;
        assert_eq!(starts(line.sequences(White, Sword)), [0, 1, 2, 3]);
        assert_eq!(starts(line.sequences_on(6, White, Sword)), [2, 3]);

        // A three: two segments both scoring 3, with the stones in the four
        // cells they share, reported at the later one. Here cells 4-9.
        let line = "-----xx-x-x----".parse::<Line>()?;
        assert_eq!(starts(line.sequences(White, Three)), [5]);
        assert_eq!(starts(line.sequences_on(7, White, Three)), [5]);
        // ... which for Black would make an overline with cell 10.
        let line = "-----oo-o-o----".parse::<Line>()?;
        assert_eq!(starts(line.sequences(Black, Three)), []);
        // Swords on each side of cell 7, but no three.
        let line = "---ooo---ooo---".parse::<Line>()?;
        assert_eq!(starts(line.sequences_on(7, Black, Sword)), [3, 7]);
        assert_eq!(starts(line.sequences_on(7, Black, Three)), []);

        // Five stones in six cells: the eye makes an overline.
        let line = "oo-ooo---------".parse::<Line>()?;
        assert_eq!(starts(line.sequences(Black, Overlining)), [1]);
        assert_eq!(starts(line.sequences(Black, Four)), []);
        // An open four has two such segments too, but not both through an
        // end, which is the only place left to play.
        let line = "-oooo-----o----".parse::<Line>()?;
        assert_eq!(starts(line.sequences(Black, Overlining)), [1]);
        assert_eq!(starts(line.sequences_on(0, Black, Overlining)), []);
        assert_eq!(starts(line.sequences_on(5, Black, Overlining)), []);
        assert_eq!(starts(line.sequences(Black, Straight)), [1]);

        let line = "-oooooo--------".parse::<Line>()?;
        assert_eq!(starts(line.sequences(Black, Overlined)), [2]);
        assert_eq!(starts(line.sequences(Black, Five)), []);
        let line = "-xxxxxx--------".parse::<Line>()?;
        assert_eq!(starts(line.sequences(White, Five)), [1, 2]);
        Ok(())
    }

    #[test]
    fn test_potentials() -> Result<(), String> {
        // A segment is worth its score + 1, nothing if it is dead. An empty
        // cell scores the best segment through it, times how many segments
        // through it reach that best; below `min` it is not reported.
        //
        // Cell 7 is in three segments holding both stones (3 each): 9. Cell
        // 5 is in one of them: 3. The same holds mirrored on the right.
        let line = "--------oo-----".parse::<Line>()?;
        let expected = [(5, 3), (6, 6), (7, 9), (10, 9), (11, 6), (12, 3)];
        assert_eq!(line.potentials(Black, 3).collect::<Vec<_>>(), expected);

        // Segments through White's stone at cell 6 are dead, so cell 5 only
        // sees segment 1 (two stones).
        let line = "--x-x-ox---xxx-".parse::<Line>()?;
        assert_eq!(
            line.potentials(White, 3).collect::<Vec<_>>(),
            [
                (0, 3),
                (1, 6),
                (3, 6),
                (5, 3),
                (8, 6),
                (9, 4),
                (10, 8),
                (14, 4)
            ]
        );
        // For Black, segments 7 (cells 7-11) and 8 (cells 8-12) have a
        // black stone just outside them, at 12 and at 7, and would make an
        // overline; that leaves cell 8 with nothing.
        let line = "--o-o-xo---ooo-".parse::<Line>()?;
        assert_eq!(
            line.potentials(Black, 3).collect::<Vec<_>>(),
            [(0, 3), (1, 6), (3, 6), (5, 3), (9, 4), (10, 8), (14, 4)]
        );
        Ok(())
    }

    #[test]
    fn test_put_and_remove() -> Result<(), String> {
        let mut line = "o-x----".parse::<Line>()?;
        line.put_mut(Black, 4);
        // Putting on an occupied cell replaces the stone.
        line.put_mut(White, 0);
        line.remove_mut(2);
        // Removing from an empty cell does nothing.
        line.remove_mut(3);
        assert_eq!(line, "x---o--".parse::<Line>()?);
        Ok(())
    }

    #[test]
    fn test_stones() -> Result<(), String> {
        let line = "o-ox-".parse::<Line>()?;
        assert_eq!(line.stone(0), Some(Black));
        assert_eq!(line.stone(1), None);
        assert_eq!(line.stone(3), Some(White));
        assert_eq!(line.stones(Black).collect::<Vec<_>>(), [0, 2]);
        assert_eq!(line.stones(White).collect::<Vec<_>>(), [3]);
        assert_eq!(line.empties().collect::<Vec<_>>(), [1, 4]);
        Ok(())
    }

    /// `potential_cap` bounds every window's potential (own stones + 1), so
    /// that lines with nothing to find can be skipped.
    #[test]
    fn test_potential_cap() -> Result<(), String> {
        let line = "-----".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 1);
        assert_eq!(line.potential_cap(White), 1);

        // No room for White's five beside Black's stone: nothing to find.
        let line = "--o--".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 2);
        assert_eq!(line.potential_cap(White), 0);

        let line = "--o---".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 2);
        assert_eq!(line.potential_cap(White), 1);

        let line = "o----x".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 2);
        assert_eq!(line.potential_cap(White), 2);

        Ok(())
    }

    #[test]
    fn test_to_string_and_parse() -> Result<(), String> {
        let line = "-o---x-".parse::<Line>()?;
        assert_eq!(line.to_string(), " . o . . . x .");
        // Spaces are ignored, so the output parses back.
        assert_eq!(line.to_string().parse::<Line>()?, line);
        assert!("-".repeat(16).parse::<Line>().is_err());
        Ok(())
    }
}
