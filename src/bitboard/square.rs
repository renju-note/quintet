use super::fundamentals::*;
use super::line::*;
use super::point::*;
use super::segment::*;
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
pub struct Row {
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

    pub fn from_points(blacks: Points, whites: Points) -> Square {
        let mut square = Square::new();
        for p in blacks.0.into_iter() {
            square.put(Player::Black, p);
        }
        for p in whites.0.into_iter() {
            square.put(Player::White, p);
        }
        square
    }

    pub fn put(&mut self, player: Player, p: Point) {
        let vidx = p.to_index(Direction::Vertical);
        self.vlines[vidx.0 as usize].put(player, vidx.1);

        let hidx = p.to_index(Direction::Horizontal);
        self.hlines[hidx.0 as usize].put(player, hidx.1);

        let aidx = p.to_index(Direction::Ascending);
        if bw(4, aidx.0, D_LINE_NUM + 3) {
            let i = (aidx.0 - 4) as usize;
            self.alines[i].put(player, aidx.1);
        }

        let didx = p.to_index(Direction::Descending);
        if bw(4, didx.0, D_LINE_NUM + 3) {
            let i = (didx.0 - 4) as usize;
            self.dlines[i].put(player, didx.1);
        }
    }

    pub fn rows(&self, player: Player, kind: RowKind) -> Vec<Row> {
        self.iter_lines()
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| Row::from_row(&r, d, i))
            })
            .flatten()
            .collect()
    }

    pub fn rows_on(&self, player: Player, kind: RowKind, p: Point) -> Vec<Row> {
        self.iter_lines_along(p)
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| Row::from_row(&r, d, i))
                    .filter(|r| r.overlap(p))
            })
            .flatten()
            .collect()
    }

    pub fn row_eyes(&self, player: Player, kind: RowKind) -> Vec<Point> {
        let mut result: Vec<_> = self
            .iter_lines()
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| Row::from_row(&r, d, i))
                    .map(|r| r.into_iter_eyes())
                    .flatten()
            })
            .flatten()
            .collect();
        result.sort_unstable();
        result.dedup();
        result
    }

    pub fn row_eyes_along(&self, player: Player, kind: RowKind, p: Point) -> Vec<Point> {
        let mut result: Vec<_> = self
            .iter_lines_along(p)
            .map(|(d, i, l)| {
                l.rows(player, kind)
                    .into_iter()
                    .map(move |r| Row::from_row(&r, d, i))
                    .map(|r| r.into_iter_eyes())
                    .flatten()
            })
            .flatten()
            .collect();
        result.sort_unstable();
        result.dedup();
        result
    }

    fn iter_lines(&self) -> impl Iterator<Item = (Direction, u8, &Line)> {
        let viter = self
            .vlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Vertical, i as u8, l));
        let hiter = self
            .hlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Horizontal, i as u8, l));
        let aiter = self
            .alines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Ascending, (i + 4) as u8, l));
        let diter = self
            .dlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Direction::Descending, (i + 4) as u8, l));
        viter.chain(hiter).chain(aiter).chain(diter)
    }

    fn iter_lines_along(&self, p: Point) -> impl Iterator<Item = (Direction, u8, &Line)> {
        let vidx = p.to_index(Direction::Vertical);
        let vline = &self.vlines[vidx.0 as usize];
        let viter = Some((Direction::Vertical, vidx.0, vline)).into_iter();

        let hidx = p.to_index(Direction::Horizontal);
        let hline = &self.hlines[hidx.0 as usize];
        let hiter = Some((Direction::Horizontal, hidx.0, hline)).into_iter();

        let aidx = p.to_index(Direction::Ascending);
        let aiter = bw(4, aidx.0, D_LINE_NUM + 3)
            .then(|| {
                let aline = &self.alines[(aidx.0 - 4) as usize];
                (Direction::Ascending, aidx.0, aline)
            })
            .into_iter();

        let didx = p.to_index(Direction::Descending);
        let diter = bw(4, didx.0, D_LINE_NUM + 3)
            .then(|| {
                let dline = &self.dlines[(didx.0 - 4) as usize];
                (Direction::Descending, didx.0, dline)
            })
            .into_iter();

        viter.chain(hiter).chain(aiter).chain(diter)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rev_hlines: Vec<_> = self.hlines.iter().rev().collect();
        let s = lines_to_string(&rev_hlines);
        f.write_str(&s)
    }
}

impl FromStr for Square {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("/") {
            from_str_points(s)
        } else {
            from_str_display(s)
        }
    }
}

impl Row {
    pub fn new(
        direction: Direction,
        start: Point,
        end: Point,
        eye1: Option<Point>,
        eye2: Option<Point>,
    ) -> Row {
        Row {
            direction: direction,
            start: start,
            end: end,
            eye1: eye1,
            eye2: eye2,
        }
    }

    pub fn from_row(r: &Segment, d: Direction, i: u8) -> Row {
        Row {
            direction: d,
            start: Index(i, r.start).to_point(d),
            end: Index(i, r.end).to_point(d),
            eye1: r.eye1.map(|e| Index(i, e).to_point(d)),
            eye2: r.eye2.map(|e| Index(i, e).to_point(d)),
        }
    }

