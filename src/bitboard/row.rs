use super::bits::*;
use std::convert::From;

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

#[derive(Debug, Eq, PartialEq)]
pub struct Row {
    pub start: u8,
    pub end: u8,
    pub eye1: Option<u8>,
    pub eye2: Option<u8>,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    pub fn is_black(self) -> bool {
        self == Player::Black
    }

    pub fn is_white(self) -> bool {
        self == Player::White
    }
}

impl From<bool> for Player {
    fn from(value: bool) -> Player {
        if value {
            Player::Black
        } else {
            Player::White
        }
    }
}

impl Row {
    fn new(start: u8, end: u8, eye1: Option<u8>, eye2: Option<u8>) -> Row {
        Row {
            start: start,
            end: end,
            eye1: eye1,
            eye2: eye2,
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
        if !window.satisfies(stones, blanks) {
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

#[cfg(test)]
mod tests {
    use super::Player::*;
    use super::RowKind::*;
    use super::*;

    #[test]
    fn test_scan_rows_limit_offset() {
        let stones = 0b0011100;
        let blanks = 0b1100010;
        let result = scan_rows(White, Three, stones, blanks, 7, 0);
        let expected = [Row::new(1, 6, Some(5), None)];
        assert_eq!(result, expected);

        let result = scan_rows(White, Three, stones, blanks, 5, 0);
        let expected = [];
        assert_eq!(result, expected);

        let result = scan_rows(White, Three, stones, blanks, 7, 1);
        let expected = [Row::new(0, 5, Some(4), None)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_rows_black_five() {
        let result = scan_rows(Black, Five, 0b0111110, 0b0000000, 7, 0);
        let expected = [Row::new(1, 5, None, None)];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Five, 0b0111110, 0b0000000, 6, 0);
        let expected = [];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Five, 0b1111110, 0b0000000, 7, 0);
        let expected = [];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_rows_black_overline() {
        let result = scan_rows(Black, Overline, 0b111111, 0b000000, 6, 0);
        let expected = [Row::new(0, 5, None, None)];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Overline, 0b1111111, 0b0000000, 6, 0);
        let expected = [Row::new(0, 5, None, None)];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Overline, 0b1111111, 0b0000000, 7, 0);
        let expected = [Row::new(0, 5, None, None), Row::new(1, 6, None, None)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_rows_white_five() {
        let result = scan_rows(White, Five, 0b11111, 0b00000, 5, 0);
        let expected = [Row::new(0, 4, None, None)];
        assert_eq!(result, expected);

        let result = scan_rows(White, Five, 0b111111, 0b00000, 6, 0);
        let expected = [Row::new(0, 4, None, None), Row::new(1, 5, None, None)];
        assert_eq!(result, expected);
    }
}
