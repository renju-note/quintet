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
    pub fn new(start: u8, end: u8, eye1: Option<u8>, eye2: Option<u8>) -> Row {
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
    fn test_scan_rows() {
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
    fn test_scan_rows_black_sword() {
        let result = scan_rows(Black, Sword, 0b0001110, 0b0110000, 7, 0);
        let expected = [Row::new(1, 5, Some(4), Some(5))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0010110, 0b0101000, 7, 0);
        let expected = [Row::new(1, 5, Some(3), Some(5))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0011010, 0b0100100, 7, 0);
        let expected = [Row::new(1, 5, Some(2), Some(5))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0011100, 0b0100010, 7, 0);
        let expected = [Row::new(1, 5, Some(1), Some(5))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0100110, 0b0011000, 7, 0);
        let expected = [Row::new(1, 5, Some(3), Some(4))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0101010, 0b0010100, 7, 0);
        let expected = [Row::new(1, 5, Some(2), Some(4))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0101100, 0b0010010, 7, 0);
        let expected = [Row::new(1, 5, Some(1), Some(4))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0110010, 0b0001100, 7, 0);
        let expected = [Row::new(1, 5, Some(2), Some(3))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0110100, 0b0001010, 7, 0);
        let expected = [Row::new(1, 5, Some(1), Some(3))];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Sword, 0b0111000, 0b0000110, 7, 0);
        let expected = [Row::new(1, 5, Some(1), Some(2))];
        assert_eq!(result, expected);

        // multiple
        let result = scan_rows(Black, Sword, 0b00011100, 0b11100011, 8, 0);
        let expected = [
            Row::new(1, 5, Some(1), Some(5)),
            Row::new(2, 6, Some(5), Some(6)),
        ];
        assert_eq!(result, expected);

        // maybe overline
        let result = scan_rows(Black, Sword, 0b1001110, 0b0110001, 7, 0);
        let expected = [];
        assert_eq!(result, expected);

        // not overline
        let result = scan_rows(Black, Sword, 0b10011100, 0b01100010, 8, 0);
        let expected = [Row::new(1, 5, Some(1), Some(5))];
        assert_eq!(result, expected);

        // multiple
        let result = scan_rows(Black, Sword, 0b0010110100, 0b1101001011, 10, 0);
        let expected = [
            Row::new(1, 5, Some(1), Some(3)),
            Row::new(4, 8, Some(6), Some(8)),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_rows_black_four() {
        let result = scan_rows(Black, Four, 0b0011110, 0b0100000, 7, 0);
        let expected = [Row::new(1, 5, Some(5), None)];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Four, 0b0101110, 0b0010000, 7, 0);
        let expected = [Row::new(1, 5, Some(4), None)];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Four, 0b0110110, 0b0001000, 7, 0);
        let expected = [Row::new(1, 5, Some(3), None)];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Four, 0b0111010, 0b0000100, 7, 0);
        let expected = [Row::new(1, 5, Some(2), None)];
        assert_eq!(result, expected);

        let result = scan_rows(Black, Four, 0b0111100, 0b00000010, 7, 0);
        let expected = [Row::new(1, 5, Some(1), None)];
        assert_eq!(result, expected);

        // open four
        let result = scan_rows(Black, Four, 0b00111100, 0b01000010, 8, 0);
        let expected = [Row::new(1, 5, Some(1), None), Row::new(2, 6, Some(6), None)];
        assert_eq!(result, expected);

        // not open four
        let result = scan_rows(Black, Four, 0b00111100, 0b00000010, 8, 0);
        let expected = [Row::new(1, 5, Some(1), None)];
        assert_eq!(result, expected);

        // not open four
        let result = scan_rows(Black, Four, 0b00111100, 0b01000000, 8, 0);
        let expected = [Row::new(2, 6, Some(6), None)];
        assert_eq!(result, expected);

        // not open four (overline)
        let result = scan_rows(Black, Four, 0b10111100, 0b01000010, 8, 0);
        let expected = [Row::new(1, 5, Some(1), None)];
        assert_eq!(result, expected);

        // not open four (overline)
        let result = scan_rows(Black, Four, 0b00111101, 0b01000010, 8, 0);
        let expected = [Row::new(2, 6, Some(6), None)];
        assert_eq!(result, expected);

        // not four (overline)
        let result = scan_rows(Black, Four, 0b10111101, 0b01000010, 8, 0);
        let expected = [];
        assert_eq!(result, expected);

        // not four (overline)
        let result = scan_rows(Black, Four, 0b01110110, 0b10001001, 8, 0);
        let expected = [];
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
    fn test_scan_rows_white_sword() {
        let result = scan_rows(White, Sword, 0b00111, 0b11000, 5, 0);
        let expected = [Row::new(0, 4, Some(3), Some(4))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b01011, 0b10100, 5, 0);
        let expected = [Row::new(0, 4, Some(2), Some(4))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b01101, 0b10010, 5, 0);
        let expected = [Row::new(0, 4, Some(1), Some(4))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b01110, 0b10001, 5, 0);
        let expected = [Row::new(0, 4, Some(0), Some(4))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b10011, 0b01100, 5, 0);
        let expected = [Row::new(0, 4, Some(2), Some(3))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b10101, 0b01010, 5, 0);
        let expected = [Row::new(0, 4, Some(1), Some(3))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b10110, 0b01001, 5, 0);
        let expected = [Row::new(0, 4, Some(0), Some(3))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b11001, 0b00110, 5, 0);
        let expected = [Row::new(0, 4, Some(1), Some(2))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b11010, 0b00101, 5, 0);
        let expected = [Row::new(0, 4, Some(0), Some(2))];
        assert_eq!(result, expected);

        let result = scan_rows(White, Sword, 0b11100, 0b00011, 5, 0);
        let expected = [Row::new(0, 4, Some(0), Some(1))];
        assert_eq!(result, expected);

        // multiple
        let result = scan_rows(White, Sword, 0b101011, 0b010100, 6, 0);
        let expected = [
            Row::new(0, 4, Some(2), Some(4)),
            Row::new(1, 5, Some(2), Some(4)),
        ];
        assert_eq!(result, expected);

        // multiple
        let result = scan_rows(White, Sword, 0b110011, 0b001100, 6, 0);
        let expected = [
            Row::new(0, 4, Some(2), Some(3)),
            Row::new(1, 5, Some(2), Some(3)),
        ];
        assert_eq!(result, expected);

        // multiple
        let result = scan_rows(White, Sword, 0b0011100, 0b1100011, 7, 0);
        let expected = [
            Row::new(0, 4, Some(0), Some(1)),
            Row::new(1, 5, Some(1), Some(5)),
            Row::new(2, 6, Some(5), Some(6)),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_rows_white_four() {
        let result = scan_rows(White, Four, 0b01111, 0b10000, 5, 0);
        let expected = [Row::new(0, 4, Some(4), None)];
        assert_eq!(result, expected);

        let result = scan_rows(White, Four, 0b10111, 0b01000, 5, 0);
        let expected = [Row::new(0, 4, Some(3), None)];
        assert_eq!(result, expected);

        let result = scan_rows(White, Four, 0b11011, 0b00100, 5, 0);
        let expected = [Row::new(0, 4, Some(2), None)];
        assert_eq!(result, expected);

        let result = scan_rows(White, Four, 0b11101, 0b00010, 5, 0);
        let expected = [Row::new(0, 4, Some(1), None)];
        assert_eq!(result, expected);

        let result = scan_rows(White, Four, 0b11110, 0b000001, 5, 0);
        let expected = [Row::new(0, 4, Some(0), None)];
        assert_eq!(result, expected);

        // open four
        let result = scan_rows(White, Four, 0b011110, 0b100001, 6, 0);
        let expected = [Row::new(0, 4, Some(0), None), Row::new(1, 5, Some(5), None)];
        assert_eq!(result, expected);

        // not open four
        let result = scan_rows(White, Four, 0b011110, 0b000001, 6, 0);
        let expected = [Row::new(0, 4, Some(0), None)];
        assert_eq!(result, expected);

        // not open four
        let result = scan_rows(White, Four, 0b011110, 0b100000, 6, 0);
        let expected = [Row::new(1, 5, Some(5), None)];
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
