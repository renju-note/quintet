use super::fundamentals::*;
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

pub use Direction::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Point(pub u8, pub u8);

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Index {
    pub direction: Direction,
    pub i: u8,
    pub j: u8,
}

#[derive(Debug)]
pub struct Points(pub Vec<Point>);

impl Point {
    pub fn to_index(&self, direction: Direction) -> Index {
        let (x, y) = (self.0, self.1);
        let n = BOARD_SIZE - 1;
        match direction {
            Vertical => Index::new(Vertical, x, y),
            Horizontal => Index::new(Horizontal, y, x),
            Ascending => {
                let i = x + n - y;
                let j = if i < n { x } else { y };
                Index::new(Ascending, i, j)
            }
            Descending => {
                let i = x + y;
                let j = if i < n { x } else { n - y };
                Index::new(Descending, i, j)
            }
        }
    }
}

impl Index {
    pub fn new(direction: Direction, i: u8, j: u8) -> Self {
        Self {
            direction: direction,
            i: i,
            j: j,
        }
    }

    pub fn to_point(&self) -> Point {
        let n = BOARD_SIZE - 1;
        let (i, j) = (self.i, self.j);
        match self.direction {
            Vertical => Point(i, j),
            Horizontal => Point(j, i),
            Ascending => {
                let x = if i < n { j } else { i + j - n };
                let y = if i < n { n - i + j } else { j };
                Point(x, y)
            }
            Descending => {
                let x = if i < n { j } else { i + j - n };
                let y = if i < n { i - j } else { n - j };
                Point(x, y)
            }
        }
    }

    pub fn walk(&self, step: i8) -> Self {
        let j = self.j as i8 + step;
        Self::new(self.direction, self.i, (self.j as i8 + step) as u8)
    }

    pub fn subsequence<'a>(&self, steps: &'a [u8]) -> impl Iterator<Item = Self> + 'a {
        let start = *self;
        steps.iter().map(move |&s| start.walk(s as i8))
    }

    pub fn maxj(&self) -> u8 {
        let n = BOARD_SIZE - 1;
        let i = self.i;
        match self.direction {
            Vertical | Horizontal => n,
            Ascending | Descending => {
                if i < n {
                    i
                } else {
                    2 * n - i
                }
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
        let mut cs = s.trim().chars();
        let x = cs
            .next()
            .map(|c| match c {
                'A'..='O' => Some(c as u8 - 'A' as u8),
                'a'..='o' => Some(c as u8 - 'a' as u8),
                _ => None,
            })
            .flatten()
            .ok_or("Failed to parse x part.")?;
        let y = cs
            .take(2)
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

impl Points {
    pub fn into_vec(self) -> Vec<Point> {
        self.0
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
        f.write_str(&result)
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
    use super::*;

    #[test]
    fn test_to_index_and_to_point() {
        fn assert_eq_point_index(p: Point, iv: Index, ih: Index, ia: Index, id: Index) {
            assert_eq!(p.to_index(Vertical), iv);
            assert_eq!(p.to_index(Horizontal), ih);
            assert_eq!(p.to_index(Ascending), ia);
            assert_eq!(p.to_index(Descending), id);
            assert_eq!(iv.to_point(), p);
            assert_eq!(ih.to_point(), p);
            assert_eq!(ia.to_point(), p);
            assert_eq!(id.to_point(), p);
        }

        // lower-left quadrant
        let p = Point(3, 6);
        let iv = Index::new(Vertical, 3, 6);
        let ih = Index::new(Horizontal, 6, 3);
        let ia = Index::new(Ascending, 11, 3);
        let id = Index::new(Descending, 9, 3);
        assert_eq_point_index(p, iv, ih, ia, id);

        // lower-right quadrant
        let p = Point(9, 6);
        let iv = Index::new(Vertical, 9, 6);
        let ih = Index::new(Horizontal, 6, 9);
        let ia = Index::new(Ascending, 17, 6);
        let id = Index::new(Descending, 15, 8);
        assert_eq_point_index(p, iv, ih, ia, id);

        // upper-left quadrant
        let p = Point(3, 12);
        let iv = Index::new(Vertical, 3, 12);
        let ih = Index::new(Horizontal, 12, 3);
        let ia = Index::new(Ascending, 5, 3);
        let id = Index::new(Descending, 15, 2);
        assert_eq_point_index(p, iv, ih, ia, id);

        // upper-right quadrant
        let p = Point(9, 12);
        let iv = Index::new(Vertical, 9, 12);
        let ih = Index::new(Horizontal, 12, 9);
        let ia = Index::new(Ascending, 11, 9);
        let id = Index::new(Descending, 21, 2);
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
