use super::util;
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
    prev_ok: bool,
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
            prev_ok: false,
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
            limit: if i + SLOT_SIZE <= size {
                i
            } else {
                size - SLOT_SIZE
            },
            i: if SLOT_SIZE - 1 <= i {
                i - (SLOT_SIZE - 1)
            } else {
                0
            },
            prev_ok: false,
        }
    }

    fn match_all(&self, my: u8) -> bool {
        util::count_ones((my & 0b00111110) >> 1) == self.n
    }

    fn match_head(&self, my: u8) -> bool {
        util::count_ones((my & 0b00011110) >> 1) == self.n
    }

    fn match_rest(&self, my: u8) -> bool {
        util::count_ones((my & 0b00111100) >> 1) == self.n
    }

    fn invalid(&self, my: u8, op: u8) -> bool {
        op & 0b00111110 != 0b0 || self.exact && my & 0b01000001 != 0b0
    }
}

impl Iterator for Sequences {
    type Item = (u8, Sequence);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i > self.limit {
            return None;
        }
        let i = self.i;
        self.i += 1;

        let op = (self.op >> i & 0b01111111) as u8;
        let my = (self.my >> i & 0b01111111) as u8;

        if self.invalid(my, op) {
            if self.kind != Single {
                self.prev_ok = false;
            }
            return self.next();
        }

        match self.kind {
            Single => {
                if self.match_all(my) {
                    let signature = (my & 0b00111110) >> 1;
                    return Some((i, Sequence(signature)));
                }
            }
            Union => {
                let ok = self.match_all(my);
                let prev_ok = self.prev_ok;
                self.prev_ok = ok;
                if prev_ok && ok {
                    let signature = (my & 0b00111110) >> 1;
                    return Some((i, Sequence(signature)));
                }
            }
            Intersect => {
                let ok = self.match_all(my) && self.match_head(my);
                let prev_ok = self.prev_ok;
                self.prev_ok = self.match_rest(my);
                if prev_ok && ok {
                    let signature = (my & 0b00011110 | 0b00100000) >> 1;
                    return Some((i, Sequence(signature)));
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
