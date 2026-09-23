use super::potential::VICTORY;
use std::fmt;

const TARGET_MASK: u8 = 0b00111110;
const MARGIN_MASK: u8 = 0b01000001;
const HEAD_MASK: u8 = 0b00011110;
const REST_MASK: u8 = 0b00111100;
const LAST_MASK: u8 = 0b00100000;

pub struct Sequences {
    my: u16,
    op: u16,
    k: SequenceKind,
    n: u8,
    exact: bool,
    limit: u8,
    i: u8,
    prev_ok: bool,
}

impl Sequences {
    pub fn new(size: u8, my: u16, op: u16, k: SequenceKind, n: u8, exact: bool) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            k,
            n,
            exact,
            limit: size - VICTORY,
            i: 0,
            prev_ok: false,
        }
    }

    pub fn new_on(i: u8, size: u8, my: u16, op: u16, k: SequenceKind, n: u8, exact: bool) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            k,
            n,
            exact,
            limit: i.min(size - VICTORY),
            i: i.max(VICTORY - 1) - (VICTORY - 1),
            prev_ok: false,
        }
    }
}

impl Iterator for Sequences {
    type Item = (u8, Sequence);

    // Windows that do not match are stepped over in this loop rather than by
    // calling `next` again: the recursion was real, not a tail call the
    // compiler folded away, and a skipped window is the common case.
    //
    // `self.i` is stepped by hand because a `for` over a range measured
    // slower, and this loop runs once per five-cell window of every line the
    // solvers look at — most of their time. `for i in self.i..=self.limit`
    // costs a VCF search 8%: `RangeInclusive` carries an extra flag for the
    // `end == u8::MAX` case, which cannot arise here because `limit` is at
    // most `RANGE - VICTORY`. Keeping the bound exclusive to get a plain
    // `Range` still costs 2%.
    fn next(&mut self) -> Option<Self::Item> {
        while self.i <= self.limit {
            let i = self.i;
            self.i += 1;

            let op_ = (self.op >> i) as u8;
            let my_ = (self.my >> i) as u8;

            if op_ & TARGET_MASK != 0b0 || self.exact && my_ & MARGIN_MASK != 0b0 {
                if self.k != Single {
                    self.prev_ok = false;
                }
                continue;
            }

            let my = my_ & TARGET_MASK;
            let ok = my.count_ones() as u8 == self.n;
            match self.k {
                Single => {
                    if ok {
                        return Some((i, Sequence(my >> 1)));
                    }
                }
                Double => {
                    let prev_ok = self.prev_ok;
                    self.prev_ok = ok;
                    if prev_ok && ok {
                        return Some((i, Sequence(my >> 1)));
                    }
                }
                Open => {
                    let prev_ok = self.prev_ok;
                    self.prev_ok = (my & REST_MASK).count_ones() as u8 == self.n;
                    if ok && prev_ok && (my & HEAD_MASK).count_ones() as u8 == self.n {
                        // discard non-eye
                        // I know '& HEAD_MASK' is not necessary,
                        // but I found removing it makes VCF solver slower for 5-10%.
                        // It'a mistery...
                        return Some((i, Sequence((my & HEAD_MASK | LAST_MASK) >> 1)));
                    }
                }
            }
        }

        None
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Sequence(pub u8);

impl Sequence {
    pub fn stones(&self) -> &'static [u8] {
        STONES_DATA[self.0 as usize]
    }

    pub fn eyes(&self) -> &'static [u8] {
        EYES_DATA[self.0 as usize]
    }
}

impl fmt::Debug for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sequence({:#010b})", self.0)
    }
}

/// How `Sequences` matches a 5-cell window `i` holding exactly `n` own stones.
///
/// - `Single`: window `i` alone.
/// - `Double`: windows `i-1` and `i` both match, i.e. a 6-cell span.
///   Both ends are either stones (`n + 1` stones in 6 cells) or empty (`Open`).
/// - `Open`: the `Double` case whose ends `i-1` and `i+4` are both empty,
///   so the `n` stones lie in the middle 4 cells with open ends on both sides.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SequenceKind {
    Single,
    Double,
    Open,
}

pub use SequenceKind::*;

