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
use crate::mate::vcf;

// MEMO: Debug printing example is 6e2bace

pub struct Searcher {
    table: Table,
    attacker_vcf_solver: vcf::iddfs::Solver,
    defender_vcf_solver: vcf::iddfs::Solver,
}

impl Searcher {
    pub fn init() -> Self {
        Self {
            table: Table::new(),
            attacker_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
            defender_vcf_solver: vcf::iddfs::Solver::init([1].to_vec()),
        }
    }

    pub fn search(mut self, state: &mut State, max_depth: u8) -> Option<Table> {
        let node = self.search_limit(state, Node::root(max_depth), max_depth);
        if node.pn != 0 {
            return None;
        }
        Some(self.table)
    }

    fn search_limit(&mut self, state: &mut State, threshold: Node, limit: u8) -> Node {
        if limit == 0 {
            return Node::inf_pn(limit);
        }
        self.search_attacks(state, threshold, limit)
    }

    fn search_attacks(&mut self, state: &mut State, threshold: Node, limit: u8) -> Node {
        if let Some(stage) = state.game().check_stage() {
            return match stage {
                End(_) => Node::inf_pn(limit),
                Forced(m) => self.expand_attack(state, m, threshold, limit),
            };
        }

        let vcf_state = &mut state.as_vcf();
        vcf_state.set_limit(limit);
        if self.attacker_vcf_solver.solve(vcf_state).is_some() {
            return Node::inf_dn(limit);
        }

        let threat_state = &mut state.as_threat();
        threat_state.set_limit(u8::MAX);
        let maybe_threat = self.defender_vcf_solver.solve(threat_state);

        let attacks = state.sorted_attacks(maybe_threat);

        loop {
            let (current, selected, next1, next2) = self.select_attack(state, &attacks, limit);
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            let next_threshold = self.next_threshold_attack(threshold, current, next1, next2);
            self.expand_attack(state, selected.unwrap(), next_threshold, limit);
        }
    }

    fn expand_attack(
        &mut self,
        state: &mut State,
        attack: Point,
        threshold: Node,
        limit: u8,
    ) -> Node {
        let last2_move = state.game().last2_move();
        state.play(attack);
        let hash = state.game().zobrist_hash(limit);
        let current = self.table.lookup(hash, limit);
        if current.pn >= threshold.pn || current.dn >= threshold.dn {
            state.undo(last2_move);
            return current;
        }
        let result = self.search_defences(state, threshold, limit);
        self.table.insert(hash, result.clone());
        state.undo(last2_move);
        result
    }

    fn search_defences(&mut self, state: &mut State, threshold: Node, limit: u8) -> Node {
        if let Some(stage) = state.game().check_stage() {
            return match stage {
                End(_) => Node::inf_dn(limit),
                Forced(m) => self.expand_defence(state, m, threshold, limit),
            };
        }

        let threat_state = &mut state.as_threat();
        threat_state.set_limit(limit - 1);
        let maybe_threat = self.attacker_vcf_solver.solve(threat_state);
        if maybe_threat.is_none() {
            return Node::inf_pn(limit);
        }

        let vcf_state = &mut state.as_vcf();
        vcf_state.set_limit(u8::MAX);
        if self.defender_vcf_solver.solve(vcf_state).is_some() {
            return Node::inf_pn(limit);
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());

        loop {
            let (current, selected, next1, next2) = self.select_defence(state, &defences, limit);
            if current.pn >= threshold.pn || current.dn >= threshold.dn {
                return current;
            }
            let next_threshold = self.next_threshold_defence(threshold, current, next1, next2);
            self.expand_defence(state, selected.unwrap(), next_threshold, limit);
        }
    }

    fn expand_defence(
        &mut self,
        state: &mut State,
        defence: Point,
        threshold: Node,
        limit: u8,
    ) -> Node {
        let last2_move = state.game().last2_move();
        state.play(defence);
        let hash = state.game().zobrist_hash(limit);
        let limit = limit - 1;
        let current = self.table.lookup(hash, limit);
        if current.pn >= threshold.pn || current.dn >= threshold.dn {
            state.undo(last2_move);
            return current;
        }
        let result = self.search_limit(state, threshold, limit);
        self.table.insert(hash, result.clone());
        state.undo(last2_move);
        result
    }

