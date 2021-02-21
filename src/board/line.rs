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
    bcache_rows: LineRows,
    wcache_rows: LineRows,
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
            bcache_rows: LineRows::default(),
            wcache_rows: LineRows::default(),
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

    pub fn rows(&mut self, black: bool, kind: RowKind) -> LineRows {
        if black && kind == self.bcache_kind {
            return self.bcache_rows.clone();
        } else if !black && kind == self.wcache_kind {
            return self.wcache_rows.clone();
        }

        let trail = self.scan(black, kind);
        let result = LineRows::from(self.size, trail);

        if black {
            self.bcache_kind = kind;
            self.bcache_rows = result;
            self.bcache_rows.clone()
        } else {
            self.wcache_kind = kind;
            self.wcache_rows = result;
            self.wcache_rows.clone()
        }
    }

    fn scan(&self, black: bool, kind: RowKind) -> Trail {
        let blacks_ = self.blacks << 1;
        let whites_ = self.whites << 1;
        let blanks_ = self.blanks() << 1;
        let limit = self.size + 2;

        let mut result = if black {
            scan(true, kind, blacks_, blanks_, limit)
        } else {
            scan(false, kind, whites_, blanks_, limit)
        };
        result.rshift();
        result
    }

    pub fn check(&self, min_bcount: u8, min_wcount: u8, min_ncount: u8) -> bool {
        self.bcount >= min_bcount && self.wcount >= min_wcount && self.ncount >= min_ncount
    }

    pub fn blanks(&self) -> Bits {
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

#[derive(Clone, Default)]
pub struct LineRows {
    pub starts: Vec<u8>,
    pub eyes: Vec<u8>,
}

impl LineRows {
    pub fn from(size: u8, t: Trail) -> LineRows {
        let mut starts = vec![];
        for i in 0..size {
            if (t.starts >> i) & 0b1 == 0b1 {
                starts.push(i);
            }
        }
        let mut eyes = vec![];
        for i in 0..size {
            if (t.eyes >> i) & 0b1 == 0b1 {
                eyes.push(i);
            }
        }
        LineRows {
            starts: starts,
            eyes: eyes,
        }
    }
}
