use super::fundamentals::*;
use super::segment::*;
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

    pub fn put(&mut self, player: Player, i: u8) {
        let stones = 0b1 << i;
        let (blacks, whites) = match player {
            Player::Black => (self.blacks | stones, self.whites & !stones),
            Player::White => (self.blacks & !stones, self.whites | stones),
        };
        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn stones(&self) -> Vec<Option<Player>> {
        (0..self.size)
            .map(|i| {
                let pat = 0b1 << i;
                if self.blacks & pat != 0b0 {
                    Some(Player::Black)
                } else if self.whites & pat != 0b0 {
                    Some(Player::White)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn segments(&self, player: Player, kind: RowKind) -> Vec<Segment> {
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
        Segment::scan(player, kind, stones, blanks, limit, offset)
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
        let s: String = self
            .stones()
            .iter()
            .map(|s| match s {
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
                'o' => line.put(Player::Black, i as u8),
                'x' => line.put(Player::White, i as u8),
                _ => (),
            }
        }
        Ok(line)
    }
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
    fn test_put() {
        let mut line = Line::new(BOARD_SIZE);
        line.put(Black, 0);
        line.put(White, 2);
        assert_eq!(line.blacks, 0b000000000000001);
        assert_eq!(line.whites, 0b000000000000100);

        // overwrite
        line.put(Black, 5);
        line.put(White, 5);
        assert_eq!(line.blacks, 0b000000000000001);
        assert_eq!(line.whites, 0b000000000100100);
    }

    #[test]
    fn test_stones() -> Result<(), String> {
        let line = "o-ox-".parse::<Line>()?;
        let result = line.stones();
        let expected = [Some(Black), None, Some(Black), Some(White), None];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_segments() -> Result<(), String> {
        let line = "-oooo---x--x---".parse::<Line>()?;

        let result = line.segments(Black, Four);
        let expected = [
            Segment::new(0, 4, Some(0), None),
            Segment::new(1, 5, Some(5), None),
        ];
        assert_eq!(result, expected);

        let result = line.segments(White, Two);
        let expected = [Segment::new(7, 12, Some(9), Some(10))];
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_to_string() {
        let mut line = Line::new(7);
        line.put(Black, 0);
        line.put(Black, 4);
        line.put(White, 2);
        assert_eq!(line.to_string(), "o-x-o--");
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "-o---x----".parse::<Line>()?;
        let mut expected = Line::new(10);
        expected.put(Black, 1);
        expected.put(White, 5);
        assert_eq!(result, expected);
        Ok(())
    }
}
