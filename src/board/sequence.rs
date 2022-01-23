use super::util;
use std::cmp;
use std::fmt;

const SLOT_SIZE: u8 = 5;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SequenceKind {
    Single,
    Union,
    Intersect,
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
    kind: SequenceKind,
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
            kind: kind,
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
            kind: kind,
            n: n,
            exact: exact,
            limit: cmp::min(size - SLOT_SIZE, i),
            i: cmp::max(0, i as i8 - SLOT_SIZE as i8 + 1) as u8,
            prev_matched: false,
        }
    }

    fn match_body(&self, my: u8) -> bool {
        let body = (my & 0b00111110) >> 1;
        util::count_ones(body) == self.n
    }

    fn match_head(&self, my: u8) -> bool {
        let head = (my & 0b00011110) >> 1;
        util::count_ones(head) == self.n
    }

    fn match_rest(&self, my: u8) -> bool {
        let rest = (my & 0b00111100) >> 1;
        util::count_ones(rest) == self.n
    }

    fn valid(&self, op: u8, my: u8) -> bool {
        if self.exact && my & 0b01000001 != 0b0 {
            return false;
        }
        (op & 0b00111110) == 0b0
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

        if !self.valid(op, my) {
            self.prev_matched = false;
            return self.next();
        }

        match self.kind {
            Single => {
                let matched = self.match_body(my);
                if matched {
                    let signature = (my & 0b00111110) >> 1;
                    return Some((i, Sequence(signature as u8)));
                }
            }
            Union => {
                let matched = self.match_body(my);
                let prev_matched = self.prev_matched;
                self.prev_matched = matched;
                if prev_matched && matched {
                    let signature = (my & 0b00111110) >> 1;
                    return Some((i, Sequence(signature as u8)));
                }
            }
            Intersect => {
                let matched = self.match_body(my) && self.match_head(my);
                let prev_matched = self.prev_matched;
                self.prev_matched = self.match_rest(my);
                if prev_matched && matched {
                    let signature = (my & 0b00011110 | 0b00100000) >> 1;
                    return Some((i, Sequence(signature as u8)));
                }
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
    }

    #[test]
    fn test_sequences_double() {
        let my = 0b001000110001010;
        let op = 0b000000001000000;

        let k = Union;

        let result = Sequences::new(15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00000101)), (8, Sequence(0b00010001))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00000101))];
        assert_eq!(result, expected);

        let k = Intersect;

        let result = Sequences::new(15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00010101))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00010101))];
        assert_eq!(result, expected);

        let my = 0b111101000110111;
        let op = 0b000000000000000;

        let k = Union;

        let result = Sequences::new(15, my, op, k, 4, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00011011)), (10, Sequence(0b00011110))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [];
        assert_eq!(result, expected);

        let k = Intersect;

        let result = Sequences::new(15, my, op, k, 4, false).collect::<Vec<_>>();
        let expected = [];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [];
        assert_eq!(result, expected);
    }
}
