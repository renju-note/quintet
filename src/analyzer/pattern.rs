use super::super::board::Stones;

pub struct RowPattern<'a> {
    pub row: &'a RowShape<'a>,
    pub size: u8,
    pub offset: u8,
    pub blacks: Stones,
    pub blmask: Stones,
    pub whites: Stones,
    pub whmask: Stones,
}

pub struct RowShape<'a> {
    pub size: u8,
    pub eyes: &'a [u8],
}

pub const BLACK_TWO_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[2, 3],
        },
        size: 8,
        offset: 2,
        blacks: 0b00001100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[1, 3],
        },
        size: 8,
        offset: 2,
        blacks: 0b00010100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[0, 3],
        },
        size: 8,
        offset: 2,
        blacks: 0b00011000,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[1, 2],
        },
        size: 8,
        offset: 2,
        blacks: 0b00100100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[0, 2],
        },
        size: 8,
        offset: 2,
        blacks: 0b00101000,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[0, 1],
        },
        size: 8,
        offset: 2,
        blacks: 0b00110000,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
];

pub const BLACK_SWORD_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[2, 3],
        },
        size: 7,
        offset: 1,
        blacks: 0b0100110,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1, 3],
        },
        size: 7,
        offset: 1,
        blacks: 0b0101010,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1, 2],
        },
        size: 7,
        offset: 1,
        blacks: 0b0110010,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[3, 4],
        },
        size: 7,
        offset: 1,
        blacks: 0b0001110,
        whites: 0b0000001,
        blmask: 0b0000000,
        whmask: 0b1000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[2, 4],
        },
        size: 7,
        offset: 1,
        blacks: 0b0010110,
        whites: 0b0000001,
        blmask: 0b0000000,
        whmask: 0b1000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1, 4],
        },
        size: 7,
        offset: 1,
        blacks: 0b0011010,
        whites: 0b0000001,
        blmask: 0b0000000,
        whmask: 0b1000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 3],
        },
        size: 7,
        offset: 1,
        blacks: 0b0101100,
        whites: 0b1000000,
        blmask: 0b0000000,
        whmask: 0b0000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 2],
        },
        size: 7,
        offset: 1,
        blacks: 0b0110100,
        whites: 0b1000000,
        blmask: 0b0000000,
        whmask: 0b0000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 1],
        },
        size: 7,
        offset: 1,
        blacks: 0b0111000,
        whites: 0b1000000,
        blmask: 0b0000000,
        whmask: 0b0000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 4],
        },
        size: 7,
        offset: 1,
        blacks: 0b0011100,
        whites: 0b1000001,
        blmask: 0b0000000,
        whmask: 0b0000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1, 4],
        },
        size: 8,
        offset: 2,
        blacks: 0b00110101,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 3],
        },
        size: 8,
        offset: 1,
        blacks: 0b10101100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b00000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 4],
        },
        size: 8,
        offset: 2,
        blacks: 0b00111001,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 4],
        },
        size: 8,
        offset: 1,
        blacks: 0b10011100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b00000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[2, 4],
        },
        size: 8,
        offset: 2,
        blacks: 0b00101101,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 2],
        },
        size: 8,
        offset: 1,
        blacks: 0b10110100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b00000001,
    },
    // actual three
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 4, 5],
        },
        size: 8,
        offset: 1,
        blacks: 0b00011100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 3, 5],
        },
        size: 8,
        offset: 1,
        blacks: 0b00101100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 2, 5],
        },
        size: 8,
        offset: 1,
        blacks: 0b00110100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 1, 5],
        },
        size: 8,
        offset: 1,
        blacks: 0b00111000,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
];

pub const BLACK_THREE_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[4],
        },
        size: 8,
        offset: 1,
        blacks: 0b00011100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[3],
        },
        size: 8,
        offset: 1,
        blacks: 0b00101100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[2],
        },
        size: 8,
        offset: 1,
        blacks: 0b00110100,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[1],
        },
        size: 8,
        offset: 1,
        blacks: 0b00111000,
        whites: 0b00000000,
        blmask: 0b00000000,
        whmask: 0b10000001,
    },
];

pub const BLACK_FOUR_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[4],
        },
        size: 7,
        offset: 1,
        blacks: 0b0011110,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[3],
        },
        size: 7,
        offset: 1,
        blacks: 0b0101110,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[2],
        },
        size: 7,
        offset: 1,
        blacks: 0b0110110,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1],
        },
        size: 7,
        offset: 1,
        blacks: 0b0111010,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0],
        },
        size: 7,
        offset: 1,
        blacks: 0b0111100,
        whites: 0b0000000,
        blmask: 0b0000000,
        whmask: 0b1000001,
    },
];

