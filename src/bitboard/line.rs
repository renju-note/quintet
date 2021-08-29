use super::bits::*;
use super::row::*;
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Line {
    pub size: u8,
    pub blacks: Bits,
    pub whites: Bits,
    checker: RowChecker,
}

impl Line {
    pub fn new(size: u8) -> Line {
        let size = std::cmp::min(size, BOARD_SIZE);
        Line {
            size: size,
            blacks: 0b0,
            whites: 0b0,
            checker: RowChecker::new(),
        }
    }

    pub fn put(&mut self, player: Player, i: u8) {
        let stones = 0b1 << i;
        let blacks: Bits;
        let whites: Bits;
        match player {
            Player::Black => {
                blacks = self.blacks | stones;
                whites = self.whites & !stones;
            }
            Player::White => {
                blacks = self.blacks & !stones;
                whites = self.whites | stones;
            }
        };

        self.checker.reset_free();
        self.checker
            .memoize_count(blacks, whites, self.blacks, self.whites);

        self.blacks = blacks;
        self.whites = whites;
    }

    pub fn rows(&mut self, player: Player, kind: RowKind) -> Vec<Row> {
        if !self.checker.may_contain(self.size, player, kind) {
            return vec![];
        }

        let blacks_ = self.blacks << 1;
        let whites_ = self.whites << 1;
        let blanks_ = self.blanks() << 1;
        let limit = self.size + 2;

        let result = match player {
            Player::Black => scan_rows(Player::Black, kind, blacks_, blanks_, limit, 1),
            Player::White => scan_rows(Player::White, kind, whites_, blanks_, limit, 1),
        };

        if result.is_empty() {
            self.checker.memoize_free(player, kind)
        }

        result
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

    fn blanks(&self) -> Bits {
        !(self.blacks | self.whites) & ((0b1 << self.size) - 1)
    }
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.blacks == other.blacks && self.whites == other.whites
    }
}

impl Eq for Line {}

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
        write!(f, "{}", s)
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
            let i = i as u8;
            match c {
                'o' => line.put(Player::Black, i),
                'x' => line.put(Player::White, i),
                _ => (),
            }
        }
        Ok(line)
    }
}

#[derive(Clone, Debug)]
struct RowChecker {
    bfree: u8,
    wfree: u8,
    bcount: u8,
    wcount: u8,
}

impl RowChecker {
    pub fn new() -> RowChecker {
        RowChecker {
            bfree: 0b111111,
            wfree: 0b111111,
            bcount: 0,
            wcount: 0,
        }
    }

    pub fn memoize_free(&mut self, player: Player, kind: RowKind) {
        let mask = free_mask(kind);
        match player {
            Player::Black => {
                self.bfree |= mask;
            }
            Player::White => {
                self.wfree |= mask;
            }
        };
    }

    pub fn reset_free(&mut self) {
        self.bfree = 0b0;
        self.wfree = 0b0;
    }

    pub fn memoize_count(
        &mut self,
        blacks: Bits,
        whites: Bits,
        prev_blacks: Bits,
        prev_whites: Bits,
    ) {
        if blacks > prev_blacks {
            self.bcount += 1;
        } else if blacks < prev_blacks {
            self.bcount -= 1;
        }
        if whites > prev_whites {
            self.wcount += 1;
        } else if whites < prev_whites {
            self.wcount -= 1;
        }
    }

    pub fn may_contain(&self, size: u8, player: Player, kind: RowKind) -> bool {
        let mask = free_mask(kind);
        if (player.is_black() && (self.bfree & mask != 0b0))
            || (player.is_white() && (self.wfree & mask != 0b0))
        {
            return false;
        }
        let min_stone_count = match kind {
            RowKind::Two => 2,
            RowKind::Sword => 3,
            RowKind::Three => 3,
            RowKind::Four => 4,
            RowKind::Five => 5,
            RowKind::Overline => 6,
        };
        let min_blank_count = match kind {
            RowKind::Two => 4,
            RowKind::Sword => 2,
            RowKind::Three => 3,
            RowKind::Four => 1,
            RowKind::Five => 0,
            RowKind::Overline => 0,
        };
        let blank_count = size - (self.bcount + self.wcount);
        blank_count >= min_blank_count
            && match player {
                Player::Black => self.bcount >= min_stone_count,
                Player::White => self.wcount >= min_stone_count,
            }
    }
}

fn free_mask(kind: RowKind) -> u8 {
    match kind {
        RowKind::Two => 0b000010,
        RowKind::Sword => 0b000001,
        RowKind::Three => 0b000100,
        RowKind::Four => 0b001000,
        RowKind::Five => 0b010000,
        RowKind::Overline => 0b100000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "-o---x----".parse::<Line>()?;
        let mut expected = Line::new(10);
        expected.put(Player::Black, 1);
        expected.put(Player::White, 5);
        assert_eq!(result, expected);
        Ok(())
    }
}
