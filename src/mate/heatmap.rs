use super::super::board::Direction::*;
use super::super::board::*;
use std::fmt;

const BOARD_SIZE_U: usize = BOARD_SIZE as usize;

const SLF: i8 = -1;
const OPN: i8 = -2;
const LIMIT: i8 = 5;

#[derive(Clone)]
pub struct Heatmap {
    map: [[i8; BOARD_SIZE_U]; BOARD_SIZE_U],
    player: Player,
}

impl Heatmap {
    pub fn new(player: Player) -> Self {
        Self {
            map: [[0; BOARD_SIZE_U]; BOARD_SIZE_U],
            player: player,
        }
    }

    pub fn put_mut(&mut self, player: Player, p: Point) {
        if player == self.player {
            self.put_mut_self(p)
        } else {
            self.put_mut_opponent(p)
        }
    }

    pub fn put(&self, player: Player, p: Point) -> Self {
        let mut result = self.clone();
        result.put_mut(player, p);
        result
    }

    fn put_mut_self(&mut self, p: Point) {
        for direction in [Vertical, Horizontal, Ascending, Descending] {
            for way in [Way::Up, Way::Down] {
                for n in Neighbors::new(p, direction, way) {
                    let h = self.get(n);
                    if h == SLF {
                        continue;
                    }
                    if h == OPN {
                        break;
                    }
                    self.inc(n);
                }
            }
        }
        self.set(p, SLF);
    }

    fn put_mut_opponent(&mut self, p: Point) {
        for direction in [Vertical, Horizontal, Ascending, Descending] {
            for way in [Way::Up, Way::Down] {
                for n in Neighbors::new(p, direction, way) {
                    let h = self.get(n);
                    if h == SLF {
                        for nn in Neighbors::new(n, direction, way.reflect()) {
                            let nh = self.get(nn);
                            if nh == SLF {
                                continue;
                            }
                            if nh == OPN {
                                break;
                            }
                            self.dec(nn)
                        }
                    }
                }
            }
        }
        self.set(p, OPN);
    }

    fn get(&self, p: Point) -> i8 {
        self.map[p.1 as usize][p.0 as usize]
    }

    fn set(&mut self, p: Point, h: i8) {
        self.map[p.1 as usize][p.0 as usize] = h
    }

    fn inc(&mut self, p: Point) {
        self.map[p.1 as usize][p.0 as usize] += 1
    }

    fn dec(&mut self, p: Point) {
        self.map[p.1 as usize][p.0 as usize] -= 1
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
                        if h > 0 {
                            h.to_string()
                        } else if h == SLF {
                            self.player.to_string()
                        } else if h == OPN {
                            self.player.opponent().to_string()
                        } else {
                            "-".to_string()
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

impl Way {
    pub fn reflect(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

struct Neighbors {
    sx: i8,
    sy: i8,
    direction: Direction,
    way: Way,
    c: i8,
}

impl Neighbors {
    fn new(p: Point, direction: Direction, way: Way) -> Self {
        Neighbors {
            sx: p.0 as i8,
            sy: p.1 as i8,
            direction: direction,
            way: way,
            c: 1,
        }
    }
}

impl Iterator for Neighbors {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let (sx, sy, c) = (self.sx, self.sy, self.c);
        if c >= LIMIT {
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
        Some(Point(x as u8, y as u8))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_mut() {
        let mut map = Heatmap::new(Player::Black);

        map.put_mut(Player::Black, Point(7, 7));
        let expected = trim_lines_string(
            "
            ---------------
            ---------------
            ---------------
            ---1---1---1---
            ----1--1--1----
            -----1-1-1-----
            ------111------
            ---1111o1111---
            ------111------
            -----1-1-1-----
            ----1--1--1----
            ---1---1---1---
            ---------------
            ---------------
            ---------------
        ",
        );
        assert_eq!(map.to_string(), expected);

        map.put_mut(Player::White, Point(8, 8));
        let expected = trim_lines_string(
            "
            ---------------
            ---------------
            ---------------
            ---1---1-------
            ----1--1-------
            -----1-1-------
            ------11x------
            ---1111o1111---
            ------111------
            -----1-1-1-----
            ----1--1--1----
            ---1---1---1---
            ---------------
            ---------------
            ---------------
        ",
        );
        assert_eq!(map.to_string(), expected);

        map.put_mut(Player::Black, Point(8, 6));
        let expected = trim_lines_string(
            "
            ---------------
            ---------------
            ---------------
            ---1---1-------
            ----2--1----1--
            -----2-1---1---
            ------21x-1----
            ---1111o2211---
            ----1122o1111--
            -----1-212-----
            ----1-111-2----
            ---1-1-11--2---
            ----1---1---1--
            ---------------
            ---------------
        ",
        );
        assert_eq!(map.to_string(), expected);

        map.put_mut(Player::White, Point(6, 8));
        let expected = trim_lines_string(
            "
            ---------------
            ---------------
            ---------------
            -------1-------
            -------1----1--
            -------1---1---
            ------x1x-1----
            ---1111o2211---
            ----1122o1111--
            -----1-212-----
            ----1-111-2----
            ---1-1-11--2---
            ----1---1---1--
            ---------------
            ---------------
        ",
        );
        assert_eq!(map.to_string(), expected);
    }

    #[test]
    fn test_put_mut_edge() {
        let mut map = Heatmap::new(Player::Black);
        map.put_mut(Player::Black, Point(0, 0));
        map.put_mut(Player::Black, Point(0, 14));
        map.put_mut(Player::Black, Point(14, 0));
        map.put_mut(Player::Black, Point(14, 14));

        let expected = trim_lines_string(
            "
            o1111-----1111o
            11-----------11
            1-1---------1-1
            1--1-------1--1
            1---1-----1---1
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            1---1-----1---1
            1--1-------1--1
            1-1---------1-1
            11-----------11
            o1111-----1111o
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
