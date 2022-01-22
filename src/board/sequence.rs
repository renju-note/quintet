use std::fmt;

const SLOT_SIZE: u8 = 5;
const WINDOW_SIZE: u8 = SLOT_SIZE + 2;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SequenceKind {
    Single,
    Double,
}

use SequenceKind::*;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Sequence(pub u8);

impl Sequence {
    pub fn eyes(&self) -> &'static [u8] {
        EYES[self.0 as usize]
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
    black: bool,
    limit: u8,
    i: u8,
    prev_matched: bool,
}

impl Sequences {
    pub fn new(
        my: u16,
        op: u16,
        kind: SequenceKind,
        n: u8,
        exact: bool,
        start: u8,
        end: u8,
    ) -> Self {
        Self {
            my: my,
            op: op,
            double: kind == Double,
            n: n,
            black: exact,
            limit: end - (WINDOW_SIZE - 1),
            i: start,
            prev_matched: false,
        }
    }

    pub fn matches(&self, op: u8, my: u8) -> bool {
        if (op & 0b00111110) != 0b0 {
            return false;
        }

        if self.black && my & 0b01000001 != 0b0 {
            return false;
        }

        let stones = (my & 0b00111110) >> 1;
        COUNT_ONES[stones as usize] == self.n
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
                let signature = (my & 0b00011110) >> 1;
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

const COUNT_ONES: [u8; 32] = [
    0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 1, 2, 2, 3, 2, 3, 3, 4, 2, 3, 3, 4, 3, 4, 4, 5,
];

const EYES: [&[u8]; 32] = [
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
