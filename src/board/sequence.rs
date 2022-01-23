use super::util;
use std::cmp;
use std::fmt;

const SLOT_SIZE: u8 = 5;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SequenceKind {
    Single,
    Double,
}

pub use SequenceKind::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Sequence(pub u8);

impl Sequence {
    pub fn eyes(&self) -> &'static [u8] {
        util::eyes(self.0)
    }
}

impl fmt::Debug for Sequence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sequence({:#010b})", self.0)
    }
}

pub struct Sequences {
    my: u16,
    op: u16,
    double: bool,
    n: u8,
    exact: bool,
    limit: u8,
    i: u8,
    prev_matched: bool,
}

impl Sequences {
    pub fn new(size: u8, my: u16, op: u16, kind: SequenceKind, n: u8, exact: bool) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            double: kind == Double,
            n: n,
            exact: exact,
            limit: size - SLOT_SIZE,
            i: 0,
            prev_matched: false,
        }
    }

    pub fn new_on(
        i: u8,
        size: u8,
        my: u16,
        op: u16,
        kind: SequenceKind,
        n: u8,
        exact: bool,
    ) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            double: kind == Double,
            n: n,
            exact: exact,
            limit: cmp::min(size - SLOT_SIZE, i),
            i: cmp::max(0, i as i8 - SLOT_SIZE as i8 + 1) as u8,
            prev_matched: false,
        }
    }

    fn matches(&self, op: u8, my: u8) -> bool {
        if (op & 0b00111110) != 0b0 {
            return false;
        }

        if self.exact && my & 0b01000001 != 0b0 {
            return false;
        }

        let stones = (my & 0b00111110) >> 1;
        util::count_ones(stones) == self.n
    }
}

impl Iterator for Sequences {
    type Item = (u8, Sequence);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        if i > self.limit {
            return None;
        }
        self.i += 1;

        let op = (self.op >> i & 0b01111111) as u8;
        let my = (self.my >> i & 0b01111111) as u8;
        let matched = self.matches(op, my);

        if self.double {
            let prev_matched = self.prev_matched;
            self.prev_matched = matched;
            if prev_matched && matched {
                let signature = (my & 0b00011110 | 0b00100000) >> 1;
                return Some((i, Sequence(signature as u8)));
            }
        } else {
            if matched {
                let signature = (my & 0b00111110) >> 1;
                return Some((i, Sequence(signature as u8)));
            }
        }

        self.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequences_single() {
        let my = 0b011100010010100;
        let op = 0b000000001000000;
        let k = Single;

        let result = Sequences::new(15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [
            (0, Sequence(0b00010100)),
            (1, Sequence(0b00001010)),
            (7, Sequence(0b00010001)),
            (8, Sequence(0b00011000)),
        ];
        assert_eq!(result, expected);

        let result = Sequences::new(11, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(0, Sequence(0b00010100)), (1, Sequence(0b00001010))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [(0, Sequence(0b00010100)), (1, Sequence(0b00001010))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 3, false).collect::<Vec<_>>();
        let expected = [(9, Sequence(0b00011100)), (10, Sequence(0b00001110))];
        assert_eq!(result, expected);

        let result = Sequences::new_on(7, 15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(7, Sequence(0b00010001))];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_sequences_double() {
        let my = 0b001010100001010;
        let op = 0b000000001000000;
        let k = Double;

        let result = Sequences::new(15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00010101)), (10, Sequence(0b00010101))];
        assert_eq!(result, expected);

        let result = Sequences::new(9, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00010101))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00010101))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 3, false).collect::<Vec<_>>();
        let expected = [];
        assert_eq!(result, expected);

        let result = Sequences::new_on(10, 15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(10, Sequence(0b00010101))];
        assert_eq!(result, expected);
    }
}
