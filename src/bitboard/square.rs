use super::bits::*;
use super::line::*;
use super::point::*;
use super::row::*;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Square {
    vlines: OrthogonalLines,
    hlines: OrthogonalLines,
    alines: DiagonalLines,
    dlines: DiagonalLines,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RowSegment {
    pub direction: Direction,
    pub start: Point,
    pub end: Point,
    pub eye1: Option<Point>,
    pub eye2: Option<Point>,
}

impl Square {
    pub fn new() -> Square {
        Square {
            vlines: orthogonal_lines(),
            hlines: orthogonal_lines(),
            alines: diagonal_lines(),
            dlines: diagonal_lines(),
        }
    }

    pub fn put(&mut self, player: Player, p: Point) {
        let vidx = p.to_index(Direction::Vertical);
        self.vlines[vidx.i as usize].put(player, vidx.j);

        let hidx = p.to_index(Direction::Horizontal);
        self.hlines[hidx.i as usize].put(player, hidx.j);

        let aidx = p.to_index(Direction::Ascending);
        if bw(4, aidx.i, D_LINE_NUM + 3) {
            let i = (aidx.i - 4) as usize;
            self.alines[i].put(player, aidx.j);
        }

        let didx = p.to_index(Direction::Descending);
        if bw(4, didx.i, D_LINE_NUM + 3) {
            let i = (didx.i - 4) as usize;
            self.dlines[i].put(player, didx.j);
        }
    }

    pub fn rows(&mut self, player: Player, kind: RowKind) -> Vec<RowSegment> {
        self.iter_mut_lines()
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| RowSegment::new(&r, d, i))
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn rows_on(&mut self, player: Player, kind: RowKind, p: Point) -> Vec<RowSegment> {
        self.iter_mut_lines_along(p)
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| RowSegment::new(&r, d, i))
                    .filter(|r| r.overlap(p))
            })
            .flatten()
            .collect::<Vec<_>>()
    }

    pub fn row_eyes(&mut self, player: Player, kind: RowKind) -> Vec<Point> {
        let mut result = self
            .iter_mut_lines()
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| RowSegment::new(&r, d, i))
                    .map(|r| r.into_iter_eyes())
                    .flatten()
            })
            .flatten()
            .collect::<Vec<_>>();
        result.sort_unstable();
        result.dedup();
        result
    }

    pub fn row_eyes_along(&mut self, player: Player, kind: RowKind, p: Point) -> Vec<Point> {
        let mut result = self
            .iter_mut_lines_along(p)
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| RowSegment::new(&r, d, i))
                    .map(|r| r.into_iter_eyes())
                    .flatten()
            })
            .flatten()
            .collect::<Vec<_>>();
        result.sort_unstable();
        result.dedup();
        result
    }

    fn iter_mut_lines(&mut self) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let viter = self
            .vlines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Vertical, i as u8, l));
        let hiter = self
            .hlines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Horizontal, i as u8, l));
        let aiter = self
            .alines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Ascending, (i + 4) as u8, l));
        let diter = self
            .dlines
            .iter_mut()
            .enumerate()
            .map(|(i, l)| (Direction::Descending, (i + 4) as u8, l));
        viter.chain(hiter).chain(aiter).chain(diter)
    }

    fn iter_mut_lines_along(
        &mut self,
        p: Point,
    ) -> impl Iterator<Item = (Direction, u8, &mut Line)> {
        let vidx = p.to_index(Direction::Vertical);
        let vline = &mut self.vlines[vidx.i as usize];
        let viter = Some((Direction::Vertical, vidx.i, vline)).into_iter();

        let hidx = p.to_index(Direction::Horizontal);
        let hline = &mut self.hlines[hidx.i as usize];
        let hiter = Some((Direction::Horizontal, hidx.i, hline)).into_iter();

        let aidx = p.to_index(Direction::Ascending);
        let aiter = if bw(4, aidx.i, D_LINE_NUM + 3) {
            let aline = &mut self.alines[(aidx.i - 4) as usize];
            Some((Direction::Ascending, aidx.i, aline))
        } else {
            None
        }
        .into_iter();

        let didx = p.to_index(Direction::Descending);
        let diter = if bw(4, didx.i, D_LINE_NUM + 3) {
            let dline = &mut self.dlines[(didx.i - 4) as usize];
            Some((Direction::Descending, didx.i, dline))
        } else {
            None
        }
        .into_iter();

        viter.chain(hiter).chain(aiter).chain(diter)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rev_hlines: Vec<_> = self.hlines.iter().rev().collect();
        let s = lines_to_string(&rev_hlines);
        write!(f, "{}", s)
    }
}

