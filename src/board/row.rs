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
                RowKind::Two => Checker { b: 2, w: 0 },
                RowKind::Sword => Checker { b: 3, w: 0 },
                RowKind::Three => Checker { b: 3, w: 0 },
                RowKind::Four => Checker { b: 4, w: 0 },
                RowKind::Five => Checker { b: 5, w: 0 },
                RowKind::Overline => Checker { b: 6, w: 0 },
                _ => Checker { b: 0, w: 0 },
            }
        } else {
            match self {
                RowKind::Two => Checker { b: 0, w: 2 },
                RowKind::Sword => Checker { b: 0, w: 3 },
                RowKind::Three => Checker { b: 0, w: 3 },
                RowKind::Four => Checker { b: 0, w: 4 },
                RowKind::Five => Checker { b: 0, w: 5 },
                _ => Checker { b: 0, w: 0 },
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
}

pub fn scan_rows(
    black: bool,
    kind: RowKind,
    stones: Bits,
    blanks: Bits,
    limit: u8,
    offset: u8,
) -> Vec<Row> {
    if black {
        match kind {
            RowKind::Two => scan(&B_TWO, &B_TWOS, stones, blanks, limit, offset),
            RowKind::Sword => scan(&B_SWORD, &B_SWORDS, stones, blanks, limit, offset),
            RowKind::Three => scan(&B_THREE, &B_THREES, stones, blanks, limit, offset),
            RowKind::Four => scan(&B_FOUR, &B_FOURS, stones, blanks, limit, offset),
            RowKind::Five => scan(&B_FIVE, &B_FIVES, stones, blanks, limit, offset),
            RowKind::Overline => scan(&B_OVERLINE, &B_OVERLINES, stones, blanks, limit, offset),
            _ => vec![],
        }
    } else {
        match kind {
            RowKind::Two => scan(&W_TWO, &W_TWOS, stones, blanks, limit, offset),
            RowKind::Sword => scan(&W_SWORD, &W_SWORDS, stones, blanks, limit, offset),
            RowKind::Three => scan(&W_THREE, &W_THREES, stones, blanks, limit, offset),
            RowKind::Four => scan(&W_FOUR, &W_FOURS, stones, blanks, limit, offset),
            RowKind::Five => scan(&W_FIVE, &W_FIVES, stones, blanks, limit, offset),
            _ => vec![],
        }
    }
}

fn scan(
    window: &Window,
    patterns: &[Pattern],
    stones: Bits,
    blanks: Bits,
    limit: u8,
    offset: u8,
) -> Vec<Row> {
    let mut result = vec![];
    let size = window.size;
    if limit < size {
        return result;
    }
    for i in 0..=(limit - size) {
        let stones = stones >> i;
        let blanks = blanks >> i;
        if !window.matches(stones, blanks) {
            continue;
        }
        for p in patterns {
            if !p.matches(stones, blanks) {
                continue;
            }
            result.push(Row {
                start: p.start() + i - offset,
                end: p.end() + i - offset,
                eye1: p.eye1().map(|e| e + i - offset),
                eye2: p.eye2().map(|e| e + i - offset),
            });
        }
    }
    result
}
