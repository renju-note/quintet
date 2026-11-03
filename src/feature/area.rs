use crate::board::{POINT_COUNT, Point, SIZE};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Sub, SubAssign};

/// A set of points, one bit per point by its `u8` code (`x * SIZE + y`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Area([u64; 4]);

impl Area {
    #[inline]
    pub const fn new() -> Self {
        Self([0; 4])
    }

    /// Adds `p`, telling whether it was not there yet.
    #[inline]
    pub fn insert(&mut self, p: Point) -> bool {
        let i = u8::from(p) as usize;
        let (word, bit) = (i / 64, 1 << (i % 64));
        let new = self.0[word] & bit == 0;
        self.0[word] |= bit;
        new
    }

    /// Removes `p`, telling whether it was there.
    #[inline]
    pub fn remove(&mut self, p: Point) -> bool {
        let i = u8::from(p) as usize;
        let (word, bit) = (i / 64, 1 << (i % 64));
        let old = self.0[word] & bit != 0;
        self.0[word] &= !bit;
        old
    }

    #[inline]

    pub fn contains(&self, p: Point) -> bool {
        let i = u8::from(p) as usize;
        self.0[i / 64] & 1 << (i % 64) != 0
    }

    #[inline]

    pub fn is_empty(&self) -> bool {
        self.0 == [0; 4]
    }

    /// How many points the area has.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// The points, in the order of their codes (`u8::from`).
    #[inline]
    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        self.into_iter()
    }

    /// The points on the four lines through `p` at most `distance` cells
    /// from it, `p` included, whether empty or not. Only `4` and `5` are
    /// tabulated.
    #[inline]
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
    #[inline]
    fn bitor_assign(&mut self, rhs: &Area) {
        for w in 0..4 {
            self.0[w] |= rhs.0[w];
        }
    }
}

impl BitOrAssign for Area {
    #[inline]
    fn bitor_assign(&mut self, rhs: Area) {
        *self |= &rhs;
    }
}

impl BitAndAssign<&Area> for Area {
    #[inline]
    fn bitand_assign(&mut self, rhs: &Area) {
        for w in 0..4 {
            self.0[w] &= rhs.0[w];
        }
    }
}

impl BitAndAssign for Area {
    #[inline]
    fn bitand_assign(&mut self, rhs: Area) {
        *self &= &rhs;
    }
}

impl BitOr for Area {
    type Output = Area;

    #[inline]

    fn bitor(mut self, rhs: Area) -> Area {
        self |= &rhs;
        self
    }
}

impl BitAnd for Area {
    type Output = Area;

    #[inline]

    fn bitand(mut self, rhs: Area) -> Area {
        self &= &rhs;
        self
    }
}

/// The points of the left area that are not in the right one.
impl SubAssign<&Area> for Area {
    #[inline]
    fn sub_assign(&mut self, rhs: &Area) {
        for w in 0..4 {
            self.0[w] &= !rhs.0[w];
        }
    }
}

impl SubAssign for Area {
    #[inline]
    fn sub_assign(&mut self, rhs: Area) {
        *self -= &rhs;
    }
}

impl Sub for Area {
    type Output = Area;

    #[inline]

    fn sub(mut self, rhs: Area) -> Area {
        self -= &rhs;
        self
    }
}

/// The points, in the order of their codes, as [`Area::points`]; the
/// iterator owns its copy of the area, so it can outlive the area it was
/// taken from.
impl IntoIterator for Area {
    type Item = Point;
    type IntoIter = AreaPoints;

    #[inline]

    fn into_iter(self) -> AreaPoints {
        AreaPoints {
            words: self.0,
            w: 0,
            bits: self.0[0],
        }
    }
}

/// The points of an [`Area`], taken by [`Area::into_iter`].
#[derive(Debug, Clone)]
pub struct AreaPoints {
    words: [u64; 4],
    /// The word `bits` was taken from.
    w: usize,
    /// What is left of that word.
    bits: u64,
}

impl Iterator for AreaPoints {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Point> {
        while self.bits == 0 {
            self.w += 1;
            if self.w >= 4 {
                return None;
            }
            self.bits = self.words[self.w];
        }
        let b = self.bits.trailing_zeros() as u8;
        self.bits &= self.bits - 1;
        let code = self.w as u8 * 64 + b;
        Some(Point(code / SIZE, code % SIZE))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest: usize = self.words[(self.w + 1).min(4)..]
            .iter()
            .map(|w| w.count_ones() as usize)
            .sum();
        let n = self.bits.count_ones() as usize + rest;
        (n, Some(n))
    }
}

impl ExactSizeIterator for AreaPoints {}

impl FromIterator<Point> for Area {
    fn from_iter<I: IntoIterator<Item = Point>>(points: I) -> Self {
        let mut result = Area::new();
        for p in points {
            result.insert(p);
        }
        result
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

    #[test]
    fn test_remove_points_and_set_operations() {
        let points = [Point(0, 0), Point(4, 3), Point(8, 7), Point(14, 14)];
        let mut set: Area = points.into_iter().collect();
        assert_eq!(set.len(), 4);
        // In code order, across the words.
        assert_eq!(set.points().collect::<Vec<_>>(), points);
        assert!(set.remove(Point(4, 3)));
        assert!(!set.remove(Point(4, 3)));
        assert!(!set.contains(Point(4, 3)));
        assert_eq!(set.len(), 3);

        let other: Area = [Point(8, 7), Point(7, 7)].into_iter().collect();
        assert_eq!((set & other).points().collect::<Vec<_>>(), [Point(8, 7)]);
        assert_eq!((set | other).len(), 4);
        assert!(Area::new().is_empty());
        assert!((set & Area::new()).is_empty());

        // The difference keeps what the other has not, across the words.
        let diff: Vec<_> = (set - other).points().collect();
        assert_eq!(diff, [Point(0, 0), Point(14, 14)]);
        assert_eq!(other - set, [Point(7, 7)].into_iter().collect());
        assert!((set - set).is_empty());
    }

    #[test]
    fn test_into_iter_owns_its_copy() {
        let points = [Point(0, 0), Point(4, 3), Point(8, 7), Point(14, 14)];
        let iter = {
            let set: Area = points.into_iter().collect();
            set.into_iter()
        };
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.collect::<Vec<_>>(), points);
        assert_eq!(Area::new().into_iter().next(), None);
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
