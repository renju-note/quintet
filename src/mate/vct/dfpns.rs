/*
Df-pn algorithm is proposed in the following paper:

Nagai, Ayumu, and Hiroshi Imai.
"Proof for the equivalence between some best-first algorithms and depth-first algorithms for AND/OR trees."
IEICE TRANSACTIONS on Information and Systems 85.10 (2002): 1645-1653.
*/
use super::state::State;
use super::table::*;
use crate::board::*;
use crate::mate::game::*;

// MEMO: Debug printing example is 6e2bace

pub struct Searcher {
    table: Table,
}

impl Searcher {
    pub fn init() -> Self {
        Self {
            table: Table::new(),
        }
    }

    pub fn search(mut self, state: &mut State) -> Option<Table> {
        let node = self.search_limit(state, Node::root(state.limit));
        if node.pn != 0 {
            return None;
        }
        Some(self.table)
    }

    fn search_limit(&mut self, state: &mut State, threshold: Node) -> Node {
        if state.limit == 0 {
            return Node::inf_pn(state.limit);
        }
        self.search_attacks(state, threshold)
    }

    fn search_attacks(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::inf_pn(state.limit),
                Forced(m) => self.expand_attack(state, m, threshold),
            };
        }

        if state.solve_attacker_vcf().is_some() {
            return Node::inf_dn(state.limit);
        }

        let maybe_threat = state.solve_defender_vcf();

        let attacks = state.sorted_attacks(maybe_threat);

        loop {
            let (current, selected, next1, next2) = self.select_attack(state, &attacks);
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            let next_threshold = self.next_threshold_attack(threshold, current, next1, next2);
            self.expand_attack(state, selected.unwrap(), next_threshold);
        }
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, threshold: Node) -> Node {
        let last2_move = state.game().last2_move();
        state.play(attack);
        let result = self.search_defences(state, threshold);
        self.table.insert(state, result.clone());
        state.undo(last2_move);
        result
    }

    fn search_defences(&mut self, state: &mut State, threshold: Node) -> Node {
        if let Some(event) = state.game().check_event() {
            return match event {
                Defeated(_) => Node::inf_dn(state.limit),
                Forced(m) => self.expand_defence(state, m, threshold),
            };
        }

        let maybe_threat = state.solve_attacker_vcf();
        if maybe_threat.is_none() {
            return Node::inf_pn(state.limit);
        }

        if state.solve_defender_vcf().is_some() {
            return Node::inf_pn(state.limit);
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());

        loop {
            let (current, selected, next1, next2) = self.select_defence(state, &defences);
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            let next_threshold = self.next_threshold_defence(threshold, current, next1, next2);
            self.expand_defence(state, selected.unwrap(), next_threshold);
        }
    }

    fn expand_defence(&mut self, state: &mut State, defence: Point, threshold: Node) -> Node {
        let last2_move = state.game().last2_move();
        state.play(defence);
        let result = self.search_limit(state, threshold);
        self.table.insert(state, result.clone());
        state.undo(last2_move);
        result
    }

    fn select_attack(
        &self,
        state: &mut State,
        attacks: &[Point],
    ) -> (Node, Option<Point>, Node, Node) {
        let limit = state.limit;
        let mut current = Node::inf_pn(limit);
        let mut selected: Option<Point> = None;
        let mut next1 = Node::inf_pn(limit);
        let mut next2 = Node::inf_pn(limit);
        for &attack in attacks {
            let child = self.table.lookup_next(state, attack);
            current = current.agg_or(child);
            if child.pn < next1.pn {
                selected.replace(attack);
                next2 = next1;
                next1 = child;
            } else if child.pn < next2.pn {
                next2 = child;
            }
            if current.pn == 0 {
                current = Node::inf_dn(current.limit);
                break;
            }
        }
        (current, selected, next1, next2)
    }

    fn next_threshold_attack(
        &self,
        threshold: Node,
        current: Node,
        next1: Node,
        next2: Node,
    ) -> Node {
        let pn = threshold.pn.min(next2.pn.checked_add(1).unwrap_or(INF));
        let dn = (threshold.dn - current.dn)
            .checked_add(next1.dn)
            .unwrap_or(INF);
        Node::new(pn, dn, current.limit)
    }

    fn select_defence(
        &self,
        state: &mut State,
        defences: &[Point],
    ) -> (Node, Option<Point>, Node, Node) {
        let limit = state.limit;
        let mut current = Node::inf_dn(limit);
        let mut selected: Option<Point> = None;
        let mut next1 = Node::inf_dn(limit - 1);
        let mut next2 = Node::inf_dn(limit - 1);
        for &defence in defences {
            let child = self.table.lookup_next(state, defence);
            current = current.agg_and(child);
            if child.dn < next1.dn {
                selected.replace(defence);
                next2 = next1;
                next1 = child;
            } else if child.dn < next2.dn {
                next2 = child;
            }
            if current.dn == 0 {
                current = Node::inf_pn(current.limit);
                break;
            }
        }
        (current, selected, next1, next2)
    }

    fn next_threshold_defence(
        &self,
        threshold: Node,
        current: Node,
        next1: Node,
        next2: Node,
    ) -> Node {
        let pn = (threshold.pn - current.pn)
            .checked_add(next1.pn)
            .unwrap_or(INF);
        let dn = threshold.dn.min(next2.dn.checked_add(1).unwrap_or(INF));
        Node::new(pn, dn, current.limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::*;
    use crate::mate::vct::resolver::Resolver;

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
        let expected = Some("J4,G7,I4,I3,E6,G4,G6".to_string());
        assert_eq!(result, expected);

        let result = solve(&board, White, 4);
        assert_eq!(result, None);

        Ok(())
    }

    fn solve(board: &Board, player: Player, limit: u8) -> Option<String> {
        let state = &mut State::init(board.clone(), player, limit);
        let searcher = Searcher::init();
        let may_table = searcher.search(state);
        may_table
            .and_then(|table| {
                let mut resolver = Resolver::init(table);
                resolver.resolve(state)
            })
            .map(|m| Points(m.path).to_string())
    }
}