    fn select_attack(
        &self,
        state: &State,
        attacks: &[Point],
        limit: u8,
    ) -> (Node, Option<Point>, Node, Node) {
        let mut game = state.game().clone();
        let mut current = Node::inf_pn(limit);
        let mut selected: Option<Point> = None;
        let mut next1 = Node::inf_pn(limit);
        let mut next2 = Node::inf_pn(limit);
        for &attack in attacks {
            let child = self.table.lookup_child(&mut game, attack, limit);
            current = Node::new(
                current.pn.min(child.pn),
                current.dn.checked_add(child.dn).unwrap_or(INF),
                current.limit.min(child.limit),
            );
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
        Node::new(
            threshold.pn.min(next2.pn.checked_add(1).unwrap_or(INF)),
            (threshold.dn - current.dn)
                .checked_add(next1.dn)
                .unwrap_or(INF),
            current.limit,
        )
    }

    fn select_defence(
        &self,
        state: &State,
        defences: &[Point],
        limit: u8,
    ) -> (Node, Option<Point>, Node, Node) {
        let mut game = state.game().clone();
        let mut current = Node::inf_dn(limit);
        let mut selected: Option<Point> = None;
        let mut next1 = Node::inf_dn(limit);
        let mut next2 = Node::inf_dn(limit);
        for &defence in defences {
            let child = self.table.lookup_child(&mut game, defence, limit);
            current = Node::new(
                current.pn.checked_add(child.pn).unwrap_or(INF),
                current.dn.min(child.dn),
                current.limit.min(child.limit),
            );
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
        Node::new(
            (threshold.pn - current.pn)
                .checked_add(next1.pn)
                .unwrap_or(INF),
            threshold.dn.min(next2.dn.checked_add(1).unwrap_or(INF)),
            current.limit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::*;
    use crate::mate::vct::resolver::Resolver;

    #[test]
    fn test_black() -> Result<(), String> {
        // No. 02 from 5-moves-to-win problems by Hiroshi Okabe
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
        let state = &mut State::init(board.clone(), Black, 4);

        let result = solve(state, 4);
        let expected = Some("F10,G9,I10,G10,H11,H12,G12".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3);
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
        let state = &mut State::init(board.clone(), White, 4);

        let result = solve(state, 4);
        let expected = Some("I10,I6,I11,I8,J11,J8,G8".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_counter() -> Result<(), String> {
        // No. 63 from 5-moves-to-win problems by Hiroshi Okabe
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
        let state = &mut State::init(board.clone(), White, 4);

        let result = solve(state, 4);
        let expected = Some("F7,E8,G8,E6,G5,G7,H6".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_forbidden_breaker() -> Result<(), String> {
        // No. 68 from 5-moves-to-win problems by Hiroshi Okabe
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
        let state = &mut State::init(board.clone(), Black, 4);

        let result = solve(state, 4);
        let expected = Some("J8,I7,I8,G8,L8,K8,K7".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3);
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
        let state = &mut State::init(board.clone(), Black, 7);

        let result = solve(state, 7);
        let expected = Some("G12,E10,F12,I12,H14,H13,F14,G13,F13,F11,E14,D15,G14".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 6);
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
        let state = &mut State::init(board.clone(), White, 5);

        let result = solve(state, 5);
        let expected = Some("J4,G7,I4,I3,E6,G4,G6".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 4);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_search_black() -> Result<(), String> {
        // No. 02 from 5-moves-to-win problems by Hiroshi Okabe
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
        let state = &mut State::init(board.clone(), Black, 4);

        let result = solve(state, 4);
        let expected = Some("F10,G9,I10,G10,H11,H12,G12".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_search_white() -> Result<(), String> {
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
        let state = &mut State::init(board.clone(), White, 4);

        let result = solve(state, 4);
        let expected = Some("I10,I6,I11,I8,J11,J8,G8".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3);
        assert_eq!(result, None);

        Ok(())
    }

    fn solve(state: &mut State, max_depth: u8) -> Option<String> {
        let searcher = Searcher::init();
        let may_table = searcher.search(state, max_depth);
        may_table
            .and_then(|table| {
                let mut resolver = Resolver::init(table);
                resolver.resolve(state)
            })
            .map(|m| Points(m.path).to_string())
    }
}
