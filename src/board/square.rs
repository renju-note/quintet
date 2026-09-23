use super::line::*;
use super::player::*;
use super::point::*;
use super::potential::VICTORY;
use super::structure::*;
use std::fmt;
use std::str::FromStr;

const D_LINE_OMIT: u8 = VICTORY - 1;
const D_LINE_NUM: u8 = (RANGE - D_LINE_OMIT) * 2 - 1; // 21

type OrthogonalLines = [Line; RANGE as usize];
type DiagonalLines = [Line; D_LINE_NUM as usize];

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Square {
    vlines: OrthogonalLines,
    hlines: OrthogonalLines,
    alines: DiagonalLines,
    dlines: DiagonalLines,
}

impl Square {
    pub fn new() -> Self {
        Self {
            vlines: orthogonal_lines(),
            hlines: orthogonal_lines(),
            alines: diagonal_lines(),
            dlines: diagonal_lines(),
        }
    }

    pub fn from_moves(moves: &Points) -> Self {
        let mut square = Self::new();
        let mut player = Black;
        for &m in moves.0.iter() {
            square.put_mut(player, m);
            player = player.opponent();
        }
        square
    }

    pub fn from_stones(blacks: &Points, whites: &Points) -> Self {
        let mut square = Self::new();
        for &p in blacks.0.iter() {
            square.put_mut(Black, p);
        }
        for &p in whites.0.iter() {
            square.put_mut(White, p);
        }
        square
    }

    pub fn put_mut(&mut self, player: Player, p: Point) {
        self.update_lines_on(p, |line, j| line.put_mut(player, j));
    }

    pub fn remove_mut(&mut self, p: Point) {
        self.update_lines_on(p, |line, j| line.remove_mut(j));
    }

    pub fn stone(&self, p: Point) -> Option<Player> {
        let vidx = p.to_index(Vertical);
        self.vlines[vidx.i as usize].stone(vidx.j)
    }

