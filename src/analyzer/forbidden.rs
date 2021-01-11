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
    cache: HashMap<OrthogonalLines, Vec<(ForbiddenKind, Point)>>,
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
        let key = board.vertical_lines();
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
        let next = board.put(true, p);
        if self.overline(&next, p, row_searcher) {
            Some(ForbiddenKind::Overline)
        } else if self.double_four(&next, p, row_searcher) {
            Some(ForbiddenKind::DoubleFour)
        } else if self.double_three(&next, p, row_searcher) {
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

    fn overline(&mut self, next: &Board, p: Point, row_searcher: &mut RowSearcher) -> bool {
        let new_overlines = row_searcher.search_containing(&next, true, RowKind::Overline, p);
        new_overlines.len() >= 1
    }

    fn double_four(&mut self, next: &Board, p: Point, row_searcher: &mut RowSearcher) -> bool {
        let new_fours = row_searcher.search_containing(&next, true, RowKind::Four, p);
        if new_fours.len() < 2 {
            return false;
        }
        distinctive(new_fours.iter().collect())
    }

    fn double_three(&mut self, next: &Board, p: Point, row_searcher: &mut RowSearcher) -> bool {
        let new_threes = row_searcher.search_containing(&next, true, RowKind::Three, p);
        if new_threes.len() < 2 {
            return false;
        }
        let truthy_threes = new_threes
            .iter()
            .filter(|r| self.judge(&next, r.eyes[0], row_searcher).is_none())
            .collect::<Vec<_>>();
        if truthy_threes.len() < 2 {
            return false;
        }
        distinctive(truthy_threes)
    }
}

fn distinctive(rows: Vec<&Row>) -> bool {
    let first = rows[0];
    for row in rows.iter().skip(1) {
        if !adjacent(first, row) {
            return true;
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
