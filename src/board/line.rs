use super::player::*;
use super::potential::*;
use super::sequence::*;
use super::structure::StructureKind::Sword;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

const MAX_SIZE: u8 = 16 - 1;

#[derive(Debug, PartialEq, Eq, Clone)]
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

    pub fn sequences(&self, r: Player, k: SequenceKind, n: u8, exact: bool) -> Sequences {
        let (my, op) = self.my_op(r);
        Sequences::new(self.size, my, op, k, n, exact)
    }

    pub fn sequences_on(&self, i: u8, r: Player, k: SequenceKind, n: u8, exact: bool) -> Sequences {
        let (my, op) = self.my_op(r);
        Sequences::new_on(i, self.size, my, op, k, n, exact)
    }

    pub fn potentials(&self, r: Player, min: u8, exact: bool) -> Potentials {
        let (my, op) = self.my_op(r);
        Potentials::new(self.size, my, op, min, exact)
    }

    pub fn potential_cap(&self, r: Player) -> u8 {
        let (my, op) = self.my_op(r);
        if self.size < op.count_ones() as u8 + 5 {
            return 0;
        }
        my.count_ones() as u8 + 1
    }

    /// Bit `j` is set if `r` has a sword in the window starting at cell `j`:
    /// the windows `sequences` lists for `Sword`, all at once.
    ///
    /// Each window's conditions are checked for every window in parallel,
    /// bit `j` standing for the window at `j`: no opponent stone in it,
    /// exactly three own stones (a bit-sliced sum of the five cells), and
    /// for Black no own stone just outside it.
    pub fn sword_starts(&self, r: Player) -> u16 {
        let (my, op) = self.my_op(r);
        let windows = (1u16 << (self.size + 1 - VICTORY)) - 1;
        let blocked = op | op >> 1 | op >> 2 | op >> 3 | op >> 4;
        // Add up the five cells of each window: a full adder on the first
        // three, a half adder on the last two, then the carries.
        let (a, b, c, d, e) = (my, my >> 1, my >> 2, my >> 3, my >> 4);
        let (s1, c1) = (a ^ b ^ c, a & b | c & (a ^ b));
        let (s2, c2) = (d ^ e, d & e);
        let (ones, c3) = (s1 ^ s2, s1 & s2);
        let twos = c1 ^ c2 ^ c3;
        let fours = c1 & c2 | c3 & (c1 ^ c2);
        let three = ones & twos & !fours;
        let (_, _, exact) = Sword.to_sequence(r);
        let overline = if exact { my << 1 | my >> 5 } else { 0 };
        three & !blocked & !overline & windows
    }

    /// `r`'s stones in the window starting at cell `j`, as `sequences`
    /// reports them.
    pub fn window(&self, r: Player, j: u8) -> Sequence {
        let (my, _) = self.my_op(r);
        Sequence((my >> j) as u8 & 0b11111)
    }

    /// `r`'s stones and the opponent's, in that order.
    fn my_op(&self, r: Player) -> (u16, u16) {
        match r {
            Black => (self.blacks, self.whites),
            White => (self.whites, self.blacks),
        }
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

    /// What `sequences` says, one window at a time.
    fn sword_starts_by_sequences(line: &Line, r: Player) -> u16 {
        let (k, n, exact) = Sword.to_sequence(r);
        let mut starts = 0;
        for (j, _) in line.sequences(r, k, n, exact) {
            starts |= 1 << j;
        }
        starts
    }

    #[test]
    fn test_sword_starts_matches_sequences() {
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
                assert_eq!(
                    line.sword_starts(r),
                    sword_starts_by_sequences(&line, r),
                    "{r:?} {line}"
                );
            }
        };
        // Every line up to nine cells, which covers the short diagonals and
        // every window with its margins.
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
        for _ in 0..100_000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let cells: Vec<u8> = (0..15).map(|i| ((x >> (2 * i)) % 3) as u8).collect();
            check(15, &cells);
        }
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
