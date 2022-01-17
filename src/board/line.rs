use super::fundamentals::*;
use super::sequence::*;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub size: u8,
    pub blacks: Bits,
    pub whites: Bits,
}

impl Line {
    pub fn new(size: u8) -> Line {
        let size = std::cmp::min(size, BOARD_SIZE);
        Line {
            size: size,
            blacks: 0b0,
            whites: 0b0,
        }
    }

    pub fn put_mut(&mut self, player: Player, i: u8) {
        let stones = 0b1 << i;
        let (blacks, whites) = match player {
            Player::Black => (self.blacks | stones, self.whites & !stones),
            Player::White => (self.blacks & !stones, self.whites | stones),
        };
        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn remove_mut(&mut self, i: u8) {
        let stones = 0b1 << i;
        self.blacks &= !stones;
        self.whites &= !stones;
    }

    pub fn put(&self, player: Player, i: u8) -> Self {
        let mut result = self.clone();
        result.put_mut(player, i);
        result
    }

    pub fn remove(&self, i: u8) -> Self {
        let mut result = self.clone();
        result.remove_mut(i);
        result
    }

    pub fn stone(&self, i: u8) -> Option<Player> {
        let pat = 0b1 << i;
        if self.blacks & pat != 0b0 {
            Some(Player::Black)
        } else if self.whites & pat != 0b0 {
            Some(Player::White)
        } else {
            None
        }
    }

    pub fn stones(&self, player: Player) -> Vec<u8> {
        let target = match player {
            Player::Black => self.blacks,
            Player::White => self.whites,
        };
        (0..self.size)
            .map(|i| {
                let pat = 0b1 << i;
                if target & pat != 0b0 {
                    Some(i)
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }

    pub fn segments(&self) -> Segments {
        Segments::new(self.blacks, self.whites, self.size)
    }

    pub fn sequences(&self, player: Player, kind: RowKind) -> Vec<Sequence> {
        if !self.may_contain(player, kind) {
            return vec![];
        }
        let offset = 1;
        let stones = match player {
            Player::Black => self.blacks << offset,
            Player::White => self.whites << offset,
        };
        let blanks = self.blanks() << offset;
        let limit = self.size + offset + offset;
        Sequence::scan(player, kind, stones, blanks, limit, offset)
    }

    pub fn sequence_eyes(&self, player: Player, kind: RowKind) -> Bits {
        if !self.may_contain(player, kind) {
            return 0b0;
        }
        let offset = 1;
        let stones = match player {
            Player::Black => self.blacks << offset,
            Player::White => self.whites << offset,
        };
        let blanks = self.blanks() << offset;
        let limit = self.size + offset + offset;
        Sequence::scan_eyes(player, kind, stones, blanks, limit, offset)
    }

    fn may_contain(&self, player: Player, kind: RowKind) -> bool {
        let min_stone = match kind {
            RowKind::Two => 2,
            RowKind::Sword => 3,
            RowKind::Three => 3,
            RowKind::Four => 4,
            RowKind::Five => 5,
            RowKind::Overline => 6,
        };
        let min_blank = match kind {
            RowKind::Two => 4,
            RowKind::Sword => 2,
            RowKind::Three => 3,
            RowKind::Four => 1,
            RowKind::Five => 0,
            RowKind::Overline => 0,
        };
        self.blanks().count_ones() >= min_blank
            && match player {
                Player::Black => self.blacks.count_ones() >= min_stone,
                Player::White => self.whites.count_ones() >= min_stone,
            }
    }

    fn blanks(&self) -> Bits {
        !(self.blacks | self.whites) & ((0b1 << self.size) - 1)
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s: String = (0..self.size)
            .map(|i| match self.stone(i) {
                Some(Player::Black) => 'o',
                Some(Player::White) => 'x',
                None => '-',
            })
            .collect();
        f.write_str(&s)
    }
}

impl FromStr for Line {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: Vec<char> = s.chars().collect();
        let size = chars.len();
        if size > BOARD_SIZE as usize {
            return Err("Wrong length.");
        }
        let mut line = Line::new(size as u8);
        for (i, c) in chars.into_iter().enumerate() {
            match c {
                'o' => line.put_mut(Player::Black, i as u8),
                'x' => line.put_mut(Player::White, i as u8),
                _ => (),
            }
        }
        Ok(line)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Segment(u8);

impl fmt::Debug for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Segment({:#010b})", self.0)
    }
}

pub struct Segments {
    blacks_: Bits,
    whites_: Bits,
    blanks_: Bits,
    size: u8,
    i: u8,
}

impl Segments {
    pub fn new(blacks: Bits, whites: Bits, limit: u8) -> Self {
        Self {
            blacks_: blacks << 1,
            whites_: whites << 1,
            blanks_: (!(blacks | whites) & ((0b1 << limit as u16) - 1)) << 1,
            size: limit - 5 + 1,
            i: 0,
        }
    }
}

impl Iterator for Segments {
    type Item = Segment;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        if i >= self.size {
            return None;
        }
        let blanks = self.blanks_ >> i & 0b0111110;
        let margin = self.blacks_ >> i & 0b1000001;
        let blacks = self.blacks_ >> i & 0b0111110;
        let whites = self.whites_ >> i & 0b0111110;
        let segment = if blanks == 0b0111110 {
            if margin == 0b0000000 {
                Segment(0b00000000)
            } else {
                Segment(0b10000000)
            }
        } else if blacks != 0b0 && whites == 0b0 && margin == 0b0 {
            Segment(0b00000000 | encode(blacks >> 1))
        } else if whites != 0b0 && blacks == 0b0 {
            Segment(0b10000000 | encode(whites >> 1))
        } else {
            Segment(0b00001111)
        };
        self.i += 1;
        return Some(segment);
    }
}

fn encode(stones: Bits) -> u8 {
    let count = stones.count_ones();
    let shape = match count {
        1 => stones.trailing_zeros(),
        2 => encode_shape(stones),
        3 => encode_shape(!stones & 0b11111),
        4 => (!stones & 0b11111).trailing_zeros(),
        _ => 0b0000,
    };
    ((count << 4) | shape) as u8
}

fn encode_shape(stones: Bits) -> u32 {
    let right = stones.trailing_zeros();
    let stones = stones & !(0b1 << right);
    let left = (stones >> 1).trailing_zeros();
    (left << 2) | right
}

#[cfg(test)]
mod tests {
    use super::Player::*;
    use super::RowKind::*;
    use super::*;

    #[test]
    fn test_new() {
        let result = Line::new(10);
        assert_eq!(result.size, 10);

        let result = Line::new(16);
        assert_eq!(result.size, BOARD_SIZE);
    }

    #[test]
    fn test_put_mut() {
        let mut line = Line::new(BOARD_SIZE);
        line.put_mut(Black, 0);
        line.put_mut(White, 2);
        assert_eq!(line.blacks, 0b000000000000001);
        assert_eq!(line.whites, 0b000000000000100);

        // overwrite
        line.put_mut(Black, 5);
        line.put_mut(White, 5);
        assert_eq!(line.blacks, 0b000000000000001);
        assert_eq!(line.whites, 0b000000000100100);
    }

    #[test]
    fn test_remove_mut() {
        let mut line = Line::new(BOARD_SIZE);
        line.put_mut(Black, 0);
        line.put_mut(White, 2);
        line.put_mut(Black, 4);
        line.put_mut(White, 5);
        line.remove_mut(0);
        line.remove_mut(2);
        line.remove_mut(3);
        assert_eq!(line.blacks, 0b000000000010000);
        assert_eq!(line.whites, 0b000000000100000);
    }

    #[test]
    fn test_stone() -> Result<(), String> {
        let line = "o-ox-".parse::<Line>()?;
        assert_eq!(line.stone(0), Some(Black));
        assert_eq!(line.stone(1), None);
        assert_eq!(line.stone(2), Some(Black));
        assert_eq!(line.stone(3), Some(White));
        assert_eq!(line.stone(4), None);
        Ok(())
    }

    #[test]
    fn test_stones() -> Result<(), String> {
        let line = "o-ox-".parse::<Line>()?;
        let result = line.stones(Black);
        let expected = [0, 2];
        assert_eq!(result, expected);
        let result = line.stones(White);
        let expected = [3];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_sequences() -> Result<(), String> {
        let line = "-oooo---x--x---".parse::<Line>()?;

        let result = line.sequences(Black, Four);
        let expected = [
            Sequence::new(0, 4, Some(0), None),
            Sequence::new(1, 5, Some(5), None),
        ];
        assert_eq!(result, expected);

        let result = line.sequences(White, Two);
        let expected = [Sequence::new(8, 11, Some(9), Some(10))];
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_to_string() {
        let mut line = Line::new(7);
        line.put_mut(Black, 0);
        line.put_mut(Black, 4);
        line.put_mut(White, 2);
        assert_eq!(line.to_string(), "o-x-o--");
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "-o---x----".parse::<Line>()?;
        let mut expected = Line::new(10);
        expected.put_mut(Black, 1);
        expected.put_mut(White, 5);
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_segments() {
        let result = Segments::new(0b000000001100000, 0b000000000000000, 15).collect::<Vec<_>>();
        let expected = [
            Segment(0b10000000),
            Segment(0b00001111),
            Segment(0b00101111),
            Segment(0b00101010),
            Segment(0b00100101),
            Segment(0b00100000),
            Segment(0b00001111),
            Segment(0b10000000),
            Segment(0b00000000),
            Segment(0b00000000),
            Segment(0b00000000),
        ];
        assert_eq!(result, expected);

        let result = Segments::new(0b000000000000000, 0b000000001100000, 15).collect::<Vec<_>>();
        let expected = [
            Segment(0b00000000),
            Segment(0b10010100),
            Segment(0b10101111),
            Segment(0b10101010),
            Segment(0b10100101),
            Segment(0b10100000),
            Segment(0b10010000),
            Segment(0b00000000),
            Segment(0b00000000),
            Segment(0b00000000),
            Segment(0b00000000),
        ];
        assert_eq!(result, expected);
    }
}
