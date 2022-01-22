use super::fundamentals::Player::*;
use super::fundamentals::*;
use super::line::*;
use super::point::Direction::*;
use super::point::*;
use super::row::*;
use super::slot::*;
use std::fmt;
use std::str::FromStr;

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

    pub fn from_points(blacks: &Points, whites: &Points) -> Self {
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
        let vidx = p.to_index(Vertical);
        Self::line_idx(vidx).map(|i| self.vlines[i].put_mut(player, vidx.j));

        let hidx = p.to_index(Horizontal);
        Self::line_idx(hidx).map(|i| self.hlines[i].put_mut(player, hidx.j));

        let aidx = p.to_index(Ascending);
        Self::line_idx(aidx).map(|i| self.alines[i].put_mut(player, aidx.j));

        let didx = p.to_index(Descending);
        Self::line_idx(didx).map(|i| self.dlines[i].put_mut(player, didx.j));
    }

    pub fn remove_mut(&mut self, p: Point) {
        let vidx = p.to_index(Vertical);
        Self::line_idx(vidx).map(|i| self.vlines[i].remove_mut(vidx.j));

        let hidx = p.to_index(Horizontal);
        Self::line_idx(hidx).map(|i| self.hlines[i].remove_mut(hidx.j));

        let aidx = p.to_index(Ascending);
        Self::line_idx(aidx).map(|i| self.alines[i].remove_mut(aidx.j));

        let didx = p.to_index(Descending);
        Self::line_idx(didx).map(|i| self.dlines[i].remove_mut(didx.j));
    }

    pub fn stone(&self, p: Point) -> Option<Player> {
        let vidx = p.to_index(Vertical);
        self.vlines[vidx.i as usize].stone(vidx.j)
    }

    pub fn stones(&self, player: Player) -> Vec<Point> {
        self.vlines
            .iter()
            .enumerate()
            .map(|(i, l)| {
                l.stones(player)
                    .into_iter()
                    .map(move |j| Index::new(Vertical, i as u8, j).to_point())
            })
            .flatten()
            .collect()
    }

    pub fn rows(&self, player: Player, kind: RowKind) -> impl Iterator<Item = Row> + '_ {
        let potential = kind.potential();
        self.iter_lines()
            .filter(move |(_, _, l)| l.potential_cap(player) >= potential)
            .map(move |(d, i, l)| Row::from_slots(l.slots(), d, i, player, kind))
            .flatten()
    }

    pub fn rows_on(
        &self,
        player: Player,
        kind: RowKind,
        p: Point,
    ) -> impl Iterator<Item = Row> + '_ {
        let potential = kind.potential();
        self.iter_lines_on(p)
            .filter(move |(_, _, l)| l.potential_cap(player) >= potential)
            .map(move |(d, i, l)| {
                let j = p.to_index(d).j;
                Row::from_slots(l.slots_on(j), d, i, player, kind)
            })
            .flatten()
    }

    // https://depth-first.com/articles/2020/06/22/returning-rust-iterators/
    pub fn slots(&self, player: Player, potential: u8) -> impl Iterator<Item = (Index, Slot)> + '_ {
        self.iter_lines()
            .filter(move |(_, _, l)| l.potential_cap(player) >= potential)
            .map(|(d, i, l)| l.slots().map(move |(j, s)| (Index::new(d, i, j), s)))
            .flatten()
            .filter(move |(_, s)| s.potential(player) == potential)
    }

    pub fn slots_on(
        &self,
        player: Player,
        potential: u8,
        p: Point,
    ) -> impl Iterator<Item = (Index, Slot)> + '_ {
        self.iter_lines_on(p)
            .filter(move |(_, _, l)| l.potential_cap(player) >= potential)
            .map(move |(d, i, l)| {
                let j = p.to_index(d).j;
                l.slots_on(j).map(move |(j, s)| (Index::new(d, i, j), s))
            })
            .flatten()
            .filter(move |(_, s)| s.potential(player) == potential)
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

    fn iter_lines(&self) -> impl Iterator<Item = (Direction, u8, &Line)> {
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
            .map(|(i, l)| (Ascending, (i as u8 + DIAGONAL_OMIT), l));

        let diter = self
            .dlines
            .iter()
            .enumerate()
            .map(|(i, l)| (Descending, (i as u8 + DIAGONAL_OMIT), l));

        viter.chain(hiter).chain(aiter).chain(diter)
    }

    fn iter_lines_on(&self, p: Point) -> impl Iterator<Item = (Direction, u8, &Line)> {
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

    fn line_idx(index: Index) -> Option<usize> {
        let i = index.i;
        match index.direction {
            Vertical => Some(i as usize),
            Horizontal => Some(i as usize),
            _ => {
                if DIAGONAL_OMIT <= i && i < DIAGONAL_OMIT + D_LINE_NUM {
                    Some((i - DIAGONAL_OMIT) as usize)
                } else {
                    None
                }
            }
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rev_hlines: Vec<_> = self.hlines.iter().rev().collect();
        let s = lines_to_string(&rev_hlines);
        f.write_str(&s)
    }
}

fn lines_to_string(lines: &[&Line]) -> String {
    lines
        .iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

impl FromStr for Square {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("/") {
            from_str_points(s)
        } else if s.contains(",") {
            from_str_moves(s)
        } else {
            from_str_display(s)
        }
    }
}

fn from_str_moves(s: &str) -> Result<Square, &'static str> {
    let moves = s.trim().parse::<Points>()?;
    Ok(Square::from_moves(&moves))
}

