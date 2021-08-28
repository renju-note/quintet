use super::bits::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Player {
    Black,
    White,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RowKind {
    Two,
    Sword,
    Three,
    Four,
    Five,
    Overline,
}

#[derive(Clone)]
pub struct Row {
    pub start: u8,
    pub end: u8,
    pub eye1: Option<u8>,
    pub eye2: Option<u8>,
}

impl Player {
    pub fn new(black: bool) -> Player {
        if black {
            Player::Black
        } else {
            Player::White
        }
    }

    pub fn opponent(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }
}

pub fn scan_rows(
    player: Player,
    kind: RowKind,
    stones: Bits,
    blanks: Bits,
    limit: u8,
    offset: u8,
) -> Vec<Row> {
    match player {
        Player::Black => match kind {
            RowKind::Two => scan(&B_TWO, &B_TWOS, stones, blanks, limit, offset),
            RowKind::Sword => scan(&B_SWORD, &B_SWORDS, stones, blanks, limit, offset),
            RowKind::Three => scan(&B_THREE, &B_THREES, stones, blanks, limit, offset),
            RowKind::Four => scan(&B_FOUR, &B_FOURS, stones, blanks, limit, offset),
            RowKind::Five => scan(&B_FIVE, &B_FIVES, stones, blanks, limit, offset),
            RowKind::Overline => scan(&B_OVERLINE, &B_OVERLINES, stones, blanks, limit, offset),
        },
        Player::White => match kind {
            RowKind::Two => scan(&W_TWO, &W_TWOS, stones, blanks, limit, offset),
            RowKind::Sword => scan(&W_SWORD, &W_SWORDS, stones, blanks, limit, offset),
            RowKind::Three => scan(&W_THREE, &W_THREES, stones, blanks, limit, offset),
            RowKind::Four => scan(&W_FOUR, &W_FOURS, stones, blanks, limit, offset),
            RowKind::Five => scan(&W_FIVE, &W_FIVES, stones, blanks, limit, offset),
            _ => vec![],
        },
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