impl FromStr for Square {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hlines_rev = s
            .trim()
            .split("\n")
            .map(|ls| ls.trim().parse::<Line>())
            .collect::<Result<Vec<_>, _>>()?;
        if hlines_rev.len() != BOARD_SIZE as usize {
            return Err("Wrong num of lines");
        }
        let mut square = Square::new();
        for (y, hline) in hlines_rev.iter().rev().enumerate() {
            if hline.size != BOARD_SIZE {
                return Err("Wrong line size");
            }
            for (x, s) in hline.stones().iter().enumerate() {
                let point = Point {
                    x: x as u8,
                    y: y as u8,
                };
                match s {
                    Some(player) => square.put(*player, point),
                    None => (),
                }
            }
        }
        Ok(square)
    }
}

impl RowSegment {
    pub fn new(r: &Row, d: Direction, i: u8) -> RowSegment {
        RowSegment {
            direction: d,
            start: Index { i: i, j: r.start }.to_point(d),
            end: Index { i: i, j: r.end }.to_point(d),
            eye1: r.eye1.map(|e| Index { i: i, j: e }.to_point(d)),
            eye2: r.eye2.map(|e| Index { i: i, j: e }.to_point(d)),
        }
    }

    pub fn overlap(&self, p: Point) -> bool {
        let (px, py) = (p.x, p.y);
        let (sx, sy) = (self.start.x, self.start.y);
        let (ex, ey) = (self.end.x, self.end.y);
        match self.direction {
            Direction::Vertical => px == sx && bw(sy, py, ey),
            Direction::Horizontal => py == sy && bw(sx, px, ex),
            Direction::Ascending => bw(sx, px, ex) && bw(sy, py, ey) && px - sx == py - sy,
            Direction::Descending => bw(sx, px, ex) && bw(ey, py, sy) && px - sx == sy - py,
        }
    }

    pub fn into_iter_eyes(&self) -> impl IntoIterator<Item = Point> {
        self.eye1.into_iter().chain(self.eye2.into_iter())
    }
}

impl fmt::Display for RowSegment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} start: {} end: {}",
            self.direction, self.start, self.end
        )?;
        for eye1 in self.eye1 {
            write!(f, " eye1: {}", eye1)?;
        }
        for eye2 in self.eye2 {
            write!(f, " eye2: {}", eye2)?;
        }
        Ok(())
    }
}

const O_LINE_NUM: u8 = BOARD_SIZE;
const D_LINE_NUM: u8 = BOARD_SIZE * 2 - 1 - (4 * 2); // 21

type OrthogonalLines = [Line; O_LINE_NUM as usize];
type DiagonalLines = [Line; D_LINE_NUM as usize];

fn orthogonal_lines() -> OrthogonalLines {
    [
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE),
    ]
}

fn diagonal_lines() -> DiagonalLines {
    [
        Line::new(BOARD_SIZE - 10),
        Line::new(BOARD_SIZE - 9),
        Line::new(BOARD_SIZE - 8),
        Line::new(BOARD_SIZE - 7),
        Line::new(BOARD_SIZE - 6),
        Line::new(BOARD_SIZE - 5),
        Line::new(BOARD_SIZE - 4),
        Line::new(BOARD_SIZE - 3),
        Line::new(BOARD_SIZE - 2),
        Line::new(BOARD_SIZE - 1),
        Line::new(BOARD_SIZE),
        Line::new(BOARD_SIZE - 1),
        Line::new(BOARD_SIZE - 2),
        Line::new(BOARD_SIZE - 3),
        Line::new(BOARD_SIZE - 4),
        Line::new(BOARD_SIZE - 5),
        Line::new(BOARD_SIZE - 6),
        Line::new(BOARD_SIZE - 7),
        Line::new(BOARD_SIZE - 8),
        Line::new(BOARD_SIZE - 9),
        Line::new(BOARD_SIZE - 10),
    ]
}

