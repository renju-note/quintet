use super::bits::*;
use super::row::*;

const MAX_SIZE: u8 = 15;

#[derive(Clone)]
pub struct Line {
    pub size: u8,
    pub blacks: Bits,
    pub whites: Bits,

    bcount: u8,
    wcount: u8,
    ncount: u8,

    bcache_kind: RowKind,
    wcache_kind: RowKind,
    bcache_rows: Vec<Row>,
    wcache_rows: Vec<Row>,
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
            ncount: size,

            bcache_kind: RowKind::Nothing,
            wcache_kind: RowKind::Nothing,
            bcache_rows: vec![],
            wcache_rows: vec![],
        }
    }

    pub fn check(&self, checker: Checker) -> bool {
        self.bcount >= checker.b && self.wcount >= checker.w && self.ncount >= checker.n
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
        if blacks > self.blacks && whites == self.whites
            || blacks == self.blacks && whites >= self.whites
        {
            self.ncount += 1;
        }

        if blacks != self.blacks {
            self.bcache_kind = RowKind::Nothing;
        }
        if whites != self.whites {
            self.wcache_kind = RowKind::Nothing;
        }

        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn rows(&mut self, black: bool, kind: RowKind) -> impl Iterator<Item = &Row> {
        if black && kind == self.bcache_kind {
            return self.bcache_rows.iter();
        } else if !black && kind == self.wcache_kind {
            return self.wcache_rows.iter();
        }

        let mut result = self.scan(black, kind);

        if black {
            self.bcache_kind = kind;
            self.bcache_rows.clear();
            self.bcache_rows.append(&mut result);
            self.bcache_rows.iter()
        } else {
            self.wcache_kind = kind;
            self.wcache_rows.clear();
            self.wcache_rows.append(&mut result);
            self.wcache_rows.iter()
        }
    }

    fn scan(&self, black: bool, kind: RowKind) -> Vec<Row> {
        let blacks_ = self.blacks << 1;
        let whites_ = self.whites << 1;
        let blanks_ = self.blanks() << 1;
        let limit = self.size + 2;

        if black {
            scan(true, kind, blacks_, blanks_, limit, 1)
        } else {
            scan(false, kind, whites_, blanks_, limit, 1)
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
