const MAX_SIZE: u8 = 32;

pub type Bits = u32;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Line {
    pub size: u8,
    pub blacks: Bits,
    pub whites: Bits,
}

impl Line {
    pub fn new(size: u8) -> Line {
        let size = std::cmp::min(size, MAX_SIZE);
        Line {
            size: size,
            blacks: 0b0,
            whites: 0b0,
        }
    }

    pub fn put(&mut self, black: bool, i: u8) {
        let stones = 0b1 << i;
        if black {
            self.blacks |= stones;
            self.whites &= !stones;
        } else {
            self.blacks &= !stones;
            self.whites |= stones;
        }
    }

    pub fn blanks(&self) -> Bits {
        !(self.blacks | self.whites) & ((0b1 << self.size) - 1)
    }

    pub fn must_have(&self, black: bool, white: bool) -> bool {
        (!black || self.blacks != 0b0) && (!white || self.whites != 0b0)
    }

    pub fn to_string(&self) -> String {
        (0..self.size)
            .map(|i| {
                let pat = 0b1 << i;
                if self.blacks & pat != 0b0 {
                    'o'
                } else if self.whites & pat != 0b0 {
                    'x'
                } else {
                    '-'
                }
            })
            .collect()
    }
}
