use super::super::board::*;
use super::row::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub struct Analyzer {
    line_row_searcher: LineRowSearcher,
    forbiddens_cache: HashMap<String, Vec<(Point, ForbiddenKind)>>,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            line_row_searcher: LineRowSearcher::new(),
            forbiddens_cache: HashMap::new(),
        }
    }

    pub fn get_rows(&mut self, board: &Board, black: bool, kind: RowKind) -> Vec<BoardRow> {
        let mut result = Vec::new();
        for (direction, i, line) in board.iter_lines() {
            let mut rows = self
                .line_row_searcher
                .get(&line, black, kind)
                .iter()
                .map(|lr| BoardRow::from(lr, direction, i))
                .collect();
            result.append(&mut rows);
        }
        result
    }

    pub fn get_forbiddens(&mut self, board: &Board) -> Vec<(Point, ForbiddenKind)> {
        let key = board.to_string();
        match self.forbiddens_cache.get(&key) {
            Some(result) => result.to_vec(),
            None => {
                let result = self.forbiddens(board);
                self.forbiddens_cache.insert(key, result.to_vec());
                result
            }
        }
    }

    pub fn forbiddens(&mut self, board: &Board) -> Vec<(Point, ForbiddenKind)> {
        (1..=BOARD_SIZE)
            .flat_map(|x| (1..=BOARD_SIZE).map(move |y| Point { x: x, y: y }))
            .map(|p| (p, self.forbidden(board, p)))
            .filter(|(_, okind)| okind.is_some())
            .map(|(p, okind)| (p, okind.unwrap()))
            .collect()
    }

    pub fn forbidden(&mut self, board: &Board, p: Point) -> Option<ForbiddenKind> {
        if self.overline(board, p) {
            Some(ForbiddenKind::Overline)
        } else if self.double_four(board, p) {
            Some(ForbiddenKind::DoubleFour)
        } else if self.double_three(board, p) {
            Some(ForbiddenKind::DoubleThree)
        } else {
            None
        }
    }

    fn overline(&mut self, board: &Board, p: Point) -> bool {
        let next = board.put(true, p);
        self.get_rows(&next, true, RowKind::Overline)
            .iter()
            .find(|&br| br.overlap(&p))
            .is_some()
    }

    fn double_four(&mut self, board: &Board, p: Point) -> bool {
        let next = board.put(true, p);
        let fours = self.get_rows(&next, true, RowKind::Four);
        let new_fours: Vec<_> = fours.iter().filter(|&br| br.overlap(&p)).collect();
        if new_fours.len() < 2 {
            return false;
        }
        distinctive(new_fours)
    }

    fn double_three(&mut self, board: &Board, p: Point) -> bool {
        let next = board.put(true, p);
        let threes = self.get_rows(&next, true, RowKind::Three);
        let new_threes: Vec<_> = threes.iter().filter(|&br| br.overlap(&p)).collect();
        if new_threes.len() < 2 {
            return false;
        }
        let truthy_threes: Vec<_> = new_threes
            .iter()
            .filter(|&r| self.forbidden(&next, r.eyes[0]).is_none())
            .map(|&r| r)
            .collect();
        distinctive(truthy_threes)
    }
}

fn distinctive(srows: Vec<&BoardRow>) -> bool {
    let mut prev: Option<&BoardRow> = None;
    for s in srows {
        match prev {
            None => (),
            Some(p) => {
                if !adjacent(p, s) {
                    return true;
                }
            }
        }
        prev = Some(s);
    }
    false
}

fn adjacent(a: &BoardRow, b: &BoardRow) -> bool {
    if a.direction != b.direction {
        return false;
    }
    let (xd, yd) = (
        a.start.x as i32 - b.start.x as i32,
        a.start.y as i32 - b.start.y as i32,
    );
    match a.direction {
        Direction::Vertical => xd == 0 && yd.abs() == 1,
        Direction::Horizontal => xd.abs() == 1 && yd == 0,
        Direction::Ascending => xd.abs() == 1 && xd == yd,
        Direction::Descending => xd.abs() == 1 && xd == -yd,
    }
}
