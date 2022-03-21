/*
Df-pn algorithm is proposed in the following paper:

Nagai, Ayumu, and Hiroshi Imai.
"Proof for the equivalence between some best-first algorithms and depth-first algorithms for AND/OR trees."
IEICE TRANSACTIONS on Information and Systems 85.10 (2002): 1645-1653.
*/
use super::state::State;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::vcf;
use std::collections::HashMap;
use std::fmt;

// MEMO: Debug printing example is 6e2bace

pub struct Solver {
    searcher: Searcher,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            searcher: Searcher::init(),
        }
    }

    pub fn solve(&mut self, state: &mut State, max_depth: u8) -> Option<Mate> {
        let node = self.searcher.search(state, max_depth);
        if node.pn != 0 {
            return None;
        }
        self.solve_limit(state, max_depth)
    }

    fn solve_limit(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        self.solve_attacks(state, limit)
    }

    pub fn solve_attacks(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        if let Some(lose_or_move) = state.check_win_or_mandatory_move() {
            let m = lose_or_move.err().unwrap();
            return self.solve_attack(state, limit, m);
        }

        let maybe_opponent_threat = self.solve_vcf(state, state.last(), u8::MAX);
        let attacks = state.sorted_attacks(maybe_opponent_threat);
        let mut game = state.game().clone();
        for attack in attacks {
            let node = self.searcher.lookup_child(&mut game, attack, limit);
            if node.pn == 0 {
                return self.solve_attack(state, limit, attack);
            }
        }
        for max_depth in 0..=limit {
            let result = self.solve_vcf(state, state.turn(), max_depth);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn solve_attack(&mut self, state: &mut State, limit: u8, attack: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(attack);
        let result = self.solve_defences(state, limit).map(|m| m.unshift(attack));
        state.undo(last2_move);
        result
    }

    fn solve_defences(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        if let Some(win_or_move) = state.check_win_or_mandatory_move() {
            return match win_or_move {
                Ok(w) => Some(Mate::new(w, vec![])),
                Err(m) => self.solve_defence(state, limit, m),
            };
        }

        let maybe_threat = self.solve_vcf(state, state.last(), limit - 1);

        let defences = state.sorted_defences(maybe_threat.unwrap());
        let mut game = state.game().clone();
        let mut min_limit = u8::MAX;
        let mut selected_defence = Point(0, 0);
        for defence in defences {
            let node = self.searcher.lookup_child(&mut game, defence, limit);
            if node.pn == 0 && node.limit < min_limit {
                min_limit = node.limit;
                selected_defence = defence;
            }
        }
        self.solve_defence(state, limit, selected_defence)
    }

    fn solve_defence(&mut self, state: &mut State, limit: u8, defence: Point) -> Option<Mate> {
        let last2_move = state.game().last2_move();
        state.play(defence);
        let limit = limit - 1;
        let result = self.solve_limit(state, limit).map(|m| m.unshift(defence));
        state.undo(last2_move);
        result
    }

    fn solve_vcf(&mut self, state: &mut State, turn: Player, limit: u8) -> Option<Mate> {
        self.searcher.solve_vcf(state, turn, limit)
    }
}

struct Searcher {
    table: HashMap<u64, Node>,
    attacker_vcf_solver: vcf::dfs::Solver,
    defender_vcf_solver: vcf::dfs::Solver,
}

impl Searcher {
    pub fn init() -> Self {
        Self {
            table: HashMap::new(),
            attacker_vcf_solver: vcf::dfs::Solver::init(),
            defender_vcf_solver: vcf::dfs::Solver::init(),
        }
    }

    pub fn search(&mut self, state: &mut State, max_depth: u8) -> Node {
        self.search_limit(state, Node::root(max_depth), max_depth)
    }

    fn search_limit(&mut self, state: &mut State, threshold: Node, limit: u8) -> Node {
        if limit == 0 {
            return Node::inf_pn(limit);
        }
        self.search_attacks(state, threshold, limit)
    }

    fn search_attacks(&mut self, state: &mut State, threshold: Node, limit: u8) -> Node {
        if let Some(lose_or_move) = state.check_win_or_mandatory_move() {
            return match lose_or_move {
                Ok(_) => Node::inf_pn(limit),
                Err(m) => self.expand_attack(state, m, threshold, limit),
            };
        }

        if self.solve_vcf(state, state.turn(), limit).is_some() {
            return Node::inf_dn(limit);
        }

        let maybe_opponent_threat = self.solve_vcf(state, state.last(), u8::MAX);

        let attacks = state.sorted_attacks(maybe_opponent_threat);

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
        let hash = state.game().get_hash(limit);
        let current = self.lookup(hash, limit);
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
        if let Some(win_or_move) = state.check_win_or_mandatory_move() {
            return match win_or_move {
                Ok(_) => Node::inf_dn(limit),
                Err(m) => self.expand_defence(state, m, threshold, limit),
            };
        }

        let maybe_threat = self.solve_vcf(state, state.last(), limit - 1);
        if maybe_threat.is_none() {
            return Node::inf_pn(limit);
        }

        if self.solve_vcf(state, state.turn(), u8::MAX).is_some() {
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
        let hash = state.game().get_hash(limit);
        let limit = limit - 1;
        let current = self.lookup(hash, limit);
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
            let child = self.lookup_child(&mut game, attack, limit);
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
            let child = self.lookup_child(&mut game, defence, limit);
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

    pub fn solve_vcf(&mut self, state: &mut State, turn: Player, limit: u8) -> Option<Mate> {
        let attacker = state.attacker();
        let game = state.game();
        let state = &mut vcf::State::new(if turn == game.last() {
            game.pass()
        } else {
            game.clone()
        });
        for max_depth in [1, limit] {
            if max_depth > limit {
                return None;
            }
            let result = if turn == attacker {
                self.attacker_vcf_solver.solve(state, max_depth)
            } else {
                self.defender_vcf_solver.solve(state, max_depth)
            };
            if result.is_some() {
                return result;
            }
        }
        None
    }

    pub fn lookup_child(&self, game: &mut Game, m: Point, limit: u8) -> Node {
        let last2_move = game.last2_move();
        game.play(m);
        let result = self.lookup(game.get_hash(limit), limit);
        game.undo(last2_move);
        result
    }

    fn lookup(&self, hash: u64, limit: u8) -> Node {
        self.table.get(&hash).map_or(Node::init(limit), |c| *c)
    }
}

const INF: usize = usize::MAX;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
struct Node {
    pn: usize,
    dn: usize,
    limit: u8,
}

impl Node {
    pub fn new(pn: usize, dn: usize, limit: u8) -> Self {
        Self {
            pn: pn,
            dn: dn,
            limit: limit,
        }
    }

    pub fn init(limit: u8) -> Self {
        Self::new(1, 1, limit)
    }

    pub fn root(limit: u8) -> Self {
        Self::new(INF - 1, INF - 1, limit)
    }

    pub fn inf_pn(limit: u8) -> Self {
        Self::new(INF, 0, limit)
    }

    pub fn inf_dn(limit: u8) -> Self {
        Self::new(0, INF, limit)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pn = if self.pn == INF {
            "INF".to_string()
        } else {
            self.pn.to_string()
        };
        let dn = if self.dn == INF {
            "INF".to_string()
        } else {
            self.dn.to_string()
        };
        write!(f, "(pn: {}, dn: {})", pn, dn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::*;

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
        let state = &mut State::init(board.clone(), Black);
        let mut solver = Solver::init();

        let result = solver.solve(state, 4);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("F10,G9,I10,G10,H11,H12,G12".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 3);
        assert_eq!(result, None);

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
        let state = &mut State::init(board.clone(), White);
        let mut solver = Solver::init();

        let result = solver.solve(state, 4);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("I10,I6,I11,I8,J11,J8,G8".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 3);
        let result = result.map(|m| Points(m.path).to_string());
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
        let state = &mut State::init(board.clone(), White);
        let mut solver = Solver::init();

        let result = solver.solve(state, 4);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("F7,E8,G8,E6,G5,G7,H6".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 3);
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
        let state = &mut State::init(board.clone(), Black);
        let mut solver = Solver::init();

        let result = solver.solve(state, 4);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("J8,I7,I8,G8,L8,K8,K7".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 3);
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
        let state = &mut State::init(board.clone(), Black);
        let mut solver = Solver::init();

        let result = solver.solve(state, 7);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("G12,E10,F12,I12,H14,H13,F14,G13,F13,F11,E14,D15,G14".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 6);
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
        let state = &mut State::init(board.clone(), White);
        let mut solver = Solver::init();

        let result = solver.solve(state, 5);
        let result = result.map(|m| Points(m.path).to_string());
        let expected = Some("J4,G7,I4,I3,E6,G4,G6".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 4);
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
        let state = &mut State::init(board.clone(), Black);
        let mut searcher = Searcher::init();

        let result = searcher.search(state, 4);
        let expected = Node::new(0, INF, 0);
        assert_eq!(result, expected);

        let result = searcher.search(state, 3);
        let expected = Node::new(INF, 0, 0);
        assert_eq!(result, expected);

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
        let state = &mut State::init(board.clone(), White);
        let mut searcher = Searcher::init();

        let result = searcher.search(state, 4);
        let expected = Node::new(0, INF, 0);
        assert_eq!(result, expected);

        let result = searcher.search(state, 3);
        let expected = Node::new(INF, 0, 0);
        assert_eq!(result, expected);

        Ok(())
    }
}
