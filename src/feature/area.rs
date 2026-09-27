use crate::board::{POINT_COUNT, Point, SIZE};
use std::ops::BitOrAssign;

/// A set of points, one bit per point by its `u8` code (`x * SIZE + y`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Area([u64; 4]);

impl Area {
    pub const fn new() -> Self {
        Self([0; 4])
    }

    /// Adds `p`, telling whether it was not there yet.
    pub fn insert(&mut self, p: Point) -> bool {
        let i = u8::from(p) as usize;
        let (word, bit) = (i / 64, 1 << (i % 64));
        let new = self.0[word] & bit == 0;
        self.0[word] |= bit;
        new
    }

    pub fn contains(&self, p: Point) -> bool {
        let i = u8::from(p) as usize;
        self.0[i / 64] & 1 << (i % 64) != 0
    }

    /// The points on the four lines through `p` at most `distance` cells
    /// from it, `p` included, whether empty or not. Only `4` and `5` are
    /// tabulated.
    pub fn around(p: Point, distance: u8) -> &'static Self {
        let i = u8::from(p) as usize;
        match distance {
            4 => &AROUND_4[i],
            5 => &AROUND_5[i],
            _ => unimplemented!("Area::around at distance {distance}"),
        }
    }
}

impl BitOrAssign<&Area> for Area {
    fn bitor_assign(&mut self, rhs: &Area) {
        for w in 0..4 {
            self.0[w] |= rhs.0[w];
        }
    }
}

impl BitOrAssign for Area {
    fn bitor_assign(&mut self, rhs: Area) {
        *self |= &rhs;
    }
}

static AROUND_4: [Area; POINT_COUNT] = around_table(4);
static AROUND_5: [Area; POINT_COUNT] = around_table(5);

const fn around_table(distance: i8) -> [Area; POINT_COUNT] {
    const STEPS: [(i8, i8); 4] = [(0, 1), (1, 0), (1, 1), (1, -1)];
    let n = SIZE as i8;
    let mut result = [Area::new(); POINT_COUNT];
    let mut i = 0;
    while i < POINT_COUNT {
        let (x, y) = ((i / SIZE as usize) as i8, (i % SIZE as usize) as i8);
        let mut s = 0;
        while s < STEPS.len() {
            let (dx, dy) = STEPS[s];
            let mut k = -distance;
            while k <= distance {
                let (nx, ny) = (x + k * dx, y + k * dy);
                if 0 <= nx && nx < n && 0 <= ny && ny < n {
                    let j = nx as usize * SIZE as usize + ny as usize;
                    result[i].0[j / 64] |= 1u64 << (j % 64);
                }
                k += 1;
            }
            s += 1;
        }
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn test_insert_and_contains() {
        let mut set = Area::new();
        let points = [Point(0, 0), Point(4, 3), Point(8, 7), Point(14, 14)];
        for p in points {
            assert!(set.insert(p));
        }
        assert!(!set.insert(points[0]));
        assert!(points.iter().all(|&p| set.contains(p)));
        assert!(!set.contains(Point(7, 7)));

        let mut other = Area::new();
        other.insert(Point(7, 7));
        set |= other;
        assert!(set.contains(Point(7, 7)));
    }

    /// The tables agree with `Board::neighbors`, which walks the lines.
    #[test]
    fn test_around_matches_neighbors() {
        let board = Board::new();
        for code in 0..SIZE * SIZE {
            let p = Point::try_from(code).unwrap();
            for distance in [4, 5] {
                let mut expected = Area::new();
                for q in board.neighbors(p, distance, false) {
                    expected.insert(q);
                }
                assert_eq!(*Area::around(p, distance), expected, "{p} {distance}");
            }
        }
    }
}