fn bw(a: u8, x: u8, b: u8) -> bool {
    a <= x && x <= b
}

fn lines_to_string(lines: &[&Line]) -> String {
    lines
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::Direction::*;
    use super::Player::*;
    use super::RowKind::*;
    use super::*;

    #[test]
    fn test_put() {
        let mut square = Square::new();
        square.put(Black, Point::new(7, 7));
        square.put(White, Point::new(8, 8));
        square.put(Black, Point::new(9, 8));
        square.put(Black, Point::new(1, 1));
        square.put(White, Point::new(1, 13));
        square.put(Black, Point::new(13, 1));
        square.put(White, Point::new(13, 13));

        let result = lines_to_string(&square.hlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
            ---------------
            -o-----------o-
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -------o-------
            --------xo-----
            ---------------
            ---------------
            ---------------
            ---------------
            -x-----------x-
            ---------------
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.vlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
            ---------------
            -o-----------x-
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -------o-------
            --------x------
            --------o------
            ---------------
            ---------------
            ---------------
            -o-----------x-
            ---------------
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.alines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
            -----
            ------
            -------
            --------
            ---------
            ----------
            -----------
            ------------
            -------------
            --------------
            -o-----ox----x-
            --------o-----
            -------------
            ------------
            -----------
            ----------
            ---------
            --------
            -------
            ------
            -----
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.dlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
            -----
            ------
            -------
            --------
            ---------
            ----------
            -----------
            ------------
            -------------
            --------------
            -x-----o-----o-
            --------------
            ------x------
            ------o-----
            -----------
            ----------
            ---------
            --------
            -------
            ------
            -----
            ",
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_rows_and_so_on() -> Result<(), String> {
        let mut square = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            -------xxxo----
            -------o-------
            ---------------
            -------o-------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Square>()?;
        let black_twos = vec![RowSegment {
            direction: Ascending,
            start: Point::new(6, 4),
            end: Point::new(11, 9),
            eye1: Some(Point::new(8, 6)),
            eye2: Some(Point::new(9, 7)),
        }];
        let white_swords = [RowSegment {
            direction: Horizontal,
            start: Point::new(5, 8),
            end: Point::new(9, 8),
            eye1: Some(Point::new(5, 8)),
            eye2: Some(Point::new(6, 8)),
        }];

        // rows
        assert_eq!(square.rows(Black, Two), black_twos);
        assert_eq!(square.rows(White, Sword), white_swords);

        // rows_on
        assert_eq!(square.rows_on(Black, Two, Point::new(10, 8)), black_twos);
        assert_eq!(square.rows_on(Black, Two, Point::new(7, 7)), []);

        // row_eyes
        assert_eq!(
            square.row_eyes(Black, Two),
            [Point::new(8, 6), Point::new(9, 7)]
        );
        assert_eq!(
            square.row_eyes(White, Sword),
            [Point::new(5, 8), Point::new(6, 8)]
        );

        // row_eyes_along
        assert_eq!(
            square.row_eyes_along(White, Sword, Point::new(0, 8)),
            [Point::new(5, 8), Point::new(6, 8)]
        );
        assert_eq!(square.row_eyes_along(White, Sword, Point::new(0, 7)), []);

        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "
            x-------------x
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            --------xo-----
            -------o-------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            o-------------o
        "
        .parse::<Square>()?;
        let mut expected = Square::new();
        expected.put(Black, Point::new(7, 7));
        expected.put(White, Point::new(8, 8));
        expected.put(Black, Point::new(9, 8));
        expected.put(Black, Point::new(0, 0));
        expected.put(White, Point::new(0, 14));
        expected.put(Black, Point::new(14, 0));
        expected.put(White, Point::new(14, 14));
        assert_eq!(result, expected);
        Ok(())
    }

    fn trim_lines_string(s: &str) -> String {
        s.trim()
            .split("\n")
            .map(|ls| ls.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
