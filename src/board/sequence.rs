use super::fundamentals::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Sequence {
    pub start: u8,
    pub end: u8,
    pub eye1: Option<u8>,
    pub eye2: Option<u8>,
}

impl Sequence {
    pub fn new(start: u8, end: u8, eye1: Option<u8>, eye2: Option<u8>) -> Sequence {
        Sequence {
            start: start,
            end: end,
            eye1: eye1,
            eye2: eye2,
        }
    }

    pub fn scan(
        player: Player,
        kind: RowKind,
        stones: Bits,
        blanks: Bits,
        limit: u8,
        offset: u8,
    ) -> Vec<Sequence> {
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
}

fn scan(
    window: &Window,
    patterns: &[Pattern],
    stones: Bits,
    blanks: Bits,
    limit: u8,
    offset: u8,
) -> Vec<Sequence> {
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
            // let position = i - offset; <- overflow!
            result.push(Sequence {
                start: p.start + i - offset,
                end: p.end + i - offset,
                eye1: p.eye1.map(|e| e + i - offset),
                eye2: p.eye2.map(|e| e + i - offset),
            });
        }
    }
    result
}

struct Window {
    size: u8,
    target: Bits,
}

struct Pattern {
    filter: Bits,
    stones: Bits,
    blanks: Bits,
    start: u8,
    end: u8,
    eye1: Option<u8>,
    eye2: Option<u8>,
}

impl Window {
    pub fn satisfies(&self, stones: Bits, blanks: Bits) -> bool {
        self.target & (stones | blanks) == self.target
    }
}

impl Pattern {
    pub fn matches(&self, stones: Bits, blanks: Bits) -> bool {
        (stones & self.filter == self.stones) && (blanks & self.filter & self.blanks == self.blanks)
    }
}

#[cfg(test)]
mod tests {
    use super::Player::*;
    use super::RowKind::*;
    use super::*;

