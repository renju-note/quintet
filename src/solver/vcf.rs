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

    pub fn solve(&mut self, board: &Board, black: bool) -> Option<Vec<Point>> {
        let fours = self.analyzer.rows(board, black, RowKind::Four);
        if fours.len() >= 1 {
            let next_move = fours[0].eyes[0];
            return Some(vec![next_move]);
        }

        // Exists counter four
        let opponent_fours = self.analyzer.rows(board, !black, RowKind::Four);
        let opponent_four_eyes = opponent_fours
            .iter()
            .flat_map(|r| r.eyes.to_vec())
            .collect::<Vec<_>>();
        if opponent_four_eyes.len() >= 2 {
            return None;
        } else if opponent_four_eyes.len() == 1 {
            let next_move = opponent_four_eyes[0];
            if black && self.analyzer.forbidden(board, next_move).is_some() {
                return None;
            }
            let next_board = board.put(black, next_move);
            let next_fours = self.analyzer.rows(&next_board, black, RowKind::Four);
            let next_four_eyes = next_fours
                .iter()
                .flat_map(|r| r.eyes.to_vec())
                .collect::<Vec<_>>();
            if next_four_eyes.len() >= 2 {
                return Some(vec![next_move]);
            } else if next_four_eyes.len() == 1 {
                let next2_move = next_four_eyes[0];
                let next2_board = next_board.put(!black, next2_move);
                match self.solve(&next2_board, black) {
                    Some(mut ps) => {
                        let mut result = vec![next_move, next2_move];
                        result.append(&mut ps);
                        return Some(result);
                    }
                    None => {
                        return None;
                    }
                }
            } else {
                return None;
            }
        }

        // Continue four move
        let swords = self.analyzer.rows(board, black, RowKind::Sword);
        for sword in swords {
            for next_move in sword.eyes {
                if black && self.analyzer.forbidden(board, next_move).is_some() {
                    continue;
                }
                let next_board = board.put(black, next_move);
                let next_fours = self.analyzer.rows(&next_board, black, RowKind::Four);
                let next_four_eyes = next_fours
                    .iter()
                    .flat_map(|r| r.eyes.to_vec())
                    .collect::<Vec<_>>();
                if next_four_eyes.len() >= 2 {
                    return Some(vec![next_move]);
                } else if next_four_eyes.len() == 1 {
                    let next2_move = next_four_eyes[0];
                    if !black && self.analyzer.forbidden(&next_board, next2_move).is_some() {
                        return Some(vec![next_move]);
                    }
                    let next2_board = next_board.put(!black, next2_move);
                    match self.solve(&next2_board, black) {
                        Some(mut ps) => {
                            let mut result = vec![next_move, next2_move];
                            result.append(&mut ps);
                            return Some(result);
                        }
                        None => {
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            }
        }
        None
    }
}
