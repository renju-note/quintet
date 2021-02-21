use super::bits::*;

pub struct Pattern {
    pub size: u8,
    pub filter: Bits,
    pub stones: Bits,
    pub blanks: Bits,
    pub eyes__: Bits,
}

impl Pattern {
    pub fn matches(&self, stones: Bits, blanks: Bits) -> bool {
        (stones & self.filter == self.stones) && (blanks & self.filter & self.blanks == self.blanks)
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

    pub fn eye1(&self) -> Option<u8> {
        match self.eyes__ {
            0b1 => Some(0),
            0b10 => Some(1),
            0b100 => Some(2),
            0b1000 => Some(3),
            0b10000 => Some(4),
            0b100000 => Some(5),
            0b1000000 => Some(6),
            0b11 => Some(0),
            0b101 => Some(0),
            0b110 => Some(1),
            0b1001 => Some(0),
            0b1010 => Some(1),
            0b1100 => Some(2),
            0b10001 => Some(0),
            0b10010 => Some(1),
            0b10100 => Some(2),
            0b11000 => Some(3),
            0b100010 => Some(1),
            0b100100 => Some(2),
            0b101000 => Some(3),
            0b110000 => Some(4),
            _ => None,
        }
    }

    pub fn eye2(&self) -> Option<u8> {
        match self.eyes__ {
            0b11 => Some(1),
            0b101 => Some(2),
            0b110 => Some(2),
            0b1001 => Some(3),
            0b1010 => Some(3),
            0b1100 => Some(3),
            0b10001 => Some(4),
            0b10010 => Some(4),
            0b10100 => Some(4),
            0b11000 => Some(4),
            0b100010 => Some(5),
            0b100100 => Some(5),
            0b101000 => Some(5),
            0b110000 => Some(5),
            _ => None,
        }
    }
}

pub const BLACK_TWOS: [Pattern; 6] = [
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00001100,
        blanks: 0b01110010,
        eyes__: 0b00110000,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00010100,
        blanks: 0b01101010,
        eyes__: 0b00101000,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00011000,
        blanks: 0b01100110,
        eyes__: 0b00100100,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00100100,
        blanks: 0b01011010,
        eyes__: 0b00011000,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00101000,
        blanks: 0b01010110,
        eyes__: 0b00010100,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00110000,
        blanks: 0b01001110,
        eyes__: 0b00001100,
    },
];

pub const BLACK_THREES: [Pattern; 4] = [
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00011100,
        blanks: 0b01100010,
        eyes__: 0b00100000,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00101100,
        blanks: 0b01010010,
        eyes__: 0b00010000,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00110100,
        blanks: 0b01001010,
        eyes__: 0b00001000,
    },
    Pattern {
        size: 8,
        filter: 0b11111111,
        stones: 0b00111000,
        blanks: 0b01000110,
        eyes__: 0b00000100,
    },
];

pub const BLACK_SWORDS: [Pattern; 10] = [
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0001110,
        blanks: 0b0110000,
        eyes__: 0b0110000,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0010110,
        blanks: 0b0101000,
        eyes__: 0b0101000,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0011010,
        blanks: 0b0100100,
        eyes__: 0b0100100,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0011100,
        blanks: 0b0100010,
        eyes__: 0b0100010,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0100110,
        blanks: 0b0011000,
        eyes__: 0b0011000,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0101010,
        blanks: 0b0010100,
        eyes__: 0b0010100,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0101100,
        blanks: 0b0010010,
        eyes__: 0b0010010,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0110010,
        blanks: 0b0001100,
        eyes__: 0b0001100,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0110100,
        blanks: 0b0001010,
        eyes__: 0b0001010,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0111000,
        blanks: 0b0000110,
        eyes__: 0b0000110,
    },
];

pub const BLACK_FOURS: [Pattern; 5] = [
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0011110,
        blanks: 0b0100000,
        eyes__: 0b0100000,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0101110,
        blanks: 0b0010000,
        eyes__: 0b0010000,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0110110,
        blanks: 0b0001000,
        eyes__: 0b0001000,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0111010,
        blanks: 0b0000100,
        eyes__: 0b0000100,
    },
    Pattern {
        size: 7,
        filter: 0b1111111,
        stones: 0b0111100,
        blanks: 0b0000010,
        eyes__: 0b0000010,
    },
];

pub const BLACK_FIVES: [Pattern; 1] = [Pattern {
    size: 7,
    filter: 0b1111111,
    stones: 0b0111110,
    blanks: 0b0000000,
    eyes__: 0b0000000,
}];

pub const BLACK_OVERLINES: [Pattern; 1] = [Pattern {
    size: 6,
    filter: 0b111111,
    stones: 0b111111,
    blanks: 0b000000,
    eyes__: 0b000000,
}];

pub const WHITE_TWOS: [Pattern; 6] = [
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b000110,
        blanks: 0b111001,
        eyes__: 0b011000,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b001010,
        blanks: 0b110101,
        eyes__: 0b010100,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b001100,
        blanks: 0b110011,
        eyes__: 0b010010,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b010010,
        blanks: 0b101101,
        eyes__: 0b001100,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b010100,
        blanks: 0b101011,
        eyes__: 0b001010,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b011000,
        blanks: 0b100111,
        eyes__: 0b000110,
    },
];

pub const WHITE_THREES: [Pattern; 4] = [
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b001110,
        blanks: 0b110001,
        eyes__: 0b010000,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b010110,
        blanks: 0b101001,
        eyes__: 0b001000,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b011010,
        blanks: 0b100101,
        eyes__: 0b000100,
    },
    Pattern {
        size: 6,
        filter: 0b111111,
        stones: 0b011100,
        blanks: 0b100011,
        eyes__: 0b000010,
    },
];

pub const WHITE_SWORDS: [Pattern; 10] = [
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b00111,
        blanks: 0b11000,
        eyes__: 0b11000,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b01011,
        blanks: 0b10100,
        eyes__: 0b10100,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b01101,
        blanks: 0b10010,
        eyes__: 0b10010,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b01110,
        blanks: 0b10001,
        eyes__: 0b10001,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b10011,
        blanks: 0b01100,
        eyes__: 0b01100,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b10101,
        blanks: 0b01010,
        eyes__: 0b01010,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b10110,
        blanks: 0b01001,
        eyes__: 0b01001,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b11001,
        blanks: 0b00110,
        eyes__: 0b00110,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b11010,
        blanks: 0b00101,
        eyes__: 0b00101,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b11100,
        blanks: 0b00011,
        eyes__: 0b00011,
    },
];

pub const WHITE_FOURS: [Pattern; 5] = [
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b01111,
        blanks: 0b10000,
        eyes__: 0b10000,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b10111,
        blanks: 0b01000,
        eyes__: 0b01000,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b11011,
        blanks: 0b00100,
        eyes__: 0b00100,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b11101,
        blanks: 0b00010,
        eyes__: 0b00010,
    },
    Pattern {
        size: 5,
        filter: 0b11111,
        stones: 0b11110,
        blanks: 0b00001,
        eyes__: 0b00001,
    },
];

pub const WHITE_FIVES: [Pattern; 1] = [Pattern {
    size: 5,
    filter: 0b11111,
    stones: 0b11111,
    blanks: 0b00000,
    eyes__: 0b00000,
}];