    pub fn overlap(&self, p: Point) -> bool {
        let (px, py) = (p.0, p.1);
        let (sx, sy) = (self.start.0, self.start.1);
        let (ex, ey) = (self.end.0, self.end.1);
        match self.direction {
            Direction::Vertical => px == sx && bw(sy, py, ey),
            Direction::Horizontal => py == sy && bw(sx, px, ex),
            Direction::Ascending => bw(sx, px, ex) && bw(sy, py, ey) && px - sx == py - sy,
            Direction::Descending => bw(sx, px, ex) && bw(ey, py, sy) && px - sx == sy - py,
        }
    }

    pub fn adjacent(&self, other: &Row) -> bool {
        if self.direction != other.direction {
            return false;
        }
        let (sx, sy) = (self.start.0, self.start.1);
        let (ox, oy) = (other.start.0, other.start.1);
        let (xd, yd) = (sx as i8 - ox as i8, sy as i8 - oy as i8);
        match self.direction {
            Direction::Vertical => xd == 0 && yd.abs() == 1,
            Direction::Horizontal => xd.abs() == 1 && yd == 0,
            Direction::Ascending => xd.abs() == 1 && xd == yd,
            Direction::Descending => xd.abs() == 1 && xd == -yd,
        }
    }

    pub fn into_iter_eyes(&self) -> impl IntoIterator<Item = Point> {
        self.eye1.into_iter().chain(self.eye2.into_iter())
    }
}

impl fmt::Display for Row {
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

const D_LINE_NUM: u8 = (BOARD_SIZE - (5 - 1)) * 2 - 1; // 21

type OrthogonalLines = [Line; BOARD_SIZE as usize];
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

fn from_str_points(s: &str) -> Result<Square, &'static str> {
    let codes = s.trim().split("/").collect::<Vec<_>>();
    if codes.len() != 2 {
        return Err("Unknown format.");
    }
    let blacks = codes[0].parse::<Points>()?;
    let whites = codes[1].parse::<Points>()?;
    Ok(Square::from_points(blacks, whites))
}

fn from_str_display(s: &str) -> Result<Square, &'static str> {
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
            let point = Point(x as u8, y as u8);
            match s {
                Some(player) => square.put(*player, point),
                None => (),
            }
        }
    }
    Ok(square)
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
        square.put(Black, Point(7, 7));
        square.put(White, Point(8, 8));
        square.put(Black, Point(9, 8));
        square.put(Black, Point(1, 1));
        square.put(White, Point(1, 13));
        square.put(Black, Point(13, 1));
        square.put(White, Point(13, 13));

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
        let square = "
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
        let black_twos = [Row::new(
            Ascending,
            Point(6, 4),
            Point(11, 9),
            Some(Point(8, 6)),
            Some(Point(9, 7)),
        )];
        let white_swords = [Row::new(
            Horizontal,
            Point(5, 8),
            Point(9, 8),
            Some(Point(5, 8)),
            Some(Point(6, 8)),
        )];

        assert_eq!(square.rows(Black, Two), black_twos);
        assert_eq!(square.rows(White, Sword), white_swords);

        assert_eq!(square.rows_on(Black, Two, Point(10, 8)), black_twos);
        assert_eq!(square.rows_on(Black, Two, Point(7, 7)), []);

        assert_eq!(square.row_eyes(Black, Two), [Point(8, 6), Point(9, 7)]);
        assert_eq!(square.row_eyes(White, Sword), [Point(5, 8), Point(6, 8)]);

        assert_eq!(
            square.row_eyes_along(White, Sword, Point(0, 8)),
            [Point(5, 8), Point(6, 8)]
        );
        assert_eq!(square.row_eyes_along(White, Sword, Point(0, 7)), []);

        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "H8,J9/I9".parse::<Square>()?;
        let mut expected = Square::new();
        expected.put(Black, Point(7, 7));
        expected.put(White, Point(8, 8));
        expected.put(Black, Point(9, 8));
        assert_eq!(result, expected);

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
        expected.put(Black, Point(7, 7));
        expected.put(White, Point(8, 8));
        expected.put(Black, Point(9, 8));
        expected.put(Black, Point(0, 0));
        expected.put(White, Point(0, 14));
        expected.put(Black, Point(14, 0));
        expected.put(White, Point(14, 14));
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_to_string() {
        let mut square = Square::new();
        square.put(Black, Point(7, 7));
        square.put(White, Point(8, 8));
        square.put(Black, Point(9, 8));
        square.put(Black, Point(0, 0));
        square.put(White, Point(0, 14));
        square.put(Black, Point(14, 0));
        square.put(White, Point(14, 14));
        let expected = trim_lines_string(
            "
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
        ",
        );
        assert_eq!(square.to_string(), expected);
    }

    #[test]
    fn test_adjacent() {
        let a = Row::new(Vertical, Point(3, 3), Point(3, 9), None, None);
        let b = Row::new(Horizontal, Point(3, 3), Point(9, 3), None, None);
        assert!(!a.adjacent(&b));

        let a = Row::new(Vertical, Point(3, 3), Point(3, 9), None, None);
        let b = Row::new(Vertical, Point(3, 4), Point(3, 10), None, None);
        assert!(a.adjacent(&b));

        let a = Row::new(Vertical, Point(3, 3), Point(3, 9), None, None);
        let b = Row::new(Vertical, Point(3, 5), Point(3, 11), None, None);
        assert!(!a.adjacent(&b));

        let a = Row::new(Descending, Point(3, 9), Point(9, 3), None, None);
        let b = Row::new(Descending, Point(4, 8), Point(10, 2), None, None);
        assert!(a.adjacent(&b));
    }

    fn trim_lines_string(s: &str) -> String {
        s.trim()
            .split("\n")
            .map(|ls| ls.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
