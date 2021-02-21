use super::bits::*;
use super::pattern::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RowKind {
    Nothing,
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

impl RowKind {
    pub fn checker(&self, black: bool) -> Checker {
        if black {
            match self {
                RowKind::Two => Checker { b: 2, w: 0, n: 4 },
                RowKind::Sword => Checker { b: 3, w: 0, n: 2 },
                RowKind::Three => Checker { b: 3, w: 0, n: 3 },
                RowKind::Four => Checker { b: 4, w: 0, n: 1 },
                RowKind::Five => Checker { b: 5, w: 0, n: 0 },
                RowKind::Overline => Checker { b: 6, w: 0, n: 0 },
                RowKind::Nothing => Checker { b: 0, w: 0, n: 0 },
            }
        } else {
            match self {
                RowKind::Two => Checker { b: 0, w: 2, n: 4 },
                RowKind::Sword => Checker { b: 0, w: 3, n: 2 },
                RowKind::Three => Checker { b: 0, w: 3, n: 3 },
                RowKind::Four => Checker { b: 0, w: 4, n: 1 },
                RowKind::Five => Checker { b: 0, w: 5, n: 0 },
                RowKind::Overline => Checker { b: 0, w: 6, n: 0 },
                RowKind::Nothing => Checker { b: 0, w: 0, n: 0 },
            }
        }
    }
}

#[derive(Clone)]
pub struct Row {
    pub start: u8,
    pub end: u8,
    pub eye1: Option<u8>,
    pub eye2: Option<u8>,
}

#[derive(Clone, Copy)]
pub struct Checker {
    pub b: u8,
    pub w: u8,
    pub n: u8,
}

pub fn scan(
    black: bool,
    kind: RowKind,
    stones: Bits,
    blanks: Bits,
    limit: u8,
    offset: u8,
) -> Vec<Row> {
    if black {
        match kind {
            RowKind::Two => scan_patterns(&BLACK_TWOS, stones, blanks, limit, offset),
            RowKind::Sword => scan_patterns(&BLACK_SWORDS, stones, blanks, limit, offset),
            RowKind::Three => scan_patterns(&BLACK_THREES, stones, blanks, limit, offset),
            RowKind::Four => scan_patterns(&BLACK_FOURS, stones, blanks, limit, offset),
            RowKind::Five => scan_patterns(&BLACK_FIVES, stones, blanks, limit, offset),
            RowKind::Overline => scan_patterns(&BLACK_OVERLINES, stones, blanks, limit, offset),
            _ => vec![],
        }
    } else {
        match kind {
            RowKind::Two => scan_patterns(&WHITE_TWOS, stones, blanks, limit, offset),
            RowKind::Sword => scan_patterns(&WHITE_SWORDS, stones, blanks, limit, offset),
            RowKind::Three => scan_patterns(&WHITE_THREES, stones, blanks, limit, offset),
            RowKind::Four => scan_patterns(&WHITE_FOURS, stones, blanks, limit, offset),
            RowKind::Five => scan_patterns(&WHITE_FIVES, stones, blanks, limit, offset),
            _ => vec![],
        }
    }
}

fn scan_patterns(
    patterns: &[Pattern],
    stones: Bits,
    blanks: Bits,
    limit: u8,
    offset: u8,
) -> Vec<Row> {
    let mut result = vec![];
    for p in patterns {
        let size = p.size();
        if limit < size {
            continue;
        }
        let start = p.start();
        let end = p.end();
        let eye1 = p.eye1();
        let eye2 = p.eye2();
        for i in 0..=(limit - size) {
            if p.matches(stones >> i, blanks >> i) {
                result.push(Row {
                    start: start + i - offset,
                    end: end + i - offset,
                    eye1: eye1.map(|e| e + i - offset),
                    eye2: eye2.map(|e| e + i - offset),
                });
            }
        }
    }
    result
}
