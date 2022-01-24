use super::fundamentals::*;
use super::sequence::*;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Line {
    blacks: u16,
    whites: u16,
    pub size: u8,
}

impl Line {
    pub fn new(size: u8) -> Self {
        Self {
            blacks: 0b0,
            whites: 0b0,
            size: size,
        }
    }

    pub fn put_mut(&mut self, player: Player, i: u8) {
        let stones = 0b1 << i;
        let (blacks, whites) = match player {
            Black => (self.blacks | stones, self.whites & !stones),
            White => (self.blacks & !stones, self.whites | stones),
        };
        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn remove_mut(&mut self, i: u8) {
        let stones = 0b1 << i;
        self.blacks &= !stones;
        self.whites &= !stones;
    }

    pub fn stone(&self, i: u8) -> Option<Player> {
        let pat = 0b1 << i;
        if self.blacks & pat != 0b0 {
            Some(Black)
        } else if self.whites & pat != 0b0 {
            Some(White)
        } else {
            None
        }
    }

    pub fn stones(&self, player: Player) -> impl Iterator<Item = u8> {
        let target = match player {
            Black => self.blacks,
            White => self.whites,
        };
        (0..self.size).filter(move |i| target & (0b1 << i) != 0b0)
    }

    pub fn sequences(&self, player: Player, kind: SequenceKind, n: u8, exact: bool) -> Sequences {
        let (my, op) = if player.is_black() {
            (self.blacks, self.whites)
        } else {
            (self.whites, self.blacks)
        };
        Sequences::new(self.size, my, op, kind, n, exact)
    }

    pub fn sequences_on(
        &self,
        i: u8,
        player: Player,
        kind: SequenceKind,
        n: u8,
        exact: bool,
    ) -> Sequences {
        let (my, op) = if player.is_black() {
            (self.blacks, self.whites)
        } else {
            (self.whites, self.blacks)
        };
        Sequences::new_on(i, self.size, my, op, kind, n, exact)
    }

    pub fn potential_cap(&self, player: Player) -> u8 {
        let nstones_opponent = match player {
            Black => self.whites.count_ones(),
            White => self.blacks.count_ones(),
        } as u8;
        if self.size < nstones_opponent + 5 {
            return 0;
        }
        let nstones = match player {
            Black => self.blacks.count_ones(),
            White => self.whites.count_ones(),
        } as u8;
        nstones + 1
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ss = (0..self.size).map(|i| match self.stone(i) {
            Some(player) => format!(" {}", char::from(player)),
            None => " .".to_string(),
        });
        f.write_str(&ss.collect::<String>())
    }
}

impl FromStr for Line {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        let size = chars.len();
        if size > BOARD_SIZE as usize {
            return Err("Wrong length.");
        }
        let mut line = Self::new(size as u8);
        for (i, c) in chars.into_iter().enumerate() {
            Player::try_from(c).map_or((), |p| line.put_mut(p, i as u8));
        }
        Ok(line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let result = line.stones(Black).collect::<Vec<_>>();
        let expected = [0, 2];
        assert_eq!(result, expected);
        let result = line.stones(White).collect::<Vec<_>>();
        let expected = [3];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_sequences() -> Result<(), String> {
        let line = "o--o--o---o---o".parse::<Line>()?;
        let result = line.sequences(Black, Single, 2, true).collect::<Vec<_>>();
        let expected = [
            (0, Sequence(0b00001001)),
            (2, Sequence(0b00010010)),
            (3, Sequence(0b00001001)),
            (6, Sequence(0b00010001)),
            (10, Sequence(0b00010001)),
        ];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_sequences_on() -> Result<(), String> {
        let line = "o--o--o---o---o".parse::<Line>()?;
        let result: Vec<_> = line.sequences_on(7, Black, Single, 2, true).collect();
        let expected = [(3, Sequence(0b00001001)), (6, Sequence(0b00010001))];
        assert_eq!(result, expected);

        let line = "-----oo-o-o----".parse::<Line>()?;
        let result: Vec<_> = line.sequences_on(7, Black, Single, 3, false).collect();
        let expected = [
            (4, Sequence(0b00010110)),
            (5, Sequence(0b00001011)),
            (6, Sequence(0b00010101)),
        ];
        assert_eq!(result, expected);
        let result: Vec<_> = line.sequences_on(7, Black, Single, 3, true).collect();
        let expected = [(4, Sequence(0b00010110))];
        assert_eq!(result, expected);
        let result: Vec<_> = line.sequences_on(7, Black, Compact, 3, false).collect();
        let expected = [(5, Sequence(0b00011011))];
        assert_eq!(result, expected);
        let result: Vec<_> = line.sequences_on(7, Black, Compact, 3, true).collect();
        let expected = [];
        assert_eq!(result, expected);

        let line = "---ooo---ooo---".parse::<Line>()?;
        let result: Vec<_> = line.sequences_on(7, Black, Single, 3, false).collect();
        let expected = [(3, Sequence(0b00000111)), (7, Sequence(0b00011100))];
        assert_eq!(result, expected);
        let result: Vec<_> = line.sequences_on(7, Black, Single, 3, true).collect();
        let expected = [(3, Sequence(0b00000111)), (7, Sequence(0b00011100))];
        assert_eq!(result, expected);
        let result: Vec<_> = line.sequences_on(7, Black, Compact, 3, false).collect();
        let expected = [];
        assert_eq!(result, expected);
        let result: Vec<_> = line.sequences_on(7, Black, Compact, 3, true).collect();
        let expected = [];
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_potential_cap() -> Result<(), String> {
        let line = "-----".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 1);
        assert_eq!(line.potential_cap(White), 1);

        let line = "--o--".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 2);
        assert_eq!(line.potential_cap(White), 0);

        let line = "o----x".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 2);
        assert_eq!(line.potential_cap(White), 2);

        let line = "--o---".parse::<Line>()?;
        assert_eq!(line.potential_cap(Black), 2);
        assert_eq!(line.potential_cap(White), 1);

        Ok(())
    }

    #[test]
    fn test_to_string() {
        let mut line = Line::new(7);
        line.put_mut(Black, 0);
        line.put_mut(Black, 4);
        line.put_mut(White, 2);
        assert_eq!(line.to_string(), " o . x . o . .");
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = " . o . . . x . . . .".parse::<Line>()?;
        let mut expected = Line::new(10);
        expected.put_mut(Black, 1);
        expected.put_mut(White, 5);
        assert_eq!(result, expected);
        Ok(())
    }
}