    #[test]
    fn test_scan() {
        let stones = 0b0011100;
        let blanks = 0b1100010;
        let result = Sequence::scan(White, Three, stones, blanks, 7, 0);
        let expected = [Sequence::new(2, 5, Some(5), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Three, stones, blanks, 5, 0);
        let expected = [];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Three, stones, blanks, 7, 1);
        let expected = [Sequence::new(1, 4, Some(4), None)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_black_two() {
        let result = Sequence::scan(Black, Two, 0b00001100, 0b01110010, 8, 0);
        let expected = [Sequence::new(2, 5, Some(4), Some(5))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Two, 0b00010100, 0b01101010, 8, 0);
        let expected = [Sequence::new(2, 5, Some(3), Some(5))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Two, 0b00011000, 0b01100110, 8, 0);
        let expected = [Sequence::new(2, 5, Some(2), Some(5))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Two, 0b00100100, 0b01011010, 8, 0);
        let expected = [Sequence::new(2, 5, Some(3), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Two, 0b00101000, 0b01010110, 8, 0);
        let expected = [Sequence::new(2, 5, Some(2), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Two, 0b00110000, 0b01001110, 8, 0);
        let expected = [Sequence::new(2, 5, Some(2), Some(3))];
        assert_eq!(result, expected);

        // two twos
        let result = Sequence::scan(Black, Two, 0b000101000, 0b111010111, 9, 0);
        let expected = [
            Sequence::new(2, 5, Some(2), Some(4)),
            Sequence::new(3, 6, Some(4), Some(6)),
        ];
        assert_eq!(result, expected);

        // two twos
        let result = Sequence::scan(Black, Two, 0b00100100100, 0b01011011010, 11, 0);
        let expected = [
            Sequence::new(2, 5, Some(3), Some(4)),
            Sequence::new(5, 8, Some(6), Some(7)),
        ];
        assert_eq!(result, expected);

        // not two
        let result = Sequence::scan(Black, Two, 0b00011100, 0b00100010, 8, 0);
        let expected = [];
        assert_eq!(result, expected);

        // not two (overline)
        let result = Sequence::scan(Black, Two, 0b100101001, 0b011010110, 9, 0);
        let expected = [];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_black_three() {
        let result = Sequence::scan(Black, Three, 0b00011100, 0b01100010, 8, 0);
        let expected = [Sequence::new(2, 5, Some(5), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Three, 0b00101100, 0b01010010, 8, 0);
        let expected = [Sequence::new(2, 5, Some(4), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Three, 0b00110100, 0b01001010, 8, 0);
        let expected = [Sequence::new(2, 5, Some(3), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Three, 0b00111000, 0b01000110, 8, 0);
        let expected = [Sequence::new(2, 5, Some(2), None)];
        assert_eq!(result, expected);

        // two threes
        let result = Sequence::scan(White, Three, 0b000111000, 0b111000111, 9, 0);
        let expected = [
            Sequence::new(2, 5, Some(2), None),
            Sequence::new(3, 6, Some(6), None),
        ];
        assert_eq!(result, expected);

        // overline
        let result = Sequence::scan(Black, Three, 0b0010110100, 0b1101001011, 10, 0);
        let expected = [];
        assert_eq!(result, expected);

        // overline
        let result = Sequence::scan(Black, Three, 0b100111001, 0b011000110, 9, 0);
        let expected = [];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_black_sword() {
        let result = Sequence::scan(Black, Sword, 0b0001110, 0b0110000, 7, 0);
        let expected = [Sequence::new(1, 5, Some(4), Some(5))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0010110, 0b0101000, 7, 0);
        let expected = [Sequence::new(1, 5, Some(3), Some(5))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0011010, 0b0100100, 7, 0);
        let expected = [Sequence::new(1, 5, Some(2), Some(5))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0011100, 0b0100010, 7, 0);
        let expected = [Sequence::new(1, 5, Some(1), Some(5))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0100110, 0b0011000, 7, 0);
        let expected = [Sequence::new(1, 5, Some(3), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0101010, 0b0010100, 7, 0);
        let expected = [Sequence::new(1, 5, Some(2), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0101100, 0b0010010, 7, 0);
        let expected = [Sequence::new(1, 5, Some(1), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0110010, 0b0001100, 7, 0);
        let expected = [Sequence::new(1, 5, Some(2), Some(3))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0110100, 0b0001010, 7, 0);
        let expected = [Sequence::new(1, 5, Some(1), Some(3))];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Sword, 0b0111000, 0b0000110, 7, 0);
        let expected = [Sequence::new(1, 5, Some(1), Some(2))];
        assert_eq!(result, expected);

        // multiple
        let result = Sequence::scan(Black, Sword, 0b00011100, 0b11100011, 8, 0);
        let expected = [
            Sequence::new(1, 5, Some(1), Some(5)),
            Sequence::new(2, 6, Some(5), Some(6)),
        ];
        assert_eq!(result, expected);

        // maybe overline
        let result = Sequence::scan(Black, Sword, 0b1001110, 0b0110001, 7, 0);
        let expected = [];
        assert_eq!(result, expected);

        // not overline
        let result = Sequence::scan(Black, Sword, 0b10011100, 0b01100010, 8, 0);
        let expected = [Sequence::new(1, 5, Some(1), Some(5))];
        assert_eq!(result, expected);

        // multiple
        let result = Sequence::scan(Black, Sword, 0b0010110100, 0b1101001011, 10, 0);
        let expected = [
            Sequence::new(1, 5, Some(1), Some(3)),
            Sequence::new(4, 8, Some(6), Some(8)),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_black_four() {
        let result = Sequence::scan(Black, Four, 0b0011110, 0b0100000, 7, 0);
        let expected = [Sequence::new(1, 5, Some(5), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Four, 0b0101110, 0b0010000, 7, 0);
        let expected = [Sequence::new(1, 5, Some(4), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Four, 0b0110110, 0b0001000, 7, 0);
        let expected = [Sequence::new(1, 5, Some(3), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Four, 0b0111010, 0b0000100, 7, 0);
        let expected = [Sequence::new(1, 5, Some(2), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Four, 0b0111100, 0b00000010, 7, 0);
        let expected = [Sequence::new(1, 5, Some(1), None)];
        assert_eq!(result, expected);

        // open four
        let result = Sequence::scan(Black, Four, 0b00111100, 0b01000010, 8, 0);
        let expected = [
            Sequence::new(1, 5, Some(1), None),
            Sequence::new(2, 6, Some(6), None),
        ];
        assert_eq!(result, expected);

        // not open four
        let result = Sequence::scan(Black, Four, 0b00111100, 0b00000010, 8, 0);
        let expected = [Sequence::new(1, 5, Some(1), None)];
        assert_eq!(result, expected);

        // not open four
        let result = Sequence::scan(Black, Four, 0b00111100, 0b01000000, 8, 0);
        let expected = [Sequence::new(2, 6, Some(6), None)];
        assert_eq!(result, expected);

        // not open four (overline)
        let result = Sequence::scan(Black, Four, 0b10111100, 0b01000010, 8, 0);
        let expected = [Sequence::new(1, 5, Some(1), None)];
        assert_eq!(result, expected);

        // not open four (overline)
        let result = Sequence::scan(Black, Four, 0b00111101, 0b01000010, 8, 0);
        let expected = [Sequence::new(2, 6, Some(6), None)];
        assert_eq!(result, expected);

        // not four (overline)
        let result = Sequence::scan(Black, Four, 0b10111101, 0b01000010, 8, 0);
        let expected = [];
        assert_eq!(result, expected);

        // not four (overline)
        let result = Sequence::scan(Black, Four, 0b01110110, 0b10001001, 8, 0);
        let expected = [];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_black_five() {
        let result = Sequence::scan(Black, Five, 0b0111110, 0b0000000, 7, 0);
        let expected = [Sequence::new(1, 5, None, None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Five, 0b0111110, 0b0000000, 6, 0);
        let expected = [];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Five, 0b1111110, 0b0000000, 7, 0);
        let expected = [];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_black_overline() {
        let result = Sequence::scan(Black, Overline, 0b111111, 0b000000, 6, 0);
        let expected = [Sequence::new(0, 5, None, None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Overline, 0b1111111, 0b0000000, 6, 0);
        let expected = [Sequence::new(0, 5, None, None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(Black, Overline, 0b1111111, 0b0000000, 7, 0);
        let expected = [
            Sequence::new(0, 5, None, None),
            Sequence::new(1, 6, None, None),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_white_two() {
        let result = Sequence::scan(White, Two, 0b000110, 0b111001, 6, 0);
        let expected = [Sequence::new(1, 4, Some(3), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Two, 0b001010, 0b110101, 6, 0);
        let expected = [Sequence::new(1, 4, Some(2), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Two, 0b001100, 0b110011, 6, 0);
        let expected = [Sequence::new(1, 4, Some(1), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Two, 0b010010, 0b101101, 6, 0);
        let expected = [Sequence::new(1, 4, Some(2), Some(3))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Two, 0b010100, 0b101011, 6, 0);
        let expected = [Sequence::new(1, 4, Some(1), Some(3))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Two, 0b011000, 0b100111, 6, 0);
        let expected = [Sequence::new(1, 4, Some(1), Some(2))];
        assert_eq!(result, expected);

        // two twos
        let result = Sequence::scan(White, Two, 0b010010010, 0b101101101, 9, 0);
        let expected = [
            Sequence::new(1, 4, Some(2), Some(3)),
            Sequence::new(4, 7, Some(5), Some(6)),
        ];
        assert_eq!(result, expected);

        // not two
        let result = Sequence::scan(White, Two, 0b001110, 0b010001, 6, 0);
        let expected = [];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_white_three() {
        let result = Sequence::scan(White, Three, 0b001110, 0b110001, 6, 0);
        let expected = [Sequence::new(1, 4, Some(4), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Three, 0b010110, 0b101001, 6, 0);
        let expected = [Sequence::new(1, 4, Some(3), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Three, 0b011010, 0b100101, 6, 0);
        let expected = [Sequence::new(1, 4, Some(2), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Three, 0b011100, 0b100011, 6, 0);
        let expected = [Sequence::new(1, 4, Some(1), None)];
        assert_eq!(result, expected);

        // two threes
        let result = Sequence::scan(White, Three, 0b0011100, 0b1100011, 7, 0);
        let expected = [
            Sequence::new(1, 4, Some(1), None),
            Sequence::new(2, 5, Some(5), None),
        ];
        assert_eq!(result, expected);

        // not three
        let result = Sequence::scan(White, Three, 0b001110, 0b010001, 6, 0);
        let expected = [];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_white_sword() {
        let result = Sequence::scan(White, Sword, 0b00111, 0b11000, 5, 0);
        let expected = [Sequence::new(0, 4, Some(3), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b01011, 0b10100, 5, 0);
        let expected = [Sequence::new(0, 4, Some(2), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b01101, 0b10010, 5, 0);
        let expected = [Sequence::new(0, 4, Some(1), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b01110, 0b10001, 5, 0);
        let expected = [Sequence::new(0, 4, Some(0), Some(4))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b10011, 0b01100, 5, 0);
        let expected = [Sequence::new(0, 4, Some(2), Some(3))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b10101, 0b01010, 5, 0);
        let expected = [Sequence::new(0, 4, Some(1), Some(3))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b10110, 0b01001, 5, 0);
        let expected = [Sequence::new(0, 4, Some(0), Some(3))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b11001, 0b00110, 5, 0);
        let expected = [Sequence::new(0, 4, Some(1), Some(2))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b11010, 0b00101, 5, 0);
        let expected = [Sequence::new(0, 4, Some(0), Some(2))];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Sword, 0b11100, 0b00011, 5, 0);
        let expected = [Sequence::new(0, 4, Some(0), Some(1))];
        assert_eq!(result, expected);

        // multiple
        let result = Sequence::scan(White, Sword, 0b101011, 0b010100, 6, 0);
        let expected = [
            Sequence::new(0, 4, Some(2), Some(4)),
            Sequence::new(1, 5, Some(2), Some(4)),
        ];
        assert_eq!(result, expected);

        // multiple
        let result = Sequence::scan(White, Sword, 0b110011, 0b001100, 6, 0);
        let expected = [
            Sequence::new(0, 4, Some(2), Some(3)),
            Sequence::new(1, 5, Some(2), Some(3)),
        ];
        assert_eq!(result, expected);

        // multiple
        let result = Sequence::scan(White, Sword, 0b0011100, 0b1100011, 7, 0);
        let expected = [
            Sequence::new(0, 4, Some(0), Some(1)),
            Sequence::new(1, 5, Some(1), Some(5)),
            Sequence::new(2, 6, Some(5), Some(6)),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_white_four() {
        let result = Sequence::scan(White, Four, 0b01111, 0b10000, 5, 0);
        let expected = [Sequence::new(0, 4, Some(4), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Four, 0b10111, 0b01000, 5, 0);
        let expected = [Sequence::new(0, 4, Some(3), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Four, 0b11011, 0b00100, 5, 0);
        let expected = [Sequence::new(0, 4, Some(2), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Four, 0b11101, 0b00010, 5, 0);
        let expected = [Sequence::new(0, 4, Some(1), None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Four, 0b11110, 0b000001, 5, 0);
        let expected = [Sequence::new(0, 4, Some(0), None)];
        assert_eq!(result, expected);

        // open four
        let result = Sequence::scan(White, Four, 0b011110, 0b100001, 6, 0);
        let expected = [
            Sequence::new(0, 4, Some(0), None),
            Sequence::new(1, 5, Some(5), None),
        ];
        assert_eq!(result, expected);

        // not open four
        let result = Sequence::scan(White, Four, 0b011110, 0b000001, 6, 0);
        let expected = [Sequence::new(0, 4, Some(0), None)];
        assert_eq!(result, expected);

        // not open four
        let result = Sequence::scan(White, Four, 0b011110, 0b100000, 6, 0);
        let expected = [Sequence::new(1, 5, Some(5), None)];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_scan_white_five() {
        let result = Sequence::scan(White, Five, 0b11111, 0b00000, 5, 0);
        let expected = [Sequence::new(0, 4, None, None)];
        assert_eq!(result, expected);

        let result = Sequence::scan(White, Five, 0b111111, 0b00000, 6, 0);
        let expected = [
            Sequence::new(0, 4, None, None),
            Sequence::new(1, 5, None, None),
        ];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_satisfies() {
        let window = Window {
            size: 7,
            target: 0b0111110,
        };
        assert!(window.satisfies(0b0101010, 0b0010100));
        assert!(window.satisfies(0b0101011, 0b0010100));
        assert!(!window.satisfies(0b0101010, 0b0010000));
    }

    #[test]
    fn test_matches() {
        let pattern = Pattern {
            filter: 0b1111111,
            stones: 0b0101010,
            blanks: 0b0010100,
            start: 1,
            end: 5,
            eye1: Some(2),
            eye2: Some(4),
        };
        assert!(pattern.matches(0b0101010, 0b0010100));
        assert!(pattern.matches(0b0101010, 0b1010101));
        assert!(!pattern.matches(0b0101000, 0b0010100));
        assert!(!pattern.matches(0b0101001, 0b0010100));
    }
}

const B_TWO: Window = Window {
    size: 8,
    target: 0b01111110,
};

const B_TWOS: [Pattern; 6] = [
    Pattern {
        filter: 0b11111111,
        stones: 0b00001100,
        blanks: 0b01110010,
        start: 2,
        end: 5,
        eye1: Some(4),
        eye2: Some(5),
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00010100,
        blanks: 0b01101010,
        start: 2,
        end: 5,
        eye1: Some(3),
        eye2: Some(5),
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00011000,
        blanks: 0b01100110,
        start: 2,
        end: 5,
        eye1: Some(2),
        eye2: Some(5),
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00100100,
        blanks: 0b01011010,
        start: 2,
        end: 5,
        eye1: Some(3),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00101000,
        blanks: 0b01010110,
        start: 2,
        end: 5,
        eye1: Some(2),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00110000,
        blanks: 0b01001110,
        start: 2,
        end: 5,
        eye1: Some(2),
        eye2: Some(3),
    },
];

const B_THREE: Window = Window {
    size: 8,
    target: 0b01111110,
};

const B_THREES: [Pattern; 4] = [
    Pattern {
        filter: 0b11111111,
        stones: 0b00011100,
        blanks: 0b01100010,
        start: 2,
        end: 5,
        eye1: Some(5),
        eye2: None,
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00101100,
        blanks: 0b01010010,
        start: 2,
        end: 5,
        eye1: Some(4),
        eye2: None,
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00110100,
        blanks: 0b01001010,
        start: 2,
        end: 5,
        eye1: Some(3),
        eye2: None,
    },
    Pattern {
        filter: 0b11111111,
        stones: 0b00111000,
        blanks: 0b01000110,
        start: 2,
        end: 5,
        eye1: Some(2),
        eye2: None,
    },
];

const B_SWORD: Window = Window {
    size: 7,
    target: 0b0111110,
};

const B_SWORDS: [Pattern; 10] = [
    Pattern {
        filter: 0b1111111,
        stones: 0b0001110,
        blanks: 0b0110000,
        start: 1,
        end: 5,
        eye1: Some(4),
        eye2: Some(5),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0010110,
        blanks: 0b0101000,
        start: 1,
        end: 5,
        eye1: Some(3),
        eye2: Some(5),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0011010,
        blanks: 0b0100100,
        start: 1,
        end: 5,
        eye1: Some(2),
        eye2: Some(5),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0011100,
        blanks: 0b0100010,
        start: 1,
        end: 5,
        eye1: Some(1),
        eye2: Some(5),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0100110,
        blanks: 0b0011000,
        start: 1,
        end: 5,
        eye1: Some(3),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0101010,
        blanks: 0b0010100,
        start: 1,
        end: 5,
        eye1: Some(2),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0101100,
        blanks: 0b0010010,
        start: 1,
        end: 5,
        eye1: Some(1),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0110010,
        blanks: 0b0001100,
        start: 1,
        end: 5,
        eye1: Some(2),
        eye2: Some(3),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0110100,
        blanks: 0b0001010,
        start: 1,
        end: 5,
        eye1: Some(1),
        eye2: Some(3),
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0111000,
        blanks: 0b0000110,
        start: 1,
        end: 5,
        eye1: Some(1),
        eye2: Some(2),
    },
];

const B_FOUR: Window = Window {
    size: 7,
    target: 0b0111110,
};

const B_FOURS: [Pattern; 5] = [
    Pattern {
        filter: 0b1111111,
        stones: 0b0011110,
        blanks: 0b0100000,
        start: 1,
        end: 5,
        eye1: Some(5),
        eye2: None,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0101110,
        blanks: 0b0010000,
        start: 1,
        end: 5,
        eye1: Some(4),
        eye2: None,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0110110,
        blanks: 0b0001000,
        start: 1,
        end: 5,
        eye1: Some(3),
        eye2: None,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0111010,
        blanks: 0b0000100,
        start: 1,
        end: 5,
        eye1: Some(2),
        eye2: None,
    },
    Pattern {
        filter: 0b1111111,
        stones: 0b0111100,
        blanks: 0b0000010,
        start: 1,
        end: 5,
        eye1: Some(1),
        eye2: None,
    },
];

const B_FIVE: Window = Window {
    size: 7,
    target: 0b0111110,
};

const B_FIVES: [Pattern; 1] = [Pattern {
    filter: 0b1111111,
    stones: 0b0111110,
    blanks: 0b0000000,
    start: 1,
    end: 5,
    eye1: None,
    eye2: None,
}];

const B_OVERLINE: Window = Window {
    size: 6,
    target: 0b111111,
};

const B_OVERLINES: [Pattern; 1] = [Pattern {
    filter: 0b111111,
    stones: 0b111111,
    blanks: 0b000000,
    start: 0,
    end: 5,
    eye1: None,
    eye2: None,
}];

const W_TWO: Window = Window {
    size: 6,
    target: 0b111111,
};

const W_TWOS: [Pattern; 6] = [
    Pattern {
        filter: 0b111111,
        stones: 0b000110,
        blanks: 0b111001,
        start: 1,
        end: 4,
        eye1: Some(3),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b111111,
        stones: 0b001010,
        blanks: 0b110101,
        start: 1,
        end: 4,
        eye1: Some(2),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b111111,
        stones: 0b001100,
        blanks: 0b110011,
        start: 1,
        end: 4,
        eye1: Some(1),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b111111,
        stones: 0b010010,
        blanks: 0b101101,
        start: 1,
        end: 4,
        eye1: Some(2),
        eye2: Some(3),
    },
    Pattern {
        filter: 0b111111,
        stones: 0b010100,
        blanks: 0b101011,
        start: 1,
        end: 4,
        eye1: Some(1),
        eye2: Some(3),
    },
    Pattern {
        filter: 0b111111,
        stones: 0b011000,
        blanks: 0b100111,
        start: 1,
        end: 4,
        eye1: Some(1),
        eye2: Some(2),
    },
];

const W_THREE: Window = Window {
    size: 6,
    target: 0b111111,
};

const W_THREES: [Pattern; 4] = [
    Pattern {
        filter: 0b111111,
        stones: 0b001110,
        blanks: 0b110001,
        start: 1,
        end: 4,
        eye1: Some(4),
        eye2: None,
    },
    Pattern {
        filter: 0b111111,
        stones: 0b010110,
        blanks: 0b101001,
        start: 1,
        end: 4,
        eye1: Some(3),
        eye2: None,
    },
    Pattern {
        filter: 0b111111,
        stones: 0b011010,
        blanks: 0b100101,
        start: 1,
        end: 4,
        eye1: Some(2),
        eye2: None,
    },
    Pattern {
        filter: 0b111111,
        stones: 0b011100,
        blanks: 0b100011,
        start: 1,
        end: 4,
        eye1: Some(1),
        eye2: None,
    },
];

const W_SWORD: Window = Window {
    size: 5,
    target: 0b11111,
};

const W_SWORDS: [Pattern; 10] = [
    Pattern {
        filter: 0b11111,
        stones: 0b00111,
        blanks: 0b11000,
        start: 0,
        end: 4,
        eye1: Some(3),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b01011,
        blanks: 0b10100,
        start: 0,
        end: 4,
        eye1: Some(2),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b01101,
        blanks: 0b10010,
        start: 0,
        end: 4,
        eye1: Some(1),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b01110,
        blanks: 0b10001,
        start: 0,
        end: 4,
        eye1: Some(0),
        eye2: Some(4),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10011,
        blanks: 0b01100,
        start: 0,
        end: 4,
        eye1: Some(2),
        eye2: Some(3),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10101,
        blanks: 0b01010,
        start: 0,
        end: 4,
        eye1: Some(1),
        eye2: Some(3),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10110,
        blanks: 0b01001,
        start: 0,
        end: 4,
        eye1: Some(0),
        eye2: Some(3),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11001,
        blanks: 0b00110,
        start: 0,
        end: 4,
        eye1: Some(1),
        eye2: Some(2),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11010,
        blanks: 0b00101,
        start: 0,
        end: 4,
        eye1: Some(0),
        eye2: Some(2),
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11100,
        blanks: 0b00011,
        start: 0,
        end: 4,
        eye1: Some(0),
        eye2: Some(1),
    },
];

const W_FOUR: Window = Window {
    size: 5,
    target: 0b11111,
};

const W_FOURS: [Pattern; 5] = [
    Pattern {
        filter: 0b11111,
        stones: 0b01111,
        blanks: 0b10000,
        start: 0,
        end: 4,
        eye1: Some(4),
        eye2: None,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b10111,
        blanks: 0b01000,
        start: 0,
        end: 4,
        eye1: Some(3),
        eye2: None,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11011,
        blanks: 0b00100,
        start: 0,
        end: 4,
        eye1: Some(2),
        eye2: None,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11101,
        blanks: 0b00010,
        start: 0,
        end: 4,
        eye1: Some(1),
        eye2: None,
    },
    Pattern {
        filter: 0b11111,
        stones: 0b11110,
        blanks: 0b00001,
        start: 0,
        end: 4,
        eye1: Some(0),
        eye2: None,
    },
];

const W_FIVE: Window = Window {
    size: 5,
    target: 0b11111,
};

const W_FIVES: [Pattern; 1] = [Pattern {
    filter: 0b11111,
    stones: 0b11111,
    blanks: 0b00000,
    start: 0,
    end: 4,
    eye1: None,
    eye2: None,
}];
