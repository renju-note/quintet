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
pub struct Point(pub u8, pub u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Index(pub u8, pub u8);

pub struct Points(pub Vec<Point>);

impl Point {
    pub fn to_index(&self, direction: Direction) -> Index {
        let (x, y) = (self.0, self.1);
        let n = BOARD_SIZE - 1;
        match direction {
            Direction::Vertical => Index(x, y),
            Direction::Horizontal => Index(y, x),
            Direction::Ascending => {
                let i = x + n - y;
                let j = if i < n { x } else { y };
                Index(i, j)
            }
            Direction::Descending => {
                let i = x + y;
                let j = if i < n { x } else { n - y };
                Index(i, j)
            }
        }
    }
}

impl Index {
    pub fn to_point(&self, direction: Direction) -> Point {
        let (i, j) = (self.0, self.1);
        let n = BOARD_SIZE - 1;
        match direction {
            Direction::Vertical => Point(i, j),
            Direction::Horizontal => Point(j, i),
            Direction::Ascending => {
                let x = if i < n { j } else { i + j - n };
                let y = if i < n { n - i + j } else { j };
                Point(x, y)
            }
            Direction::Descending => {
                let x = if i < n { j } else { i + j - n };
                let y = if i < n { i - j } else { n - j };
                Point(x, y)
            }
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = char::from_u32(('A' as u8 + self.0) as u32).unwrap();
        let y = self.1 + 1;
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
        Ok(Point(x, y))
    }
}

impl TryFrom<u8> for Point {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let x = value / BOARD_SIZE;
        let y = value % BOARD_SIZE;
        if x < BOARD_SIZE && y < BOARD_SIZE {
            Ok(Point(x, y))
        } else {
            Err("Invalid code")
        }
    }
}

impl From<Point> for u8 {
    fn from(value: Point) -> u8 {
        value.0 * BOARD_SIZE + value.1
    }
}

impl fmt::Display for Points {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = self
            .0
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{}", result)
    }
}

impl FromStr for Points {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ps = s
            .trim()
            .split(",")
            .map(|m| m.parse::<Point>())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Points(ps))
    }
}

impl TryFrom<&[u8]> for Points {
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let ps = value
            .iter()
            .map(|c| Point::try_from(*c))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Points(ps))
    }
}

impl From<Points> for Vec<u8> {
    fn from(value: Points) -> Vec<u8> {
        value.0.into_iter().map(|p| u8::from(p)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Direction::*;
    use super::*;

    #[test]
    fn test_to_index_and_to_point() {
        fn assert_eq_point_index(p: Point, iv: Index, ih: Index, ia: Index, id: Index) {
            assert_eq!(p.to_index(Vertical), iv);
            assert_eq!(p.to_index(Horizontal), ih);
            assert_eq!(p.to_index(Ascending), ia);
            assert_eq!(p.to_index(Descending), id);
            assert_eq!(iv.to_point(Vertical), p);
            assert_eq!(ih.to_point(Horizontal), p);
            assert_eq!(ia.to_point(Ascending), p);
            assert_eq!(id.to_point(Descending), p);
        }

        // lower-left quadrant
        let p = Point(3, 6);
        let iv = Index(3, 6);
        let ih = Index(6, 3);
        let ia = Index(11, 3);
        let id = Index(9, 3);
        assert_eq_point_index(p, iv, ih, ia, id);

        // lower-right quadrant
        let p = Point(9, 6);
        let iv = Index(9, 6);
        let ih = Index(6, 9);
        let ia = Index(17, 6);
        let id = Index(15, 8);
        assert_eq_point_index(p, iv, ih, ia, id);

        // upper-left quadrant
        let p = Point(3, 12);
        let iv = Index(3, 12);
        let ih = Index(12, 3);
        let ia = Index(5, 3);
        let id = Index(15, 2);
        assert_eq_point_index(p, iv, ih, ia, id);

        // upper-right quadrant
        let p = Point(9, 12);
        let iv = Index(9, 12);
        let ih = Index(12, 9);
        let ia = Index(11, 9);
        let id = Index(21, 2);
        assert_eq_point_index(p, iv, ih, ia, id);
    }

    #[test]
    fn test_to_string() {
        let result = Point(3, 5).to_string();
        assert_eq!(result, "D6");

        let result = Point(11, 10).to_string();
        assert_eq!(result, "L11");
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "E2".parse::<Point>()?;
        assert_eq!(result, Point(4, 1));

        let result = "M15".parse::<Point>()?;
        assert_eq!(result, Point(12, 14));

        Ok(())
    }

    #[test]
    fn test_try_from_u8() -> Result<(), String> {
        let result = Point::try_from(72)?;
        assert_eq!(result, Point(4, 12));
        Ok(())
    }

    #[test]
    fn test_into_u8() {
        let result = u8::from(Point(4, 12));
        assert_eq!(result, 72);
    }

    #[test]
    fn test_points_to_string() {
        let ps = vec![Point(7, 7), Point(7, 8), Point(8, 8)];
        assert_eq!(Points(ps).to_string(), "H8,H9,I9");
    }
}
