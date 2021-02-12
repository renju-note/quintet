use super::pattern::*;

const MAX_SIZE: u8 = 32;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Line {
    pub size: u8,
    pub blacks: Bits,
    pub whites: Bits,

    b4eyes: Bits,
    w4eyes: Bits,
    bseyes: Bits,
    wseyes: Bits,
}

impl Line {
    pub fn new(size: u8) -> Line {
        let size = std::cmp::min(size, MAX_SIZE);
        Line {
            size: size,
            blacks: 0b0,
            whites: 0b0,
            b4eyes: 0b0,
            w4eyes: 0b0,
            bseyes: 0b0,
            wseyes: 0b0,
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
        self.scan();
    }

    fn scan(&mut self) {
        let blacks_ = self.blacks << 1;
        let whites_ = self.whites << 1;
        let blanks_ = self.blanks() << 1;
        let limit = self.size + 2;
        self.b4eyes = scan_eyes(&BLACK_FOURS, blacks_, blanks_, limit) >> 1;
        self.w4eyes = scan_eyes(&WHITE_FOURS, whites_, blanks_, limit) >> 1;
        self.bseyes = scan_eyes(&BLACK_SWORDS, blacks_, blanks_, limit) >> 1;
        self.wseyes = scan_eyes(&WHITE_SWORDS, whites_, blanks_, limit) >> 1;
    }

    pub fn four_eyes(&self, black: bool) -> Vec<u8> {
        let target = if black { self.b4eyes } else { self.w4eyes };
        let mut result = vec![];
        for i in 0..self.size {
            if (target >> i) & 0b1 == 0b1 {
                result.push(i);
            }
        }
        result
    }

    pub fn sword_eyes(&self, black: bool) -> Vec<u8> {
        let target = if black { self.bseyes } else { self.wseyes };
        let mut result = vec![];
        for i in 0..self.size {
            if (target >> i) & 0b1 == 0b1 {
                result.push(i);
            }
        }
        result
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
