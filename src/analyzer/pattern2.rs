use super::super::board::Bits;

pub struct Pattern {
    pub filter: Bits,
    pub stones: Bits,
    pub blanks: Bits,
    pub eyes__: Bits,
}

impl Pattern {
    pub fn size(&self) -> u8 {
        match self.filter {
            0b11111 => 5,
            0b111111 => 6,
            0b1111111 => 7,
            0b11111111 => 8,
            _ => 0,
        }
    }

    pub fn start(&self) -> u8 {
        match (self.stones | self.blanks) & 0b11 {
            0b11 => 0,
            0b10 => 1,
            0b00 => 2,
            _ => 0,
        }
    }

    pub fn end(&self) -> u8 {
        match self.stones | self.blanks {
            0b1111 => 3,
            0b11110 => 4,
            0b111100 => 5,
            0b11111 => 4,
            0b111110 => 5,
            0b1111100 => 6,
            0b111111 => 5,
            0b1111110 => 6,
            0b11111100 => 7,
            _ => 0,
        }
    }

    pub fn eyes(&self) -> Vec<u8> {
        match self.eyes__ {
            0b1 => vec![0],
            0b10 => vec![1],
            0b100 => vec![2],
            0b1000 => vec![3],
            0b10000 => vec![4],
            0b100000 => vec![5],
            0b1000000 => vec![6],
            _ => vec![],
        }
    }
}

pub const BLACK_FOURS: [Pattern; 5] = [
    Pattern {
        filter: 0b1111111,
        stones: 0b0011110,
        blanks: 0b0100000,
        eyes__: 0b0100000,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0101110,
        blanks: 0b0010000,
        eyes__: 0b0010000,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0110110,
        blanks: 0b0001000,
        eyes__: 0b0001000,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0111010,
        blanks: 0b0000100,
        eyes__: 0b0000100,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0111100,
        blanks: 0b0000010,
        eyes__: 0b0000010,
    },
];

pub const BLACK_FIVES: [Pattern; 1] = [Pattern {
    filter: 0b1111111,
    stones: 0b0111110,
    blanks: 0b0000000,
    eyes__: 0b0000000,
}];

pub const BLACK_OVERLINES: [Pattern; 1] = [Pattern {
    filter: 0b111111,
    stones: 0b111111,
    blanks: 0b000000,
    eyes__: 0b000000,
}];

pub const WHITE_FOURS: [Pattern; 5] = [
    Pattern {
        filter: 0b11111,
        stones: 0b01111,
        blanks: 0b10000,
        eyes__: 0b10000,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10111,
        blanks: 0b01000,
        eyes__: 0b01000,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11011,
        blanks: 0b00100,
        eyes__: 0b00100,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11101,
        blanks: 0b00010,
        eyes__: 0b00010,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11110,
        blanks: 0b00001,
        eyes__: 0b00001,
    },
];

pub const WHITE_FIVES: [Pattern; 1] = [Pattern {
    filter: 0b11111,
    stones: 0b11111,
    blanks: 0b00000,
    eyes__: 0b00000,
}];
