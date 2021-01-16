use super::super::board::Bits;

pub struct Pattern {
    pub filter: Bits,
    pub stones: Bits,
    pub blanks: Bits,
    pub eyes__: Bits,
}

impl Pattern {
    pub fn matches(&self, stones: Bits, blanks: Bits, segment_offset: i8) -> Option<Segment> {
        if (stones & self.filter == self.stones)
            && (blanks & self.filter & self.blanks == self.blanks)
        {
            Some(Segment {
                start: self.start(segment_offset),
                end: self.end(segment_offset),
                eyes: self.eyes(segment_offset),
            })
        } else {
            None
        }
    }

    pub fn size(&self) -> u8 {
        match self.filter {
            0b11111 => 5,
            0b111111 => 6,
            0b1111111 => 7,
            0b11111111 => 8,
            _ => 0,
        }
    }

    fn start(&self, offset: i8) -> u8 {
        let result = match (self.stones | self.blanks) & 0b11 {
            0b11 => 0,
            0b10 => 1,
            0b00 => 2,
            _ => 0,
        };
        (result as i8 + offset) as u8
    }

    fn end(&self, offset: i8) -> u8 {
        let result = match self.stones | self.blanks {
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
        };
        (result as i8 + offset) as u8
    }

    fn eyes(&self, offset: i8) -> Vec<u8> {
        match self.eyes__ {
            0b1 => vec![(0 + offset) as u8],
            0b10 => vec![(1 + offset) as u8],
            0b100 => vec![(2 + offset) as u8],
            0b1000 => vec![(3 + offset) as u8],
            0b10000 => vec![(4 + offset) as u8],
            0b100000 => vec![(5 + offset) as u8],
            0b1000000 => vec![(6 + offset) as u8],
            0b11 => vec![(0 + offset) as u8, (1 + offset) as u8],
            0b101 => vec![(0 + offset) as u8, (2 + offset) as u8],
            0b110 => vec![(1 + offset) as u8, (2 + offset) as u8],
            0b1001 => vec![(0 + offset) as u8, (3 + offset) as u8],
            0b1010 => vec![(1 + offset) as u8, (3 + offset) as u8],
            0b1100 => vec![(2 + offset) as u8, (3 + offset) as u8],
            0b10001 => vec![(0 + offset) as u8, (4 + offset) as u8],
            0b10010 => vec![(1 + offset) as u8, (4 + offset) as u8],
            0b10100 => vec![(2 + offset) as u8, (4 + offset) as u8],
            0b11000 => vec![(3 + offset) as u8, (4 + offset) as u8],
            0b100010 => vec![(1 + offset) as u8, (5 + offset) as u8],
            0b100100 => vec![(2 + offset) as u8, (5 + offset) as u8],
            0b101000 => vec![(3 + offset) as u8, (5 + offset) as u8],
            0b110000 => vec![(4 + offset) as u8, (5 + offset) as u8],
            _ => vec![],
        }
    }
}

#[derive(Clone)]
pub struct Segment {
    pub start: u8,
    pub end: u8,
    pub eyes: Vec<u8>,
}

pub const BLACK_SWORDS: [Pattern; 10] = [
    Pattern {
        filter: 0b1111111,
        stones: 0b0001110,
        blanks: 0b0110000,
        eyes__: 0b0110000,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0010110,
        blanks: 0b0101000,
        eyes__: 0b0101000,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0011010,
        blanks: 0b0100100,
        eyes__: 0b0100100,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0011100,
        blanks: 0b0100010,
        eyes__: 0b0100010,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0100110,
        blanks: 0b0011000,
        eyes__: 0b0011000,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0101010,
        blanks: 0b0010100,
        eyes__: 0b0010100,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0101100,
        blanks: 0b0010010,
        eyes__: 0b0010010,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0110010,
        blanks: 0b0001100,
        eyes__: 0b0001100,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0110100,
        blanks: 0b0001010,
        eyes__: 0b0001010,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0111000,
        blanks: 0b0000110,
        eyes__: 0b0000110,
    },
];

pub const BLACK_THREES: [Pattern; 4] = [
    Pattern {
        filter: 0b11111111,
        stones: 0b00011100,
        blanks: 0b01100010,
        eyes__: 0b00100000,
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00101100,
        blanks: 0b01010010,
        eyes__: 0b00010000,
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00110100,
        blanks: 0b01001010,
        eyes__: 0b00001000,
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00111000,
        blanks: 0b01000110,
        eyes__: 0b00000100,
    },
];

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

pub const WHITE_SWORDS: [Pattern; 10] = [
    Pattern {
        filter: 0b11111,
        stones: 0b00111,
        blanks: 0b11000,
        eyes__: 0b11000,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b01011,
        blanks: 0b10100,
        eyes__: 0b10100,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b01101,
        blanks: 0b10010,
        eyes__: 0b10010,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b01110,
        blanks: 0b10001,
        eyes__: 0b10001,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10011,
        blanks: 0b01100,
        eyes__: 0b01100,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10101,
        blanks: 0b01010,
        eyes__: 0b01010,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10110,
        blanks: 0b01001,
        eyes__: 0b01001,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11001,
        blanks: 0b00110,
        eyes__: 0b00110,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11010,
        blanks: 0b00101,
        eyes__: 0b00101,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11100,
        blanks: 0b00011,
        eyes__: 0b00011,
    },
];

pub const WHITE_THREES: [Pattern; 4] = [
    Pattern {
        filter: 0b111111,
        stones: 0b001110,
        blanks: 0b110001,
        eyes__: 0b010000,
    },
    Pattern {
        filter: 0b111111,
        stones: 0b010110,
        blanks: 0b101001,
        eyes__: 0b001000,
    },
    Pattern {
        filter: 0b111111,
        stones: 0b011010,
        blanks: 0b100101,
        eyes__: 0b000100,
    },
    Pattern {
        filter: 0b111111,
        stones: 0b011100,
        blanks: 0b100011,
        eyes__: 0b000010,
    },
];

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