const STONES_DATA: [&[u8]; 32] = [
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

const EYES_DATA: [&[u8]; 32] = [
    &[0, 1, 2, 3, 4],
    &[1, 2, 3, 4],
    &[0, 2, 3, 4],
    &[2, 3, 4],
    &[0, 1, 3, 4],
    &[1, 3, 4],
    &[0, 3, 4],
    &[3, 4],
    &[0, 1, 2, 4],
    &[1, 2, 4],
    &[0, 2, 4],
    &[2, 4],
    &[0, 1, 4],
    &[1, 4],
    &[0, 4],
    &[4],
    &[0, 1, 2, 3],
    &[1, 2, 3],
    &[0, 2, 3],
    &[2, 3],
    &[0, 1, 3],
    &[1, 3],
    &[0, 3],
    &[3],
    &[0, 1, 2],
    &[1, 2],
    &[0, 2],
    &[2],
    &[0, 1],
    &[1],
    &[0],
    &[],
];

#[cfg(test)]
mod tests {
    use super::*;

    /// A line written like `"-o-ox"`, cell 0 first: `o` is an own stone, `x`
    /// an opponent's. Returns `(size, my, op)`.
    fn bits(line: &str) -> (u8, u16, u16) {
        let (mut my, mut op) = (0, 0);
        for (i, c) in line.chars().enumerate() {
            match c {
                'o' => my |= 1 << i,
                'x' => op |= 1 << i,
                _ => {}
            }
        }
        (line.len() as u8, my, op)
    }

    /// Each match as its window's start and the window's five cells, `o`
    /// where the sequence has a bit.
    fn show(found: impl Iterator<Item = (u8, Sequence)>) -> Vec<(u8, String)> {
        found
            .map(|(i, s)| {
                let cells = (0..5).map(|j| if s.0 & (1 << j) != 0 { 'o' } else { '-' });
                (i, cells.collect())
            })
            .collect()
    }

    fn sequences(line: &str, k: SequenceKind, n: u8, exact: bool) -> Vec<(u8, String)> {
        let (size, my, op) = bits(line);
        show(Sequences::new(size, my, op, k, n, exact))
    }

    fn sequences_on(line: &str, i: u8, k: SequenceKind, n: u8, exact: bool) -> Vec<(u8, String)> {
        let (size, my, op) = bits(line);
        show(Sequences::new_on(i, size, my, op, k, n, exact))
    }

    #[test]
    fn test_single() {
        let line = "--o-o-xo---ooo-";

        // Every window with exactly two own stones and no opponent stone.
        let expected = [
            (0, "--o-o".to_string()),
            (1, "-o-o-".to_string()),
            (7, "o---o".to_string()),
            (8, "---oo".to_string()),
        ];
        assert_eq!(sequences(line, Single, 2, false), expected);

        // `exact` also drops the windows with an own stone just outside
        // them: filling those in would make six or more.
        assert_eq!(sequences(line, Single, 2, true), expected[..2]);

        // A shorter line has fewer windows.
        assert_eq!(sequences(&line[..11], Single, 2, false), expected[..2]);

        let expected = [(9, "--ooo".to_string()), (10, "-ooo-".to_string())];
        assert_eq!(sequences(line, Single, 3, false), expected);
    }

    #[test]
    fn test_double() {
        // Two overlapping windows that both match: `n + 1` stones in six
        // cells, reported at the second window. Cells 0-5 here (window 1)
        // and 7-12 (window 8).
        let line = "-o-o--xoo---o--";
        assert_eq!(
            sequences(line, Double, 2, false),
            [(1, "o-o--".to_string()), (8, "o---o".to_string())]
        );
        // `exact` looks one cell beyond each window, and window 8 (cells
        // 8-12) has an own stone at cell 7.
        assert_eq!(sequences(line, Double, 2, true), [(1, "o-o--".to_string())]);

        // Four stones in six cells, the shape that becomes an overline.
        let line = "ooo-oo---o-oooo";
        assert_eq!(
            sequences(line, Double, 4, false),
            [(1, "oo-oo".to_string()), (10, "-oooo".to_string())]
        );
        assert_eq!(sequences(line, Double, 2, true), []);
    }

    #[test]
    fn test_open() {
        // A `Double` whose two ends are both empty: the stones sit in the
        // middle four cells. The fifth cell is marked too, so that it is not
        // taken for an eye: it is the far open end, not a cell to play.
        let line = "-o-o--xoo---o--";
        let expected = [(1, "o-o-o".to_string())];
        assert_eq!(sequences(line, Open, 2, false), expected);
        assert_eq!(sequences(line, Open, 2, true), expected);

        // No six-cell span with both ends empty.
        let line = "ooo-oo---o-oooo";
        assert_eq!(sequences(line, Open, 4, false), []);
        assert_eq!(sequences(line, Open, 2, true), []);
    }

    #[test]
    fn test_sequences_on() {
        // Only the windows through cell 7.
        let line = "o--o--o---o---o";
        assert_eq!(
            sequences_on(line, 7, Single, 2, true),
            [(3, "o--o-".to_string()), (6, "o---o".to_string())]
        );

        let line = "-----oo-o-o----";
        assert_eq!(
            sequences_on(line, 7, Single, 3, false),
            [
                (4, "-oo-o".to_string()),
                (5, "oo-o-".to_string()),
                (6, "o-o-o".to_string()),
            ]
        );
        // Windows 5 and 6 touch another stone just outside them.
        assert_eq!(
            sequences_on(line, 7, Single, 3, true),
            [(4, "-oo-o".to_string())]
        );
        // Cells 4-9: open three with an eye at 7.
        assert_eq!(
            sequences_on(line, 7, Open, 3, false),
            [(5, "oo-oo".to_string())]
        );
        // ... but it would become an overline with cell 10.
        assert_eq!(sequences_on(line, 7, Open, 3, true), []);

        // One group on each side of cell 7, both reaching it.
        let line = "---ooo---ooo---";
        let expected = [(3, "ooo--".to_string()), (7, "--ooo".to_string())];
        assert_eq!(sequences_on(line, 7, Single, 3, false), expected);
        assert_eq!(sequences_on(line, 7, Single, 3, true), expected);
        assert_eq!(sequences_on(line, 7, Open, 3, false), []);
    }
}
