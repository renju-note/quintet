use super::super::board::*;
use super::row::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ForbiddenKind {
    DoubleThree,
    DoubleFour,
    Overline,
}

pub struct ForbiddenSearcher {
    cache: HashMap<String, Vec<(ForbiddenKind, Point)>>,
}

impl ForbiddenSearcher {
    pub fn new() -> ForbiddenSearcher {
        ForbiddenSearcher {
            cache: HashMap::new(),
        }
    }

    pub fn search(
        &mut self,
        board: &Board,
        row_searcher: &mut RowSearcher,
    ) -> Vec<(ForbiddenKind, Point)> {
        let key = board.to_string();
        match self.cache.get(&key) {
            Some(result) => result.to_vec(),
            None => {
                let result = self.all(board, row_searcher);
                self.cache.insert(key, result.to_vec());
                result
            }
        }
    }

    pub fn judge(
        &mut self,
        board: &Board,
        p: Point,
        row_searcher: &mut RowSearcher,
    ) -> Option<ForbiddenKind> {
        if self.overline(board, p, row_searcher) {
            Some(ForbiddenKind::Overline)
        } else if self.double_four(board, p, row_searcher) {
            Some(ForbiddenKind::DoubleFour)
        } else if self.double_three(board, p, row_searcher) {
            Some(ForbiddenKind::DoubleThree)
        } else {
            None
        }
    }

    fn all(
        &mut self,
        board: &Board,
        row_searcher: &mut RowSearcher,
    ) -> Vec<(ForbiddenKind, Point)> {
        (1..=BOARD_SIZE)
            .flat_map(|x| (1..=BOARD_SIZE).map(move |y| Point { x: x, y: y }))
            .map(|p| (self.judge(board, p, row_searcher), p))
            .filter(|(k, _)| k.is_some())
            .map(|(k, p)| (k.unwrap(), p))
            .collect()
    }

    fn overline(&mut self, board: &Board, p: Point, row_searcher: &mut RowSearcher) -> bool {
        let next = board.put(true, p);
        row_searcher
            .search(&next, true, RowKind::Overline)
            .iter()
            .find(|&br| br.overlap(&p))
            .is_some()
    }

    fn double_four(&mut self, board: &Board, p: Point, row_searcher: &mut RowSearcher) -> bool {
        let next = board.put(true, p);
        let fours = row_searcher.search(&next, true, RowKind::Four);
        let new_fours: Vec<_> = fours.iter().filter(|&br| br.overlap(&p)).collect();
        if new_fours.len() < 2 {
            return false;
        }
        distinctive(new_fours)
    }

    fn double_three(&mut self, board: &Board, p: Point, row_searcher: &mut RowSearcher) -> bool {
        let next = board.put(true, p);
        let threes = row_searcher.search(&next, true, RowKind::Three);
        let new_threes: Vec<_> = threes.iter().filter(|&br| br.overlap(&p)).collect();
        if new_threes.len() < 2 {
            return false;
        }
        let truthy_threes: Vec<_> = new_threes
            .iter()
            .filter(|&r| self.judge(&next, r.eyes[0], row_searcher).is_none())
            .map(|&r| r)
            .collect();
        distinctive(truthy_threes)
    }
}

fn distinctive(rows: Vec<&Row>) -> bool {
    let mut first: Option<&Row> = None;
    for r in rows {
        match first {
            None => {
                first = Some(r);
            }
            Some(p) => {
                if !adjacent(p, r) {
                    return true;
                }
            }
        }
    }
    false
}

fn adjacent(a: &Row, b: &Row) -> bool {
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
