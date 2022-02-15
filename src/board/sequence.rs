use std::fmt;

pub const VICTORY: u8 = 5;

const TARGET_MASK: u8 = 0b00111110;
const MARGIN_MASK: u8 = 0b01000001;
const HEAD_MASK: u8 = 0b00011110;
const REST_MASK: u8 = 0b00111100;
const LAST_MASK: u8 = 0b00100000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SequenceKind {
    Single,
    Double,
    Compact,
}

pub use SequenceKind::*;

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

pub struct Sequences {
    my: u16,
    op: u16,
    k: SequenceKind,
    n: u8,
    strict: bool,
    limit: u8,
    i: u8,
    prev_ok: bool,
}

impl Sequences {
    pub fn new(size: u8, my: u16, op: u16, k: SequenceKind, n: u8, strict: bool) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            k: k,
            n: n,
            strict: strict,
            limit: size - VICTORY,
            i: 0,
            prev_ok: false,
        }
    }

    pub fn new_on(i: u8, size: u8, my: u16, op: u16, k: SequenceKind, n: u8, strict: bool) -> Self {
        Self {
            my: my << 1,
            op: op << 1,
            k: k,
            n: n,
            strict: strict,
            limit: i.min(size - VICTORY),
            i: i.max(VICTORY - 1) - (VICTORY - 1),
            prev_ok: false,
        }
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

        let op_ = (self.op >> i) as u8;
        let my_ = (self.my >> i) as u8;

        if op_ & TARGET_MASK != 0b0 || self.strict && my_ & MARGIN_MASK != 0b0 {
            if self.k != Single {
                self.prev_ok = false;
            }
            return self.next();
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
            Compact => {
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
    fn test_sequences_double_or_compact() {
        let my = 0b001000110001010;
        let op = 0b000000001000000;

        let k = Double;

        let result = Sequences::new(15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00000101)), (8, Sequence(0b00010001))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00000101))];
        assert_eq!(result, expected);

        let k = Compact;

        let result = Sequences::new(15, my, op, k, 2, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00010101))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00010101))];
        assert_eq!(result, expected);

        let my = 0b111101000110111;
        let op = 0b000000000000000;

        let k = Double;

        let result = Sequences::new(15, my, op, k, 4, false).collect::<Vec<_>>();
        let expected = [(1, Sequence(0b00011011)), (10, Sequence(0b00011110))];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [];
        assert_eq!(result, expected);

        let k = Compact;

        let result = Sequences::new(15, my, op, k, 4, false).collect::<Vec<_>>();
        let expected = [];
        assert_eq!(result, expected);

        let result = Sequences::new(15, my, op, k, 2, true).collect::<Vec<_>>();
        let expected = [];
        assert_eq!(result, expected);
    }
}

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