pub const BLACK_FIVE_PATTERNS: &[&RowPattern] = &[&RowPattern {
    row: &RowShape { size: 5, eyes: &[] },
    size: 7,
    offset: 1,
    blacks: 0b0111110,
    whites: 0b0000000,
    blmask: 0b0000000,
    whmask: 0b1000001,
}];

pub const BLACK_OVERLINE_PATTERNS: &[&RowPattern] = &[&RowPattern {
    row: &RowShape { size: 6, eyes: &[] },
    size: 6,
    offset: 0,
    blacks: 0b111111,
    whites: 0b000000,
    blmask: 0b000000,
    whmask: 0b000000,
}];

pub const WHITE_TWO_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[2, 3],
        },
        size: 6,
        offset: 1,
        blacks: 0b000000,
        whites: 0b000110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[1, 3],
        },
        size: 6,
        offset: 1,
        blacks: 0b000000,
        whites: 0b001010,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[0, 3],
        },
        size: 6,
        offset: 1,
        blacks: 0b000000,
        whites: 0b001100,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[1, 2],
        },
        size: 6,
        offset: 1,
        blacks: 0b000000,
        whites: 0b010010,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[0, 2],
        },
        size: 6,
        offset: 1,
        blacks: 0b000000,
        whites: 0b010100,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 4,
            eyes: &[0, 1],
        },
        size: 6,
        offset: 1,
        blacks: 0b000000,
        whites: 0b011000,
        blmask: 0b000000,
        whmask: 0b000000,
    },
];

pub const WHITE_SWORD_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[2, 3],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b10011,
        blmask: 0b00000,
        whmask: 0b00000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1, 3],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b10101,
        blmask: 0b00000,
        whmask: 0b00000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1, 2],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b11001,
        blmask: 0b00000,
        whmask: 0b00000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[3, 4],
        },
        size: 6,
        offset: 1,
        blacks: 0b000001,
        whites: 0b001110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[2, 4],
        },
        size: 6,
        offset: 1,
        blacks: 0b000001,
        whites: 0b010110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1, 4],
        },
        size: 6,
        offset: 1,
        blacks: 0b000001,
        whites: 0b011010,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 3],
        },
        size: 6,
        offset: 0,
        blacks: 0b100000,
        whites: 0b010110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 2],
        },
        size: 6,
        offset: 0,
        blacks: 0b100000,
        whites: 0b011010,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 1],
        },
        size: 6,
        offset: 0,
        blacks: 0b100000,
        whites: 0b011100,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0, 4],
        },
        size: 7,
        offset: 1,
        blacks: 0b1000001,
        whites: 0b0011100,
        blmask: 0b0000000,
        whmask: 0b0000000,
    },
    // actual three
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 4, 5],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b001110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 3, 5],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b010110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 2, 5],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b011010,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[0, 1, 5],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b011100,
        blmask: 0b000000,
        whmask: 0b000000,
    },
];

pub const WHITE_THREE_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[4],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b001110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[3],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b010110,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[2],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b011010,
        blmask: 0b000000,
        whmask: 0b000000,
    },
    &RowPattern {
        row: &RowShape {
            size: 6,
            eyes: &[1],
        },
        size: 6,
        offset: 0,
        blacks: 0b000000,
        whites: 0b011100,
        blmask: 0b000000,
        whmask: 0b000000,
    },
];

pub const WHITE_FOUR_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[4],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b01111,
        blmask: 0b00000,
        whmask: 0b00000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[3],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b10111,
        blmask: 0b00000,
        whmask: 0b00000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[2],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b11011,
        blmask: 0b00000,
        whmask: 0b00000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[1],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b11101,
        blmask: 0b00000,
        whmask: 0b00000,
    },
    &RowPattern {
        row: &RowShape {
            size: 5,
            eyes: &[0],
        },
        size: 5,
        offset: 0,
        blacks: 0b00000,
        whites: 0b11110,
        blmask: 0b00000,
        whmask: 0b00000,
    },
];

pub const WHITE_FIVE_PATTERNS: &[&RowPattern] = &[&RowPattern {
    row: &RowShape { size: 5, eyes: &[] },
    size: 5,
    offset: 0,
    blacks: 0b00000,
    whites: 0b11111,
    blmask: 0b00000,
    whmask: 0b00000,
}];