fn from_str_points(s: &str) -> Result<Square, &'static str> {
    let codes = s.trim().split("/").collect::<Vec<_>>();
    if codes.len() != 2 {
        return Err("Unknown format.");
    }
    let blacks = codes[0].parse::<Points>()?;
    let whites = codes[1].parse::<Points>()?;
    Ok(Square::from_points(&blacks, &whites))
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
    for (i, hline) in hlines_rev.iter().rev().enumerate() {
        if hline.size != BOARD_SIZE {
            return Err("Wrong line size");
        }
        for j in 0..hline.size {
            hline.stone(j).map(|player| {
                let point = Index::new(Horizontal, i as u8, j as u8).to_point();
                square.put_mut(player, point)
            });
        }
    }
    Ok(square)
}

const DIAGONAL_OMIT: u8 = 5 - 1;
const D_LINE_NUM: u8 = (BOARD_SIZE - DIAGONAL_OMIT) * 2 - 1; // 21

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

#[cfg(test)]
mod tests {
    use super::RowKind::*;
    use super::*;

    #[test]
    fn test_put_remove() {
        let mut square = Square::new();
        square.put_mut(Black, Point(7, 7));
        square.put_mut(White, Point(8, 8));
        square.put_mut(Black, Point(9, 8));
        square.put_mut(Black, Point(1, 1));
        square.put_mut(White, Point(1, 13));
        square.put_mut(Black, Point(13, 1));
        square.put_mut(White, Point(13, 13));

        let result = lines_to_string(&square.hlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . o . . . . . . . . . . . o .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . o . . . . . . .
             . . . . . . . . x o . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . x . . . . . . . . . . . x .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.vlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . o . . . . . . . . . . . x .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . o . . . . . . .
             . . . . . . . . x . . . . . .
             . . . . . . . . o . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . o . . . . . . . . . . . x .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.alines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . .
             . . . . . .
             . . . . . . .
             . . . . . . . .
             . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . . . .
             . . . . . . . . . . . .
             . . . . . . . . . . . . .
             . . . . . . . . . . . . . .
             . o . . . . . o x . . . . x .
             . . . . . . . . o . . . . .
             . . . . . . . . . . . . .
             . . . . . . . . . . . .
             . . . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . .
             . . . . . . . .
             . . . . . . .
             . . . . . .
             . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.dlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . .
             . . . . . .
             . . . . . . .
             . . . . . . . .
             . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . . . .
             . . . . . . . . . . . .
             . . . . . . . . . . . . .
             . . . . . . . . . . . . . .
             . x . . . . . o . . . . . o .
             . . . . . . . . . . . . . .
             . . . . . . x . . . . . .
             . . . . . . o . . . . .
             . . . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . .
             . . . . . . . .
             . . . . . . .
             . . . . . .
             . . . . .
            ",
        );
        assert_eq!(result, expected);

        square.remove_mut(Point(7, 7));
        square.remove_mut(Point(8, 8));
        square.remove_mut(Point(9, 9));

        let result = lines_to_string(&square.hlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . o . . . . . . . . . . . o .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . o . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . x . . . . . . . . . . . x .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.vlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . . . . . . . . . . . .
             . o . . . . . . . . . . . x .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . o . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . o . . . . . . . . . . . x .
             . . . . . . . . . . . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.alines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . .
             . . . . . .
             . . . . . . .
             . . . . . . . .
             . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . . . .
             . . . . . . . . . . . .
             . . . . . . . . . . . . .
             . . . . . . . . . . . . . .
             . o . . . . . . . . . . . x .
             . . . . . . . . o . . . . .
             . . . . . . . . . . . . .
             . . . . . . . . . . . .
             . . . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . .
             . . . . . . . .
             . . . . . . .
             . . . . . .
             . . . . .
            ",
        );
        assert_eq!(result, expected);

        let result = lines_to_string(&square.dlines.iter().collect::<Vec<_>>());
        let expected = trim_lines_string(
            "
             . . . . .
             . . . . . .
             . . . . . . .
             . . . . . . . .
             . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . . . .
             . . . . . . . . . . . .
             . . . . . . . . . . . . .
             . . . . . . . . . . . . . .
             . x . . . . . . . . . . . o .
             . . . . . . . . . . . . . .
             . . . . . . . . . . . . .
             . . . . . . o . . . . .
             . . . . . . . . . . .
             . . . . . . . . . .
             . . . . . . . . .
             . . . . . . . .
             . . . . . . .
             . . . . . .
             . . . . .
            ",
        );
        assert_eq!(result, expected);
    }

