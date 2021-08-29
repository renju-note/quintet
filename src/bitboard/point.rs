use super::bits::*;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

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

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = char::from_u32(('A' as u8 + self.x) as u32).unwrap();
        let y = self.y + 1;
        write!(f, "{}{}", x, y)
    }
}

impl FromStr for Point {
    type Err = &'static str;

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
            .ok_or("Failed to parse x part.")?;
        let y = s
            .chars()
            .skip(1)
            .collect::<String>()
            .parse::<u8>()
            .ok()
            .map(|n| match n {
                1..=BOARD_SIZE => Some(n - 1),
                _ => None,
            })
            .flatten()
            .ok_or("Failed to parse y part.")?;
        Ok(Point { x: x, y: y })
    }
}

impl TryFrom<u8> for Point {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let x = value / BOARD_SIZE;
        let y = value % BOARD_SIZE;
        if is_valid(x) && is_valid(y) {
            Ok(Point { x: x, y: y })
        } else {
            Err("Invalid code")
        }
    }
}

impl From<Point> for u8 {
    fn from(value: Point) -> u8 {
        value.x * BOARD_SIZE + value.y
    }
}

const N: u8 = BOARD_SIZE - 1;

fn is_valid(n: u8) -> bool {
    (1..=BOARD_SIZE).contains(&n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_index_and_to_point() {
        fn assert_eq_point_index(p: Point, iv: Index, ih: Index, ia: Index, id: Index) {
            assert_eq!(p.to_index(Direction::Vertical), iv);
            assert_eq!(p.to_index(Direction::Horizontal), ih);
            assert_eq!(p.to_index(Direction::Ascending), ia);
            assert_eq!(p.to_index(Direction::Descending), id);
            assert_eq!(iv.to_point(Direction::Vertical), p);
            assert_eq!(ih.to_point(Direction::Horizontal), p);
            assert_eq!(ia.to_point(Direction::Ascending), p);
            assert_eq!(id.to_point(Direction::Descending), p);
        }

        // lower-left quadrant
        let p = Point { x: 3, y: 6 };
        let iv = Index { i: 3, j: 6 };
        let ih = Index { i: 6, j: 3 };
        let ia = Index { i: 11, j: 3 };
        let id = Index { i: 9, j: 3 };
        assert_eq_point_index(p, iv, ih, ia, id);
        // lower-right quadrant
        let p = Point { x: 9, y: 6 };
        let iv = Index { i: 9, j: 6 };
        let ih = Index { i: 6, j: 9 };
        let ia = Index { i: 17, j: 6 };
        let id = Index { i: 15, j: 8 };
        assert_eq_point_index(p, iv, ih, ia, id);
        // upper-left quadrant
        let p = Point { x: 3, y: 12 };
        let iv = Index { i: 3, j: 12 };
        let ih = Index { i: 12, j: 3 };
        let ia = Index { i: 5, j: 3 };
        let id = Index { i: 15, j: 2 };
        assert_eq_point_index(p, iv, ih, ia, id);
        // upper-right quadrant
        let p = Point { x: 9, y: 12 };
        let iv = Index { i: 9, j: 12 };
        let ih = Index { i: 12, j: 9 };
        let ia = Index { i: 11, j: 9 };
        let id = Index { i: 21, j: 2 };
        assert_eq_point_index(p, iv, ih, ia, id);
    }

    #[test]
    fn test_format() {
        let result = format!("{}", Point { x: 3, y: 5 });
        assert_eq!(result, "D6");
        let result = format!("{}", Point { x: 11, y: 10 });
        assert_eq!(result, "L11");
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "E2".parse::<Point>()?;
        assert_eq!(result, Point { x: 4, y: 1 });
        let result = "M15".parse::<Point>()?;
        assert_eq!(result, Point { x: 12, y: 14 });
        Ok(())
    }

    #[test]
    fn test_try_from_u8() -> Result<(), String> {
        let result = Point::try_from(72)?;
        assert_eq!(result, Point { x: 4, y: 12 });
        Ok(())
    }

    #[test]
    fn test_into_u8() {
        let result = u8::from(Point { x: 4, y: 12 });
        assert_eq!(result, 72);
    }
}
