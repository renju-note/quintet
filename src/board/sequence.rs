use std::fmt;

const SLOT_SIZE: u8 = 5;
const WINDOW_SIZE: u8 = SLOT_SIZE + 2;

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
    black: bool,
    n: u8,
    my: u16,
    op: u16,
    limit: u8,
    i: u8,
}

impl Sequences {
    pub fn new(black: bool, n: u8, my: u16, op: u16, start: u8, end: u8) -> Self {
        Sequences {
            black: black,
            n: n,
            my: my,
            op: op,
            limit: end - (WINDOW_SIZE - 1),
            i: start,
        }
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

        let op = self.op >> i;
        if (op & 0b00111110) != 0b0 {
            return self.next();
        }

        let my = self.my >> i;
        if self.black && my & 0b01000001 != 0b0 {
            return self.next();
        }

        let signature = (my & 0b00111110) >> 1;
        if COUNT_ONES[signature as usize] == self.n {
            Some((i, Sequence(signature as u8)))
        } else {
            self.next()
        }
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
