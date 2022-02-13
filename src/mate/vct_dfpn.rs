use super::super::board::StructureKind::*;
use super::super::board::*;
use super::field::*;
use super::game::*;
use super::vcf;
use std::collections::{HashMap, HashSet};
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
        if limit == 0 {
            return None;
        }
        self.solve_attacks(state, limit)
    }

    pub fn solve_attacks(&mut self, state: &mut State, limit: u8) -> Option<Mate> {
        if let Some(lose_or_move) = state.check_win_or_mandatory_move() {
            return match lose_or_move {
                Ok(_) => None,
                Err(m) => self.solve_attack(state, limit, m),
            };
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

        if self.solve_vcf(state, state.turn(), u8::MAX).is_some() {
            return None;
        }

        let maybe_threat = self.solve_vcf(state, state.last(), limit - 1);
        if maybe_threat.is_none() {
            return None;
        }

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
        if state.game().is_forbidden_move(defence) {
            return Some(Mate::new(Win::Forbidden(defence), vec![]));
        }

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

        if self.solve_vcf(state, state.turn(), limit).is_some() {
            return Node::inf_dn(limit);
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

pub struct State {
    attacker: Player,
    game: Game,
    field: PotentialField,
}

impl State {
    pub fn new(attacker: Player, game: Game, field: PotentialField) -> Self {
        Self {
            attacker: attacker,
            game: game,
            field: field,
        }
    }

    pub fn init(board: Board, turn: Player) -> Self {
        let field = PotentialField::init(turn, 2, &board);
        let game = Game::init(board, turn);
        Self::new(turn, game, field)
    }

    pub fn attacker(&self) -> Player {
        self.attacker
    }

    pub fn game(&self) -> &'_ Game {
        &self.game
    }

    pub fn turn(&self) -> Player {
        self.game.turn()
    }

    pub fn last(&self) -> Player {
        self.game.last()
    }

    pub fn play(&mut self, next_move: Point) {
        self.game.play(next_move);
        self.field.update_along(next_move, self.game.board());
    }

    pub fn undo(&mut self, last2_move: Point) {
        let last_move = self.game.last_move();
        self.game.undo(last2_move);
        self.field.update_along(last_move, self.game.board());
    }

    pub fn check_win_or_mandatory_move(&self) -> Option<Result<Win, Point>> {
        let (maybe_first, maybe_another) = self.game.check_last_four_eyes();
        if maybe_first.is_some() && maybe_another.is_some() {
            let win = Win::Fours(maybe_first.unwrap(), maybe_another.unwrap());
            Some(Ok(win))
        } else if maybe_first.map_or(false, |e| self.game.is_forbidden_move(e)) {
            let win = Win::Forbidden(maybe_first.unwrap());
            Some(Ok(win))
        } else if maybe_first.is_some() {
            let mandatory_move = maybe_first.unwrap();
            Some(Err(mandatory_move))
        } else {
            None
        }
    }

    pub fn sorted_attacks(&self, maybe_threat: Option<Mate>) -> Vec<Point> {
        let mut potentials = self.potentials();
        if let Some(threat) = maybe_threat {
            let threat_defences = self.threat_defences(&threat);
            let threat_defences = threat_defences.into_iter().collect::<HashSet<_>>();
            potentials.retain(|(p, _)| threat_defences.contains(p));
        }
        potentials.sort_by(|a, b| b.1.cmp(&a.1));
        potentials
            .into_iter()
            .map(|t| t.0)
            .filter(|&p| !self.game.is_forbidden_move(p))
            .collect()
    }

    pub fn sorted_defences(&self, threat: Mate) -> Vec<Point> {
        let mut result = self.threat_defences(&threat);
        let mut potential_map = HashMap::new();
        for (p, o) in self.potentials() {
            potential_map.insert(p, o);
        }
        result.sort_by(|a, b| {
            let oa = potential_map.get(a).unwrap_or(&0);
            let ob = potential_map.get(b).unwrap_or(&0);
            ob.cmp(oa)
        });
        result
            .into_iter()
            .filter(|&p| !self.game.is_forbidden_move(p))
            .collect()
    }

    fn potentials(&self) -> Vec<(Point, u8)> {
        let min = if self.attacker == Player::Black { 4 } else { 3 };
        self.field.collect(min)
    }

    fn threat_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = self.direct_defences(threat);
        result.extend(self.counter_defences(threat));
        result.extend(self.four_moves());
        result
    }

    fn direct_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = threat.path.clone();
        match threat.win {
            Win::Fours(p1, p2) => {
                result.extend([p1, p2]);
            }
            Win::Forbidden(p) => {
                result.push(p);
                result.extend(self.game.board().neighbors(p, 5, true));
            }
            _ => (),
        }
        result
    }

    fn counter_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut game = self.game.pass();
        let threater = game.turn();
        let mut result = vec![];
        for &p in &threat.path {
            let turn = game.turn();
            game.play(p);
            if turn == threater {
                continue;
            }
            let swords = game.board().structures_on(p, turn, Sword);
            for s in swords {
                result.extend(s.eyes());
            }
        }
        result
    }

    fn four_moves(&self) -> Vec<Point> {
        self.game
            .board()
            .structures(self.turn(), Sword)
            .flat_map(|s| s.eyes())
            .collect()
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
