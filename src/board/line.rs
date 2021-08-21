use super::row::*;

const MAX_SIZE: u8 = 15;

#[derive(Clone)]
pub struct Line {
    pub size: u8,
    pub blacks: Bits,
    pub whites: Bits,

    bcount: u8,
    wcount: u8,
}

#[derive(Clone, Copy)]
pub struct Checker {
    pub b: u8,
    pub w: u8,
}

impl Line {
    pub fn new(size: u8) -> Line {
        let size = std::cmp::min(size, MAX_SIZE);
        Line {
            size: size,
            blacks: 0b0,
            whites: 0b0,

            bcount: 0,
            wcount: 0,
        }
    }

    pub fn check(&self, checker: Checker) -> bool {
        self.bcount >= checker.b && self.wcount >= checker.w
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

        if blacks > self.blacks {
            self.bcount += 1;
        } else if blacks < self.blacks {
            self.bcount -= 1;
        }
        if whites > self.whites {
            self.wcount += 1;
        } else if whites < self.whites {
            self.wcount -= 1;
        }

        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn rows(&self, black: bool, kind: RowKind) -> Vec<Row> {
        let blacks_ = self.blacks << 1;
        let whites_ = self.whites << 1;
        let blanks_ = self.blanks() << 1;
        let limit = self.size + 2;

        if black {
            scan_rows(true, kind, blacks_, blanks_, limit, 1)
        } else {
            scan_rows(false, kind, whites_, blanks_, limit, 1)
        }
    }

    fn blanks(&self) -> Bits {
        !(self.blacks | self.whites) & ((0b1 << self.size) - 1)
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

pub fn get_checker(kind: RowKind, black: bool) -> Checker {
    if black {
        match kind {
            RowKind::Two => Checker { b: 2, w: 0 },
            RowKind::Sword => Checker { b: 3, w: 0 },
            RowKind::Three => Checker { b: 3, w: 0 },
            RowKind::Four => Checker { b: 4, w: 0 },
            RowKind::Five => Checker { b: 5, w: 0 },
            RowKind::Overline => Checker { b: 6, w: 0 },
            _ => Checker { b: 0, w: 0 },
        }
    } else {
        match kind {
            RowKind::Two => Checker { b: 0, w: 2 },
            RowKind::Sword => Checker { b: 0, w: 3 },
            RowKind::Three => Checker { b: 0, w: 3 },
            RowKind::Four => Checker { b: 0, w: 4 },
            RowKind::Five => Checker { b: 0, w: 5 },
            _ => Checker { b: 0, w: 0 },
        }
    }
}
