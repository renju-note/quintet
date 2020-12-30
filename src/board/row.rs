pub type Stones = u32;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

pub struct Row {
    pub start: u32,
    pub size: u32,
    pub eyes: Vec<u32>,
}

pub fn search_pattern(
    blacks: Stones,
    whites: Stones,
    within: u32,
    black: bool,
    kind: RowKind,
) -> Vec<Row> {
    match (black, kind) {
        (true, RowKind::Two) => search_multi(blacks, whites, within, BLACK_TWO_PATTERNS),
        (true, RowKind::Sword) => search_multi(blacks, whites, within, BLACK_SWORD_PATTERNS),
        (true, RowKind::Three) => search_multi(blacks, whites, within, BLACK_THREE_PATTERNS),
        (true, RowKind::Four) => search_multi(blacks, whites, within, BLACK_FOUR_PATTERNS),
        (true, RowKind::Five) => search_multi(blacks, whites, within, BLACK_FIVE_PATTERNS),
        (true, RowKind::Overline) => search_multi(blacks, whites, within, BLACK_OVERLINE_PATTERNS),
        (false, RowKind::Two) => search_multi(blacks, whites, within, WHITE_TWO_PATTERNS),
        (false, RowKind::Sword) => search_multi(blacks, whites, within, WHITE_SWORD_PATTERNS),
        (false, RowKind::Three) => search_multi(blacks, whites, within, WHITE_THREE_PATTERNS),
        (false, RowKind::Four) => search_multi(blacks, whites, within, WHITE_FOUR_PATTERNS),
        (false, RowKind::Five) => search_multi(blacks, whites, within, WHITE_FIVE_PATTERNS),
        _ => vec![],
    }
}

fn search_multi(blacks: Stones, whites: Stones, within: u32, patterns: &[&RowPattern]) -> Vec<Row> {
    patterns
        .into_iter()
        .flat_map(|p| search(blacks, whites, within, &p))
        .collect()
}

fn search(blacks: Stones, whites: Stones, within: u32, pattern: &RowPattern) -> Vec<Row> {
    let mut result = Vec::new();
    if within < pattern.size {
        return result;
    }
    let filter: Stones = (1 << pattern.size) - 1;
    let mut blacks = blacks;
    let mut whites = whites;
    for i in 0..=(within - pattern.size) {
        if (blacks & filter & !pattern.blmask) == pattern.blacks
            && (whites & filter & !pattern.whmask) == pattern.whites
        {
            let start = i + pattern.offset;
            let row = Row {
                start: start,
                size: pattern.row.size,
                eyes: pattern
                    .row
                    .eyes
                    .to_vec()
                    .into_iter()
                    .map(|eye| eye + start)
                    .collect(),
            };
            result.push(row);
        }
        blacks = blacks >> 1;
        whites = whites >> 1;
    }
    return result;
}

struct RowPattern<'a> {
    row: &'a RowShape<'a>,
    size: u32,
    offset: u32,
    blacks: Stones,
    blmask: Stones,
    whites: Stones,
    whmask: Stones,
}

struct RowShape<'a> {
    kind: RowKind,
    size: u32,
    eyes: &'a [u32],
}

const BLACK_TWO_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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

const BLACK_SWORD_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
            size: 5,
            eyes: &[3, 5],
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
            kind: RowKind::Sword,
            size: 5,
            eyes: &[2, 5],
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
            size: 5,
            eyes: &[1, 5],
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
];

const BLACK_THREE_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Three,
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
            kind: RowKind::Three,
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
            kind: RowKind::Three,
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
            kind: RowKind::Three,
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

const BLACK_FOUR_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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

const BLACK_FIVE_PATTERNS: &[&RowPattern] = &[&RowPattern {
    row: &RowShape {
        kind: RowKind::Five,
        size: 5,
        eyes: &[],
    },
    size: 7,
    offset: 1,
    blacks: 0b0111110,
    whites: 0b0000000,
    blmask: 0b0000000,
    whmask: 0b1000001,
}];

const BLACK_OVERLINE_PATTERNS: &[&RowPattern] = &[&RowPattern {
    row: &RowShape {
        kind: RowKind::Overline,
        size: 6,
        eyes: &[],
    },
    size: 6,
    offset: 0,
    blacks: 0b111111,
    whites: 0b000000,
    blmask: 0b000000,
    whmask: 0b000000,
}];

const WHITE_TWO_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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
            kind: RowKind::Two,
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

const WHITE_SWORD_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
            size: 5,
            eyes: &[3, 5],
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
            kind: RowKind::Sword,
            size: 5,
            eyes: &[2, 5],
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
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
            kind: RowKind::Sword,
            size: 5,
            eyes: &[1, 5],
        },
        size: 7,
        offset: 1,
        blacks: 0b1000001,
        whites: 0b0011100,
        blmask: 0b0000000,
        whmask: 0b0000000,
    },
];

const WHITE_THREE_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Three,
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
            kind: RowKind::Three,
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
            kind: RowKind::Three,
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
            kind: RowKind::Three,
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

const WHITE_FOUR_PATTERNS: &[&RowPattern] = &[
    &RowPattern {
        row: &RowShape {
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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
            kind: RowKind::Four,
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

const WHITE_FIVE_PATTERNS: &[&RowPattern] = &[&RowPattern {
    row: &RowShape {
        kind: RowKind::Five,
        size: 5,
        eyes: &[],
    },
    size: 5,
    offset: 0,
    blacks: 0b00000,
    whites: 0b11111,
    blmask: 0b00000,
    whmask: 0b00000,
}];