    #[test]
    fn test_stone_and_stones() -> Result<(), String> {
        let square = "
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
        "
        .parse::<Square>()?;

        assert_eq!(square.stone(Point(7, 7)), Some(Black));
        assert_eq!(square.stone(Point(8, 8)), Some(White));
        assert_eq!(square.stone(Point(9, 9)), None);

        assert_eq!(square.stones(Black), [Point(7, 7), Point(9, 8)]);
        assert_eq!(square.stones(White), [Point(8, 8)]);
        Ok(())
    }

    #[test]
    fn test_rows_and_so_on() -> Result<(), String> {
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
        let black_twos = [Row::new(
            Ascending,
            Point(7, 5),
            Point(10, 8),
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

        assert_eq!(square.rows(Black, Two).collect::<Vec<_>>(), black_twos);
        assert_eq!(square.rows(White, Sword).collect::<Vec<_>>(), white_swords);

        assert_eq!(
            square.rows_on(Black, Two, Point(10, 8)).collect::<Vec<_>>(),
            black_twos
        );
        assert_eq!(
            square.rows_on(Black, Two, Point(7, 7)).collect::<Vec<_>>(),
            []
        );

        Ok(())
    }

    #[test]
    fn test_to_pretty_string() {
        let mut square = Square::new();
        square.put_mut(Black, Point(7, 7));
        square.put_mut(White, Point(8, 8));
        square.put_mut(Black, Point(9, 8));
        square.put_mut(Black, Point(0, 0));
        square.put_mut(White, Point(0, 14));
        square.put_mut(Black, Point(14, 0));
        square.put_mut(White, Point(14, 14));
        let expected = "
15 x . . . . . . . . . . . . . x
14 . . . . . . . . . . . . . . .
13 . . . . . . . . . . . . . . .
12 . . . . . . . . . . . . . . .
11 . . . . . . . . . . . . . . .
10 . . . . . . . . . . . . . . .
 9 . . . . . . . . x o . . . . .
 8 . . . . . . . o . . . . . . .
 7 . . . . . . . . . . . . . . .
 6 . . . . . . . . . . . . . . .
 5 . . . . . . . . . . . . . . .
 4 . . . . . . . . . . . . . . .
 3 . . . . . . . . . . . . . . .
 2 . . . . . . . . . . . . . . .
 1 o . . . . . . . . . . . . . o
   A B C D E F G H I J K L M N O
        "
        .trim();
        assert_eq!(square.to_pretty_string(), expected);
    }

    #[test]
    fn test_parse() -> Result<(), String> {
        let result = "H8,I9,J9".parse::<Square>()?;
        let mut expected = Square::new();
        expected.put_mut(Black, Point(7, 7));
        expected.put_mut(White, Point(8, 8));
        expected.put_mut(Black, Point(9, 8));
        assert_eq!(result, expected);

        let result = "H8,J9/I9".parse::<Square>()?;
        let mut expected = Square::new();
        expected.put_mut(Black, Point(7, 7));
        expected.put_mut(White, Point(8, 8));
        expected.put_mut(Black, Point(9, 8));
        assert_eq!(result, expected);

        let result = "
         x . . . . . . . . . . . . . x
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
         o . . . . . . . . . . . . . o
        "
        .parse::<Square>()?;
        let mut expected = Square::new();
        expected.put_mut(Black, Point(7, 7));
        expected.put_mut(White, Point(8, 8));
        expected.put_mut(Black, Point(9, 8));
        expected.put_mut(Black, Point(0, 0));
        expected.put_mut(White, Point(0, 14));
        expected.put_mut(Black, Point(14, 0));
        expected.put_mut(White, Point(14, 14));
        assert_eq!(result, expected);

        Ok(())
    }

    #[test]
    fn test_to_string() {
        let mut square = Square::new();
        square.put_mut(Black, Point(7, 7));
        square.put_mut(White, Point(8, 8));
        square.put_mut(Black, Point(9, 8));
        square.put_mut(Black, Point(0, 0));
        square.put_mut(White, Point(0, 14));
        square.put_mut(Black, Point(14, 0));
        square.put_mut(White, Point(14, 14));
        let expected = trim_lines_string(
            "
             x . . . . . . . . . . . . . x
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
             o . . . . . . . . . . . . . o
            ",
        );
        assert_eq!(square.to_string(), expected);
    }

    fn trim_lines_string(s: &str) -> String {
        s.trim()
            .split("\n")
            .map(|ls| " ".to_string() + ls.trim())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
