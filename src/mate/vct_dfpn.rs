use super::super::board::*;
use super::game::*;
use super::vcf;
use super::vct::State;
use std::collections::HashMap;
use std::fmt;

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

        if let Some(vcf) = self.solve_vcf(state, state.turn(), limit) {
            return Some(vcf);
        }

        let maybe_opponent_threat = self.solve_vcf(state, state.last(), u8::MAX);
        let attacks = state.sorted_attacks(maybe_opponent_threat);
        let mut game = state.game().clone();
        for attack in attacks {
            let node = self.searcher.lookup_move(&mut game, attack, limit);
            if node.pn == 0 {
                return self.solve_attack(state, limit, attack);
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
            let node = self.searcher.lookup_move(&mut game, defence, limit);
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

    fn solve_vcf(&mut self, state: &mut State, turn: Player, max_depth: u8) -> Option<Mate> {
        self.searcher.solve_vcf(state, turn, max_depth)
    }
}

struct Searcher {
    table: HashMap<u64, Node>,
    vcf_solver: vcf::Solver,
    opponent_vcf_solver: vcf::Solver,
}

impl Searcher {
    pub fn init() -> Self {
        Self {
            table: HashMap::new(),
            vcf_solver: vcf::Solver::init(),
            opponent_vcf_solver: vcf::Solver::init(),
        }
    }

    pub fn search(&mut self, state: &mut State, max_depth: u8) -> Node {
        self.search_limit(state, Node::root(max_depth), max_depth)
    }

    fn search_limit(&mut self, state: &mut State, bound: Node, limit: u8) -> Node {
        if limit == 0 {
            return Node::inf_pn(limit);
        }
        self.search_attacks(state, bound, limit)
    }

    fn search_attacks(&mut self, state: &mut State, bound: Node, limit: u8) -> Node {
        if let Some(lose_or_move) = state.check_win_or_mandatory_move() {
            return match lose_or_move {
                Ok(_) => Node::inf_pn(limit),
                Err(m) => self.loop_attacks(state, &[m], bound, limit),
            };
        }

        if let Some(vcf) = self.solve_vcf(state, state.turn(), limit) {
            return Node::inf_dn(limit - (vcf.path.len() as u8 + 1) / 2);
        }

        let maybe_opponent_threat = self.solve_vcf(state, state.last(), u8::MAX);

        let attacks = state.sorted_attacks(maybe_opponent_threat);
        self.loop_attacks(state, &attacks, bound, limit)
    }

    fn loop_attacks(
        &mut self,
        state: &mut State,
        attacks: &[Point],
        bound: Node,
        limit: u8,
    ) -> Node {
        loop {
            let (current, best) = self.select_attack(state, attacks, limit);
            if current.pn > bound.pn || current.dn > bound.dn {
                return current;
            }
            let (next, next_bound) = best.unwrap();
            self.expand_attack(state, next, next_bound, limit);
        }
    }

    fn expand_attack(&mut self, state: &mut State, attack: Point, bound: Node, limit: u8) -> Node {
        let last2_move = state.game().last2_move();
        state.play(attack);
        let hash = state.game().get_hash(limit);
        let result = self.search_defences(state, bound, limit);
        self.table.insert(hash, result.clone());
        state.undo(last2_move);
        result
    }

    fn search_defences(&mut self, state: &mut State, bound: Node, limit: u8) -> Node {
        if let Some(win_or_move) = state.check_win_or_mandatory_move() {
            return match win_or_move {
                Ok(_) => Node::inf_dn(limit),
                Err(m) => self.loop_defences(state, &[m], bound, limit),
            };
        }

        if self.solve_vcf(state, state.turn(), u8::MAX).is_some() {
            return Node::inf_pn(limit);
        }

        let maybe_threat = self.solve_vcf(state, state.last(), limit - 1);
        if maybe_threat.is_none() {
            return Node::inf_pn(limit);
        }

        let defences = state.sorted_defences(maybe_threat.unwrap());
        self.loop_defences(state, &defences, bound, limit)
    }

    fn loop_defences(
        &mut self,
        state: &mut State,
        defences: &[Point],
        bound: Node,
        limit: u8,
    ) -> Node {
        loop {
            let (current, best) = self.select_defence(state, defences, limit);
            if current.dn > bound.dn || current.pn > bound.pn {
                return current;
            }
            let (next, next_bound) = best.unwrap();
            self.expand_defence(state, next, next_bound, limit);
        }
    }

    fn expand_defence(
        &mut self,
        state: &mut State,
        defence: Point,
        bound: Node,
        limit: u8,
    ) -> Node {
        let last2_move = state.game().last2_move();
        state.play(defence);
        let hash = state.game().get_hash(limit);
        let result = self.search_limit(state, bound, limit - 1);
        self.table.insert(hash, result.clone());
        state.undo(last2_move);
        result
    }

    fn select_attack(
        &self,
        state: &State,
        attacks: &[Point],
        limit: u8,
    ) -> (Node, Option<(Point, Node)>) {
        let mut game = state.game().clone();
        let mut best: Option<(Point, Node)> = None;
        let mut pn = INF;
        let mut dn: usize = 0;
        let mut l = limit;
        for &attack in attacks {
            let node = self.lookup_move(&mut game, attack, limit);
            if node.pn < pn {
                best.replace((attack, node));
            }
            pn = pn.min(node.pn);
            dn = dn.checked_add(node.dn).unwrap_or(INF);
            l = l.min(node.limit);
            if pn == 0 {
                dn = INF;
                break;
            }
        }
        (Node::new(pn, dn, l), best)
    }

    fn select_defence(
        &self,
        state: &State,
        defences: &[Point],
        limit: u8,
    ) -> (Node, Option<(Point, Node)>) {
        let mut game = state.game().clone();
        let mut best: Option<(Point, Node)> = None;
        let mut pn: usize = 0;
        let mut dn = INF;
        let mut l = limit;
        for &defence in defences {
            let node = self.lookup_move(&mut game, defence, limit);
            if node.dn < dn {
                best.replace((defence, node));
            }
            pn = pn.checked_add(node.pn).unwrap_or(INF);
            dn = dn.min(node.dn);
            l = l.min(node.limit);
            if dn == 0 {
                pn = INF;
                break;
            }
        }
        (Node::new(pn, dn, l), best)
    }

    pub fn solve_vcf(&mut self, state: &mut State, turn: Player, max_depth: u8) -> Option<Mate> {
        let attacker = state.attacker();
        let game = state.game();
        let state = &mut vcf::State::new(if turn == game.last() {
            game.pass()
        } else {
            game.clone()
        });
        if turn == attacker {
            self.vcf_solver.solve(state, max_depth)
        } else {
            self.opponent_vcf_solver.solve(state, max_depth)
        }
    }

    pub fn lookup_move(&self, game: &mut Game, m: Point, limit: u8) -> Node {
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
    use super::super::super::board::Player::*;
    use super::*;

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