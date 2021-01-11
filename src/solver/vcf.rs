use super::super::analyzer::*;
use super::super::board::*;

pub struct VCFSolver {
    analyzer: Analyzer,
}

impl VCFSolver {
    pub fn new() -> VCFSolver {
        VCFSolver {
            analyzer: Analyzer::new(),
        }
    }

    pub fn solve(
        &mut self,
        board: &Board,
        black: bool,
        depth_limit: u8,
        shortest: bool,
    ) -> Option<Vec<Point>> {
        // Exists self four
        let fours = self.analyzer.rows(board, black, RowKind::Four);
        if fours.len() >= 1 {
            return Some(vec![]);
        }

        if depth_limit == 0 {
            return None;
        }

        // Exists opponent's four
        let op_four_eyes = self.analyzer.row_eyes(board, !black, RowKind::Four);
        if op_four_eyes.len() >= 2 {
            return None;
        } else if op_four_eyes.len() == 1 {
            let next_move = op_four_eyes[0];
            return self.solve_one(board, black, next_move, depth_limit, shortest);
        }

        // Continue four move
        let next_move_cands = self.analyzer.row_eyes(board, black, RowKind::Sword);
        let mut min_depth = depth_limit;
        let mut result: Option<Vec<Point>> = None;
        for next_move in next_move_cands {
            match self.solve_one(board, black, next_move, min_depth, shortest) {
                Some(ps) => {
                    if !shortest {
                        return Some(ps);
                    }
                    let depth = ((ps.len() + 1) / 2) as u8;
                    if depth < min_depth {
                        min_depth = depth;
                        result = Some(ps);
                    }
                }
                None => continue,
            }
        }
        result
    }

    fn solve_one(
        &mut self,
        board: &Board,
        black: bool,
        next_move: Point,
        depth_limit: u8,
        shortest: bool,
    ) -> Option<Vec<Point>> {
        if black && self.analyzer.forbidden(board, next_move).is_some() {
            return None;
        }
        let next_board = board.put(black, next_move);
        let next_four_eyes = self.analyzer.row_eyes(&next_board, black, RowKind::Four);
        if next_four_eyes.len() >= 2 {
            Some(vec![next_move])
        } else if next_four_eyes.len() == 1 {
            let next2_move = next_four_eyes[0];
            if !black && self.analyzer.forbidden(&next_board, next2_move).is_some() {
                return Some(vec![next_move]);
            }
            let next2_board = next_board.put(!black, next2_move);
            self.solve(&next2_board, black, depth_limit - 1, shortest)
                .map(|mut ps| {
                    let mut result = vec![next_move, next2_move];
                    result.append(&mut ps);
                    result
                })
        } else {
            None
        }
    }
}
