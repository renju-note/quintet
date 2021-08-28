use super::bits::*;
use std::fmt;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Index {
    pub i: u8,
    pub j: u8,
}

impl Point {
    pub fn to_index(&self, direction: Direction) -> Index {
        let (x, y) = (self.x, self.y);
        match direction {
            Direction::Vertical => Index { i: x, j: y },
            Direction::Horizontal => Index { i: y, j: x },
            Direction::Ascending => {
                let i = x + N - y;
                let j = if i < N { x } else { y };
                Index { i: i, j: j }
            }
            Direction::Descending => {
                let i = x + y;
                let j = if i < N { x } else { N - y };
                Index { i: i, j: j }
            }
        }
    }
}

impl Index {
    pub fn to_point(&self, direction: Direction) -> Point {
        let (i, j) = (self.i, self.j);
        match direction {
            Direction::Vertical => Point { x: i, y: j },
            Direction::Horizontal => Point { x: j, y: i },
            Direction::Ascending => {
                let x = if i < N { j } else { i + j - N };
                let y = if i < N { N - i + j } else { j };
                Point { x: x, y: y }
            }
            Direction::Descending => {
                let x = if i < N { j } else { i + j - N };
                let y = if i < N { i - j } else { N - j };
                Point { x: x, y: y }
            }
        }
    }
}

impl FromStr for Point {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let x = s
            .chars()
            .nth(0)
            .map(|c| match c {
                'A'..='O' => Some(c as u8 - 'A' as u8),
                'a'..='o' => Some(c as u8 - 'a' as u8),
                _ => None,
            })
            .flatten()
            .ok_or("Failed to parse x part.".to_string())?;
        let y = s
            .chars()
            .skip(1)
            .collect::<String>()
            .parse::<u8>()
            .ok()
            .map(|n| match n {
                1..=15 => Some(n - 1),
                _ => None,
            })
            .flatten()
            .ok_or("Failed to parse y part.".to_string())?;
        Ok(Point { x: x, y: y })
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let xs = char::from_u32(('A' as u8 + self.x) as u32)
            .unwrap()
            .to_string();
        let ys = (self.y + 1).to_string();
        write!(f, "{}{}", xs, ys)
    }
}

const N: u8 = BOARD_SIZE - 1;
