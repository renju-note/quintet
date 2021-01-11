use super::super::board::*;
use super::forbidden::*;
use super::row::*;

pub struct Analyzer {
    row_searcher: RowSearcher,
    forbidden_seacher: ForbiddenSearcher,
}

impl Analyzer {
    pub fn new() -> Analyzer {
        Analyzer {
            row_searcher: RowSearcher::new(),
            forbidden_seacher: ForbiddenSearcher::new(),
        }
    }

    pub fn rows(&mut self, board: &Board, black: bool, kind: RowKind) -> Vec<Row> {
        self.row_searcher.search(board, black, kind)
    }

    pub fn row_eyes(&mut self, board: &Board, black: bool, kind: RowKind) -> Vec<Point> {
        self.row_searcher
            .search(board, black, kind)
            .iter()
            .flat_map(|r| r.eyes.to_vec())
            .collect()
    }

    pub fn forbiddens(&mut self, board: &Board) -> Vec<(ForbiddenKind, Point)> {
        self.forbidden_seacher.search(board, &mut self.row_searcher)
    }

    pub fn forbidden(&mut self, board: &Board, p: Point) -> Option<ForbiddenKind> {
        self.forbidden_seacher
            .judge(board, p, &mut self.row_searcher)
    }
}
