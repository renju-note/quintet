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
    bcache: Bits,
    wcache: Bits,
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
            bcache: 0b0,
            wcache: 0b0,
        }
    }

    pub fn put(&mut self, black: bool, i: u8) {
        let stones = 0b1 << i;
        let blacks: Bits;
        let whites: Bits;
        if black {
            blacks = self.blacks | stones;
            whites = self.whites & !stones;
            if self.blacks != blacks {
                self.blacks = blacks;
                self.bcount += 1;
                self.ncount -= 1;
                self.bcache_kind = RowKind::Nothing;
            }
            if self.whites != whites {
                self.whites = whites;
                self.wcount -= 1;
                self.ncount += 1;
                self.wcache_kind = RowKind::Nothing;
            }
        } else {
            blacks = self.blacks & !stones;
            whites = self.whites | stones;
            if self.blacks != blacks {
                self.blacks = blacks;
                self.bcount -= 1;
                self.ncount += 1;
                self.bcache_kind = RowKind::Nothing;
            }
            if self.whites != whites {
                self.whites = whites;
                self.wcount += 1;
                self.ncount -= 1;
                self.wcache_kind = RowKind::Nothing;
            }
        }
    }

    pub fn eyes(&mut self, black: bool, kind: RowKind, cache: bool) -> Vec<u8> {
        let eyes = if black && kind == self.bcache_kind {
            self.bcache
        } else if !black && kind == self.wcache_kind {
            self.wcache
        } else {
            self.scan_eyes(black, kind)
        };

        if cache {
            if black {
                self.bcache_kind = kind;
                self.bcache = eyes;
            } else {
                self.wcache_kind = kind;
                self.wcache = eyes;
            }
        }

        let mut result = vec![];
        for i in 0..self.size {
            if (eyes >> i) & 0b1 == 0b1 {
                result.push(i);
            }
        }
        result
    }

    fn scan_eyes(&self, black: bool, kind: RowKind) -> Bits {
        let blacks_ = self.blacks << 1;
        let whites_ = self.whites << 1;
        let blanks_ = self.blanks() << 1;
        let limit = self.size + 2;

        if black {
            scan_eyes(true, kind, blacks_, blanks_, limit) >> 1
        } else {
            scan_eyes(false, kind, whites_, blanks_, limit) >> 1
        }
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
