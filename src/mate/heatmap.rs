use super::super::board::Direction::*;
use super::super::board::*;
use std::fmt;

const BOARD_SIZE_U: usize = BOARD_SIZE as usize;

#[derive(Default, Clone)]
pub struct Heatmap {
    map: [[i8; BOARD_SIZE_U]; BOARD_SIZE_U],
    limit: u8,
}

impl Heatmap {
    pub fn new(limit: u8) -> Self {
        let mut result = Self::default();
        result.limit = limit;
        result
    }

    pub fn put_mut(&mut self, p: Point) {
        let (x, y) = (p.0 as usize, p.1 as usize);
        self.map[y][x] = -1;
        for direction in [Vertical, Horizontal, Ascending, Descending] {
            for way in [Way::Up, Way::Down] {
                for (x, y) in Neighbors::new(p, direction, way, self.limit) {
                    if self.map[y][x] == -1 {
                        break;
                    }
                    self.map[y][x] += 1;
                }
            }
        }
    }

    pub fn put(&self, p: Point) -> Self {
        let mut result = self.clone();
        result.put_mut(p);
        result
    }
}

impl fmt::Display for Heatmap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result: String = self
            .map
            .iter()
            .rev()
            .map(|vmap| {
                vmap.into_iter()
                    .map(|&h| {
                        if h == 0 {
                            '-'
                        } else if h < 0 {
                            '*'
                        } else {
                            h.to_string().chars().last().unwrap()
                        }
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");
        f.write_str(&result)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Way {
    Up,
    Down,
}

struct Neighbors {
    sx: i8,
    sy: i8,
    direction: Direction,
    way: Way,
    limit: i8,
    c: i8,
}

impl Neighbors {
    fn new(p: Point, direction: Direction, way: Way, limit: u8) -> Self {
        Neighbors {
            sx: p.0 as i8,
            sy: p.1 as i8,
            direction: direction,
            way: way,
            limit: limit as i8,
            c: 1,
        }
    }
}

impl Iterator for Neighbors {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (sx, sy, c) = (self.sx, self.sy, self.c);
        if c > self.limit {
            return None;
        }
        let (x, y) = match (self.direction, self.way) {
            (Direction::Vertical, Way::Up) => (sx, sy + c),
            (Direction::Vertical, Way::Down) => (sx, sy - c),
            (Direction::Horizontal, Way::Up) => (sx + c, sy),
            (Direction::Horizontal, Way::Down) => (sx - c, sy),
            (Direction::Ascending, Way::Up) => (sx + c, sy + c),
            (Direction::Ascending, Way::Down) => (sx - c, sy - c),
            (Direction::Descending, Way::Up) => (sx + c, sy - c),
            (Direction::Descending, Way::Down) => (sx - c, sy + c),
        };
        if x < 0 || BOARD_SIZE <= x as u8 || y < 0 || BOARD_SIZE <= y as u8 {
            return None;
        }
        self.c = c + 1;
        Some((x as usize, y as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_mut() {
        let mut map = Heatmap::new(3);

        map.put_mut(Point(7, 7));
        let expected = trim_lines_string(
            "
            ---------------
            ---------------
            ---------------
            ---------------
            ----1--1--1----
            -----1-1-1-----
            ------111------
            ----111*111----
            ------111------
            -----1-1-1-----
            ----1--1--1----
            ---------------
            ---------------
            ---------------
            ---------------
        ",
        );
        assert_eq!(map.to_string(), expected);

        map.put_mut(Point(9, 8));
        let expected = trim_lines_string(
            "
            ---------------
            ---------------
            ---------------
            ------1--1--1--
            ----1--2-111---
            -----1-1121----
            ------222*111--
            ----111*222----
            ------1211-1---
            -----111-2--1--
            ----1--1--1----
            ---------------
            ---------------
            ---------------
            ---------------
        ",
        );
        assert_eq!(map.to_string(), expected);
    }

    #[test]
    fn test_put_mut_edge() {
        let mut map = Heatmap::new(3);
        map.put_mut(Point(0, 0));
        map.put_mut(Point(0, 14));
        map.put_mut(Point(14, 0));
        map.put_mut(Point(14, 14));

        let expected = trim_lines_string(
            "
            *111-------111*
            11-----------11
            1-1---------1-1
            1--1-------1--1
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            1--1-------1--1
            1-1---------1-1
            11-----------11
            *111-------111*
        ",
        );
        assert_eq!(map.to_string(), expected);
    }

    fn trim_lines_string(s: &str) -> String {
        s.trim()
            .split("\n")
            .map(|ls| ls.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