    pub fn stones(&self, player: Player) -> impl Iterator<Item = Point> + '_ {
        self.vlines.iter().enumerate().flat_map(move |(i, l)| {
            l.stones(player)
                .map(move |j| Index::new(Vertical, i as u8, j).to_point())
        })
    }

    pub fn neighbors(
        &self,
        p: Point,
        distance: u8,
        only_empty: bool,
    ) -> impl Iterator<Item = Point> + '_ {
        let d = distance as i8;
        vec![
            p.to_index(Vertical),
            p.to_index(Horizontal),
            p.to_index(Ascending),
            p.to_index(Descending),
        ]
        .into_iter()
        .flat_map(move |idx| (-d..=d).flat_map(move |j| idx.walk_checked(j)))
        .map(|idx| idx.to_point())
        .filter(move |&p| !only_empty || self.stone(p).is_none())
    }

    pub fn empties(&self) -> impl Iterator<Item = Point> + '_ {
        self.vlines.iter().enumerate().flat_map(move |(i, l)| {
            l.empties()
                .map(move |j| Index::new(Vertical, i as u8, j).to_point())
        })
    }

    /// The `i`-th line in direction `d`, or `None` for the diagonals shorter
    /// than five, which are not stored.
    pub fn line(&self, d: Direction, i: u8) -> Option<&Line> {
        let index = Index::new(d, i, 0);
        Self::line_idx(index).map(|k| match d {
            Vertical => &self.vlines[k],
            Horizontal => &self.hlines[k],
            Ascending => &self.alines[k],
            Descending => &self.dlines[k],
        })
    }

    /// The line through `p` in direction `d`. `p` itself is at cell
    /// `p.to_index(d).j` of it.
    pub fn line_on(&self, p: Point, d: Direction) -> Option<&Line> {
        self.line(d, p.to_index(d).i)
    }

    /// Every stored line, as `(direction, i, line)`.
    pub fn lines(&self) -> impl Iterator<Item = (Direction, u8, &Line)> {
        let viter = self
            .vlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Vertical, i as u8, l));

        let hiter = self
            .hlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Horizontal, i as u8, l));

        let aiter = self
            .alines
            .iter()
            .enumerate()
            .map(|(i, l)| (Ascending, (i as u8 + D_LINE_OMIT), l));

        let diter = self
            .dlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Descending, (i as u8 + D_LINE_OMIT), l));

        viter.chain(hiter).chain(aiter).chain(diter)
    }

    /// The (at most four) stored lines through `p`.
    pub fn lines_on(&self, p: Point) -> impl Iterator<Item = (Direction, u8, &Line)> {
        let vidx = p.to_index(Vertical);
        let viter = Self::line_idx(vidx)
            .map(|i| (Vertical, vidx.i, &self.vlines[i]))
            .into_iter();

        let hidx = p.to_index(Horizontal);
        let hiter = Self::line_idx(hidx)
            .map(|i| (Horizontal, hidx.i, &self.hlines[i]))
            .into_iter();

        let aidx = p.to_index(Ascending);
        let aiter = Self::line_idx(aidx)
            .map(|i| (Ascending, aidx.i, &self.alines[i]))
            .into_iter();

        let didx = p.to_index(Descending);
        let diter = Self::line_idx(didx)
            .map(|i| (Descending, didx.i, &self.dlines[i]))
            .into_iter();

        viter.chain(hiter).chain(aiter).chain(diter)
    }

    pub fn structures(&self, r: Player, k: StructureKind) -> impl Iterator<Item = Structure> + '_ {
        let (sk, n, exact) = k.to_sequence(r);
        self.lines()
            .filter(move |(_, _, l)| l.potential_cap(r) > n)
            .flat_map(move |(d, i, l)| {
                l.sequences(r, sk, n, exact)
                    .map(move |(j, s)| Structure::new(Index::new(d, i, j), s))
            })
    }

    pub fn structures_on(
        &self,
        p: Point,
        r: Player,
        k: StructureKind,
    ) -> impl Iterator<Item = Structure> + '_ {
        let (sk, n, exact) = k.to_sequence(r);
        self.lines_on(p)
            .filter(move |(_, _, l)| l.potential_cap(r) > n)
            .flat_map(move |(d, i, l)| {
                let j = p.to_index(d).j;
                l.sequences_on(j, r, sk, n, exact)
                    .map(move |(j, s)| Structure::new(Index::new(d, i, j), s))
            })
    }

    pub fn potentials(
        &self,
        r: Player,
        min: u8,
        exact: bool,
    ) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.lines()
            .filter(move |(_, _, l)| l.potential_cap(r) >= min)
            .flat_map(move |(d, i, l)| {
                l.potentials(r, min, exact)
                    .map(move |(j, p)| (Index::new(d, i, j), p))
            })
    }

    pub fn potentials_along(
        &self,
        p: Point,
        r: Player,
        min: u8,
        exact: bool,
    ) -> impl Iterator<Item = (Index, u8)> + '_ {
        self.lines_on(p)
            .filter(move |(_, _, l)| l.potential_cap(r) >= min)
            .flat_map(move |(d, i, l)| {
                l.potentials(r, min, exact)
                    .map(move |(j, p)| (Index::new(d, i, j), p))
            })
    }

    pub fn to_pretty_string(&self) -> String {
        let mut result = String::new();
        for (i, l) in self.hlines.iter().enumerate().rev() {
            result.push_str(&format!("{: >2}{}\n", i + 1, l));
        }
        let xindices = ('A'..='O').map(|c| c.to_string()).collect::<Vec<_>>();
        result.push_str(&format!("   {}", xindices.join(" ")));
        result
    }

    /// Applies `f` to each stored line through `p`, with `p`'s cell on it.
    fn update_lines_on(&mut self, p: Point, mut f: impl FnMut(&mut Line, u8)) {
        let vidx = p.to_index(Vertical);
        if let Some(i) = Self::line_idx(vidx) {
            f(&mut self.vlines[i], vidx.j)
        }

        let hidx = p.to_index(Horizontal);
        if let Some(i) = Self::line_idx(hidx) {
            f(&mut self.hlines[i], hidx.j)
        }

        let aidx = p.to_index(Ascending);
        if let Some(i) = Self::line_idx(aidx) {
            f(&mut self.alines[i], aidx.j)
        }

        let didx = p.to_index(Descending);
        if let Some(i) = Self::line_idx(didx) {
            f(&mut self.dlines[i], didx.j)
        }
    }

    fn line_idx(index: Index) -> Option<usize> {
        let i = index.i;
        match index.d {
            Vertical => Some(i as usize),
            Horizontal => Some(i as usize),
            _ => {
                if (D_LINE_OMIT..D_LINE_OMIT + D_LINE_NUM).contains(&i) {
                    Some((i - D_LINE_OMIT) as usize)
                } else {
                    None
                }
            }
        }
    }
}

impl Default for Square {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = self
            .hlines
            .iter()
            .rev()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        f.write_str(&s)
    }
}

impl FromStr for Square {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("/") {
            from_str_stones(s)
        } else if s.contains(",") {
            from_str_moves(s)
        } else {
            from_str_display(s)
        }
    }
}

fn from_str_stones(s: &str) -> Result<Square, &'static str> {
    let mut codes = s.trim().split("/");
    let blacks_str = codes.next().ok_or("Wrong format.")?;
    let whites_str = codes.next().ok_or("Wrong format.")?;
    let blacks = blacks_str.parse::<Points>()?;
    let whites = whites_str.parse::<Points>()?;
    Ok(Square::from_stones(&blacks, &whites))
}

