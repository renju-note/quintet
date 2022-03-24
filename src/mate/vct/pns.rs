use super::resolver::*;
use super::searcher::*;
use super::state::State;
use super::table::*;
use crate::board::*;
use crate::mate::mate::*;
use crate::mate::vcf;

pub struct Solver {
    table: Table,
    vcf_solver: vcf::iddfs::Solver,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            table: Table::new(),
            vcf_solver: vcf::iddfs::Solver::init((1..u8::MAX).collect()),
        }
    }

    pub fn solve(&mut self, state: &mut State) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}

impl Searcher for Solver {
    fn table(&mut self) -> &mut Table {
        &mut self.table
    }

    // MEMO: choose_attack|choose_defence does trick;
    // approximate child node's initial pn|dn by inheriting the number of *current* attacks|defences
    // since we don't know the number of *next* defences|attacks

    fn choose_attack(&self, state: &mut State, attacks: &[Point], _threshold: Node) -> Choice {
        let limit = state.limit();
        let mut current = Node::inf_pn(limit);
        let mut next_move: Option<Point> = None;
        let mut next = Node::inf_pn(limit);
        // trick
        let init = Node::init_dn(attacks.len(), limit);
        for &attack in attacks {
            let child = self.table.lookup_next(state, attack).unwrap_or(init);
            current = current.min_pn_sum_dn(child);
            if child.pn < next.pn {
                next_move.replace(attack);
                next = child;
            }
            if current.pn == 0 {
                current = Node::inf_dn(current.limit);
                break;
            }
        }
        let next_threshold = Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        );
        Choice {
            current: current,
            next_move: next_move,
            next_threshold: next_threshold,
        }
    }

    fn choose_defence(&self, state: &mut State, defences: &[Point], _threshold: Node) -> Choice {
        let limit = state.limit();
        let mut current = Node::inf_dn(limit);
        let mut next_move: Option<Point> = None;
        let mut next = Node::inf_dn(limit - 1);
        // trick
        let init = Node::init_pn(defences.len(), limit - 1);
        for &defence in defences {
            let child = self.table.lookup_next(state, defence).unwrap_or(init);
            current = current.min_dn_sum_pn(child);
            if child.dn < next.dn {
                next_move.replace(defence);
                next = child;
            }
            if current.dn == 0 {
                current = Node::inf_pn(current.limit);
                break;
            }
        }
        let next_threshold = Node::new(
            next.pn.checked_add(1).unwrap_or(INF),
            next.dn.checked_add(1).unwrap_or(INF),
            next.limit,
        );
        Choice {
            current: current,
            next_move: next_move,
            next_threshold: next_threshold,
        }
    }
}

impl Resolver for Solver {
    fn table(&self) -> &Table {
        &self.table
    }

    fn solve_vcf(&mut self, state: &mut vcf::State) -> Option<Mate> {
        self.vcf_solver.solve(state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::*;

    #[test]
    fn test_black() -> Result<(), String> {
        // No. 02 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . o x o . . . . .
         . . . . . . x o . x . . . . .
         . . . . . . . x o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let result = solve(&board, Black, 4);
        let expected = Some("F10,G9,I10,G10,H11,H12,G12".to_string());
        assert_eq!(result, expected);

        let result = solve(&board, Black, 3);
        assert!(result.is_none());

        Ok(())
    }

    #[test]
    fn test_white() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . o . . o . . . . .
         . . . . . . o x x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let result = solve(&board, White, 4);
        let expected = Some("I10,I6,I11,I8,J11,J8,G8".to_string());
        assert_eq!(result, expected);

        let result = solve(&board, White, 3);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_counter() -> Result<(), String> {
        // No. 63 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . o . . . . .
         . . . . . . . o x . . . . . .
         . . . x x o . x o . . . . . .
         . . . . . o . o o x . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . x . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let result = solve(&board, White, 4);
        let expected = Some("F7,E8,G8,E6,G5,G7,H6".to_string());
        assert_eq!(result, expected);

        let result = solve(&board, White, 3);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_forbidden_breaker() -> Result<(), String> {
        // No. 68 from 5-moves-to-end problems by Hiroshi Okabe
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . o x o . . . . .
         . . . . . . o x o . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let result = solve(&board, Black, 4);
        let expected = Some("J8,I7,I8,G8,L8,K8,K7".to_string());
        assert_eq!(result, expected);

        let result = solve(&board, Black, 3);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_mise_move() -> Result<(), String> {
        // https://twitter.com/nachirenju/status/1487315157382414336
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . x . . . . . .
         . . . . . . . o . . . . . . .
         . . . . . . x o o . . . . . .
         . . . . . o o o x x . . . . .
         . . . . o x x x x o . . . . .
         . . . x . x o o . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let result = solve(&board, Black, 7);
        let expected = Some("G12,E10,F12,I12,H14,H13,F14,G13,F13,F11,E14,D15,G14".to_string());
        assert_eq!(result, expected);

        let result = solve(&board, Black, 6);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_dual_forbiddens() -> Result<(), String> {
        let board = "
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . x . . . . . . .
         . . . . . . . o o . . . . . .
         . . . . . . . o x . . . . . .
         . . . . . . . x x o . . . . .
         . . . . . . o o x . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
         . . . . . . . . . . . . . . .
        "
        .parse::<Board>()?;

        let result = solve(&board, White, 5);
        let expected = Some("J4,K3,I4,I3,F8,G7,E6,G9,G6".to_string());
        assert_eq!(result, expected);

        let result = solve(&board, White, 4);
        assert_eq!(result, None);

        Ok(())
    }

    fn solve(board: &Board, player: Player, limit: u8) -> Option<String> {
        let state = &mut State::init(board.clone(), player, limit);
        let mut solver = Solver::init();
        solver.solve(state).map(|m| Points(m.path).to_string())
    }
}
