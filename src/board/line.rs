use super::fundamentals::*;
use std::cmp;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub blacks: Bits,
    pub whites: Bits,
    pub size: u8,
}

impl Line {
    pub fn new(size: u8) -> Line {
        let size = std::cmp::min(size, BOARD_SIZE);
        Line {
            blacks: 0b0,
            whites: 0b0,
            size: size,
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
        let blacks = self.blacks << 1;
        let whites = self.whites << 1;
        let start = 0;
        let end = self.size + 1;
        Segments::new(blacks, whites, start, end)
    }

    pub fn segments_on(&self, i: u8) -> Segments {
        let blacks = self.blacks << 1;
        let whites = self.whites << 1;
        let start = cmp::max(0, (i + 1) as i8 - (WINDOW_SIZE as i8 - 1)) as u8;
        let end = cmp::min(self.size + 1, (i + 1) + (WINDOW_SIZE - 1));
        Segments::new(blacks, whites, start, end)
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

const WINDOW_SIZE: u8 = 7;
const WINDOW_MASK: u16 = (0b1 << 7) - 1;

pub struct Segments {
    blacks: Bits,
    whites: Bits,
    limit: u8,
    i: u8,
}

impl Segments {
    pub fn new(blacks: Bits, whites: Bits, start: u8, end: u8) -> Self {
        Segments {
            blacks: blacks,
            whites: whites,
            limit: end - (WINDOW_SIZE - 1),
            i: start,
        }
    }
}

impl Iterator for Segments {
    type Item = (u8, u8, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.i;
        if i > self.limit {
            return None;
        }
        self.i += 1;
        Some((
            i,
            (self.blacks >> i & WINDOW_MASK) as u8,
            (self.whites >> i & WINDOW_MASK) as u8,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Player::*;
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
    fn test_segments() -> Result<(), String> {
        let line = "oo-----o-----xx".parse::<Line>()?;
        let result = line.segments().collect::<Vec<_>>();
        let expected = [
            (0, 0b0000110, 0b0000000),
            (1, 0b0000011, 0b0000000),
            (2, 0b1000001, 0b0000000),
            (3, 0b0100000, 0b0000000),
            (4, 0b0010000, 0b0000000),
            (5, 0b0001000, 0b0000000),
            (6, 0b0000100, 0b0000000),
            (7, 0b0000010, 0b0000000),
            (8, 0b0000001, 0b1000000),
            (9, 0b0000000, 0b1100000),
            (10, 0b0000000, 0b0110000),
        ];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_segments_on() -> Result<(), String> {
        let line = "oo-----o-----xx".parse::<Line>()?;
        let result = line.segments_on(7).collect::<Vec<_>>();
        let expected = [
            (2, 0b1000001, 0b0000000),
            (3, 0b0100000, 0b0000000),
            (4, 0b0010000, 0b0000000),
            (5, 0b0001000, 0b0000000),
            (6, 0b0000100, 0b0000000),
            (7, 0b0000010, 0b0000000),
            (8, 0b0000001, 0b1000000),
        ];
        assert_eq!(result, expected);

        let result = line.segments_on(2).collect::<Vec<_>>();
        let expected = [
            (0, 0b0000110, 0b0000000),
            (1, 0b0000011, 0b0000000),
            (2, 0b1000001, 0b0000000),
            (3, 0b0100000, 0b0000000),
        ];
        assert_eq!(result, expected);

        let result = line.segments_on(5).collect::<Vec<_>>();
        let expected = [
            (0, 0b0000110, 0b0000000),
            (1, 0b0000011, 0b0000000),
            (2, 0b1000001, 0b0000000),
            (3, 0b0100000, 0b0000000),
            (4, 0b0010000, 0b0000000),
            (5, 0b0001000, 0b0000000),
            (6, 0b0000100, 0b0000000),
        ];
        assert_eq!(result, expected);

        let result = line.segments_on(6).collect::<Vec<_>>();
        let expected = [
            (1, 0b0000011, 0b0000000),
            (2, 0b1000001, 0b0000000),
            (3, 0b0100000, 0b0000000),
            (4, 0b0010000, 0b0000000),
            (5, 0b0001000, 0b0000000),
            (6, 0b0000100, 0b0000000),
            (7, 0b0000010, 0b0000000),
        ];
        assert_eq!(result, expected);

        let result = line.segments_on(12).collect::<Vec<_>>();
        let expected = [
            (7, 0b0000010, 0b0000000),
            (8, 0b0000001, 0b1000000),
            (9, 0b0000000, 0b1100000),
            (10, 0b0000000, 0b0110000),
        ];
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
}
