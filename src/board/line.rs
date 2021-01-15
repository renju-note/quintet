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

    pub fn put(&self, black: bool, i: u8) -> Line {
        let stones = 0b1 << i;
        let blacks: Bits;
        let whites: Bits;
        if black {
            blacks = self.blacks | stones;
            whites = self.whites & !stones;
        } else {
            blacks = self.blacks & !stones;
            whites = self.whites | stones;
        }
        Line {
            size: self.size,
            blacks: blacks,
            whites: whites,
        }
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
