use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

pub const RANGE: u8 = 15;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Vertical,
    Horizontal,
    Ascending,
    Descending,
}

pub use Direction::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Point(pub u8, pub u8);

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = char::from_u32(('A' as u8 + self.0) as u32).unwrap();
        let y = self.1 + 1;
        write!(f, "{}{}", x, y)
    }
}

impl Point {
    pub fn to_index(&self, d: Direction) -> Index {
        let (x, y) = (self.0, self.1);
        let n = RANGE - 1;
        match d {
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
                1..=RANGE => Some(n - 1),
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
        let x = value / RANGE;
        let y = value % RANGE;
        if x < RANGE && y < RANGE {
            Ok(Point(x, y))
        } else {
            Err("Invalid code")
        }
    }
}

impl From<Point> for u8 {
    fn from(value: Point) -> u8 {
        value.0 * RANGE + value.1
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Index {
    pub d: Direction,
    pub i: u8,
    pub j: u8,
}

impl Index {
    pub fn new(d: Direction, i: u8, j: u8) -> Self {
        Self { d: d, i: i, j: j }
    }

    pub fn to_point(&self) -> Point {
        let n = RANGE - 1;
        let (i, j) = (self.i, self.j);
        match self.d {
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

    pub fn walk(&self, step: u8) -> Self {
        Self::new(self.d, self.i, (self.j + step) as u8)
    }

    pub fn walk_checked(&self, step: i8) -> Option<Self> {
        let j = self.j as i8 + step;
        if 0 <= j && j <= self.maxj() as i8 {
            Some(Self::new(self.d, self.i, j as u8))
        } else {
            None
        }
    }

    pub fn mapped<'a>(&self, steps: &'a [u8]) -> impl Iterator<Item = Self> + 'a {
        let start = *self;
        steps.iter().map(move |&s| start.walk(s))
    }

    pub fn maxj(&self) -> u8 {
        let n = RANGE - 1;
        let i = self.i;
        match self.d {
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

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Index::new({:?}, {}, {})", self.d, self.i, self.j)
    }
}

#[derive(Debug)]
pub struct Points(pub Vec<Point>);

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