fn from_str_moves(s: &str) -> Result<Square, &'static str> {
    let moves = s.trim().parse::<Points>()?;
    Ok(Square::from_moves(&moves))
}

fn from_str_display(s: &str) -> Result<Square, &'static str> {
    let hlines_rev = s
        .trim()
        .split("\n")
        .map(|ls| ls.trim().parse::<Line>())
        .collect::<Result<Vec<_>, _>>()?;
    if hlines_rev.len() != RANGE as usize {
        return Err("Wrong num of lines");
    }
    let mut square = Square::new();
    for (i, hline) in hlines_rev.iter().rev().enumerate() {
        if hline.size != RANGE {
            return Err("Wrong line size");
        }
        for j in 0..hline.size {
            if let Some(player) = hline.stone(j) {
                let point = Index::new(Horizontal, i as u8, j).to_point();
                square.put_mut(player, point)
            }
        }
    }
    Ok(square)
}

fn orthogonal_lines() -> OrthogonalLines {
    [
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
        Line::new(RANGE),
    ]
}

fn diagonal_lines() -> DiagonalLines {
    [
        Line::new(RANGE - 10),
        Line::new(RANGE - 9),
        Line::new(RANGE - 8),
        Line::new(RANGE - 7),
        Line::new(RANGE - 6),
        Line::new(RANGE - 5),
        Line::new(RANGE - 4),
        Line::new(RANGE - 3),
        Line::new(RANGE - 2),
        Line::new(RANGE - 1),
        Line::new(RANGE),
        Line::new(RANGE - 1),
        Line::new(RANGE - 2),
        Line::new(RANGE - 3),
        Line::new(RANGE - 4),
        Line::new(RANGE - 5),
        Line::new(RANGE - 6),
        Line::new(RANGE - 7),
        Line::new(RANGE - 8),
        Line::new(RANGE - 9),
        Line::new(RANGE - 10),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_DIRECTIONS: [Direction; 4] = [Vertical, Horizontal, Ascending, Descending];

    fn points(it: impl Iterator<Item = Point>) -> String {
        Points(it.collect()).to_string()
    }

    /// Every point is stored once per direction; all four copies must agree
    /// with `expected` after any sequence of puts and removes.
    fn assert_lines_agree(square: &Square, expected: &[(Point, Player)]) {
        for x in 0..RANGE {
            for y in 0..RANGE {
                let p = Point(x, y);
                let want = expected.iter().find(|(q, _)| *q == p).map(|&(_, r)| r);
                for d in ALL_DIRECTIONS {
                    if let Some(line) = square.line_on(p, d) {
                        assert_eq!(line.stone(p.to_index(d).j), want, "{p} {d:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn test_put_and_remove() {
        let mut square = Square::new();
        // The centre, and a stone near each corner, where the diagonals are
        // short or not stored at all.
        let stones = [
            (Point(7, 7), Black),
            (Point(8, 8), White),
            (Point(9, 8), Black),
            (Point(1, 1), Black),
            (Point(1, 13), White),
            (Point(13, 1), Black),
            (Point(13, 13), White),
            (Point(0, 0), White),
        ];
        for (p, r) in stones {
            square.put_mut(r, p);
        }
        assert_lines_agree(&square, &stones);

        // Replacing a stone, removing two, and removing from an empty point.
        square.put_mut(Black, Point(1, 13));
        square.remove_mut(Point(7, 7));
        square.remove_mut(Point(8, 8));
        square.remove_mut(Point(9, 9));
        let stones = [
            (Point(9, 8), Black),
            (Point(1, 1), Black),
            (Point(1, 13), Black),
            (Point(13, 1), Black),
            (Point(13, 13), White),
            (Point(0, 0), White),
        ];
        assert_lines_agree(&square, &stones);
        assert_eq!(points(square.stones(Black)), "B2,B14,J9,N2");
        assert_eq!(points(square.stones(White)), "A1,N14");
        assert_eq!(square.empties().count(), 225 - 6);
    }

    #[test]
    fn test_lines() -> Result<(), String> {
        let square = "H8,I9,J9".parse::<Square>()?;

        // The diagonals shorter than a five are not stored.
        assert!(square.line(Ascending, D_LINE_OMIT).is_some());
        assert!(square.line(Ascending, D_LINE_OMIT - 1).is_none());
        assert!(square.line(Descending, D_LINE_OMIT - 1).is_none());
        assert!(square.line_on(Point(0, 0), Descending).is_none());

        // `lines` and `lines_on` agree with `line`.
        assert_eq!(
            square.lines().count(),
            (RANGE as usize + D_LINE_NUM as usize) * 2
        );
        for (d, i, line) in square.lines() {
            assert_eq!(square.line(d, i), Some(line));
        }
        assert_eq!(square.lines_on(Point(7, 7)).count(), 4);
        assert_eq!(square.lines_on(Point(0, 0)).count(), 3);
        for (d, i, line) in square.lines_on(Point(7, 7)) {
            let index = Point(7, 7).to_index(d);
            assert_eq!(i, index.i);
            assert_eq!(line.stone(index.j), Some(Black));
        }

        Ok(())
    }

    #[test]
    fn test_structures() -> Result<(), String> {
        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x x x o . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;

        // Black's only open two is H6-K9 on the diagonal: H6-H8 is closed
        // by White's H9.
        let twos: Vec<_> = square.structures(Black, Two).collect();
        assert_eq!(twos.len(), 1);
        assert_eq!(points(twos[0].eyes()), "I7,J8");

        // White's H9-J9 is closed by K9, so it is a sword (three stones that
        // can only become a closed four), not a three.
        assert_eq!(square.structures(White, Three).count(), 0);
        let swords: Vec<_> = square.structures(White, Sword).collect();
        assert_eq!(swords.len(), 1);
        assert_eq!(points(swords[0].stones()), "H9,I9,J9");
        assert_eq!(points(swords[0].eyes()), "F9,G9");

        // `structures_on` is the same search, limited to one point's lines.
        let on_h9: Vec<_> = square.structures_on(Point(7, 8), White, Sword).collect();
        assert_eq!(on_h9, swords);
        assert_eq!(square.structures_on(Point(7, 8), Black, Two).count(), 0);

        Ok(())
    }

    #[test]
    fn test_potentials() -> Result<(), String> {
        let square = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x x x o . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Square>()?;
        let potentials = |r, min, exact| -> Vec<(Direction, String, u8)> {
            square
                .potentials(r, min, exact)
                .map(|(i, o)| (i.d, i.to_point().to_string(), o))
                .collect()
        };

        // Line by line, the empty points and what `Line::potentials` gives
        // them: see `potential.rs`.
        let expected = [
            (Vertical, "H4".to_string(), 3),
            (Vertical, "H5".to_string(), 3),
            (Vertical, "H7".to_string(), 3),
            (Ascending, "G5".to_string(), 3),
            (Ascending, "I7".to_string(), 6),
            (Ascending, "J8".to_string(), 6),
            (Ascending, "L10".to_string(), 3),
        ];
        assert_eq!(potentials(Black, 3, true), expected);

        let expected = [
            (Horizontal, "E9".to_string(), 3),
            (Horizontal, "F9".to_string(), 4),
            (Horizontal, "G9".to_string(), 4),
        ];
        assert_eq!(potentials(White, 3, false), expected);

        // `potentials_along` is the same, limited to one point's lines.
        let along: Vec<_> = square
            .potentials_along(Point(7, 8), White, 3, false)
            .collect();
        let all: Vec<_> = square.potentials(White, 3, false).collect();
        assert_eq!(along, all);

        Ok(())
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let mut expected = Square::new();
        expected.put_mut(Black, Point(7, 7));
        expected.put_mut(White, Point(8, 8));
        expected.put_mut(Black, Point(9, 8));

        // Moves, alternating from Black.
        assert_eq!("H8,I9,J9".parse::<Square>()?, expected);
        // Black stones / White stones.
        assert_eq!("H8,J9/I9".parse::<Square>()?, expected);
        // The board as `Display` writes it, row 15 first.
        let s = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x o . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        ";
        assert_eq!(s.parse::<Square>()?, expected);
        assert_eq!(expected.to_string().parse::<Square>()?, expected);

        assert!(". . .\n. . .".parse::<Square>().is_err());
        Ok(())
    }

    #[test]
    fn test_to_pretty_string() {
        let mut square = Square::new();
        square.put_mut(Black, Point(7, 7));
        square.put_mut(White, Point(8, 8));
        square.put_mut(Black, Point(0, 0));
        square.put_mut(White, Point(14, 14));
        let expected = "
15 . . . . . . . . . . . . . . x
14 . . . . . . . . . . . . . . .
13 . . . . . . . . . . . . . . .
12 . . . . . . . . . . . . . . .
11 . . . . . . . . . . . . . . .
10 . . . . . . . . . . . . . . .
 9 . . . . . . . . x . . . . . .
 8 . . . . . . . o . . . . . . .
 7 . . . . . . . . . . . . . . .
 6 . . . . . . . . . . . . . . .
 5 . . . . . . . . . . . . . . .
 4 . . . . . . . . . . . . . . .
 3 . . . . . . . . . . . . . . .
 2 . . . . . . . . . . . . . . .
 1 o . . . . . . . . . . . . . .
   A B C D E F G H I J K L M N O
        "
        .trim();
        assert_eq!(square.to_pretty_string(), expected);
    }
}
