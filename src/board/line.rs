use super::bits::Bits;
use super::row::*;

pub const MAX_LINE_LENGTH: u8 = 15;

#[derive(Clone)]
pub struct Line {
    pub size: u8,
    pub blacks: Bits,
    pub whites: Bits,

    row_checker: RowChecker,
}

impl Line {
    pub fn new(size: u8) -> Line {
        let size = std::cmp::min(size, MAX_LINE_LENGTH);
        Line {
            size: size,
            blacks: 0b0,
            whites: 0b0,

            row_checker: RowChecker::new(),
        }
    }

    pub fn put(&mut self, black: bool, i: u8) {
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

        self.row_checker.reset_free();
        self.row_checker
            .memoize_count(blacks, whites, self.blacks, self.whites);

        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn rows(&mut self, black: bool, kind: RowKind) -> Vec<Row> {
        if !self.row_checker.may_contain(self.size, black, kind) {
            return vec![];
        }

        let blacks_ = self.blacks << 1;
        let whites_ = self.whites << 1;
        let blanks_ = self.blanks() << 1;
        let limit = self.size + 2;

        let result = if black {
            scan_rows(true, kind, blacks_, blanks_, limit, 1)
        } else {
            scan_rows(false, kind, whites_, blanks_, limit, 1)
        };

        if result.is_empty() {
            self.row_checker.memoize_free(black, kind)
        }

        result
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

    fn blanks(&self) -> Bits {
        !(self.blacks | self.whites) & ((0b1 << self.size) - 1)
    }
}

#[derive(Clone)]
struct RowChecker {
    bfree: u8,
    wfree: u8,
    bcount: u8,
    wcount: u8,
}

impl RowChecker {
    pub fn new() -> RowChecker {
        RowChecker {
            bfree: 0b111111,
            wfree: 0b111111,
            bcount: 0,
            wcount: 0,
        }
    }

    pub fn memoize_free(&mut self, black: bool, kind: RowKind) {
        let mask = free_mask(kind);
        if black {
            self.bfree |= mask
        } else {
            self.wfree |= mask
        }
    }

    pub fn reset_free(&mut self) {
        self.bfree = 0b0;
        self.wfree = 0b0;
    }

    pub fn memoize_count(
        &mut self,
        blacks: Bits,
        whites: Bits,
        prev_blacks: Bits,
        prev_whites: Bits,
    ) {
        if blacks > prev_blacks {
            self.bcount += 1;
        } else if blacks < prev_blacks {
            self.bcount -= 1;
        }
        if whites > prev_whites {
            self.wcount += 1;
        } else if whites < prev_whites {
            self.wcount -= 1;
        }
    }

    pub fn may_contain(&self, size: u8, black: bool, kind: RowKind) -> bool {
        let mask = free_mask(kind);
        if (black && (self.bfree & mask != 0b0)) || (!black && (self.wfree & mask != 0b0)) {
            return false;
        }
        let min_stone_count = match kind {
            RowKind::Two => 2,
            RowKind::Sword => 3,
            RowKind::Three => 3,
            RowKind::Four => 4,
            RowKind::Five => 5,
            RowKind::Overline => 6,
        };
        let min_blank_count = match kind {
            RowKind::Two => 4,
            RowKind::Sword => 2,
            RowKind::Three => 3,
            RowKind::Four => 1,
            RowKind::Five => 0,
            RowKind::Overline => 0,
        };
        let blank_count = size - (self.bcount + self.wcount);
        blank_count >= min_blank_count
            && if black {
                self.bcount >= min_stone_count
            } else {
                self.wcount >= min_stone_count
            }
    }
}

fn free_mask(kind: RowKind) -> u8 {
    match kind {
        RowKind::Two => 0b000010,
        RowKind::Sword => 0b000001,
        RowKind::Three => 0b000100,
        RowKind::Four => 0b001000,
        RowKind::Five => 0b010000,
        RowKind::Overline => 0b100000,
    }
}
