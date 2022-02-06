use super::super::board::StructureKind::*;
use super::super::board::*;
use super::field::*;
use super::game::*;
use super::vcf;
use std::collections::{HashMap, HashSet};

pub struct Solver {
    deadends: HashSet<u64>,
    vcf_solver: vcf::Solver,
    op_vcf_solver: vcf::Solver,
}

impl Solver {
    pub fn init() -> Self {
        Self {
            deadends: HashSet::new(),
            vcf_solver: vcf::Solver::init(),
            op_vcf_solver: vcf::Solver::init(),
        }
    }

    pub fn solve(&mut self, state: &mut State, max_depth: u8) -> Option<Mate> {
        for (d, b) in Self::depth_breadth_priorities(max_depth) {
            let result = self.solve_depth_breadth(state, d, b);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn solve_depth_breadth(&mut self, state: &mut State, d: u8, b: u8) -> Option<Mate> {
        if d == 0 {
            return None;
        }

        let hash = state.game().get_hash(d, b);
        if self.deadends.contains(&hash) {
            return None;
        }

        let result = self.solve_attacks(state, d, b);

        if result.is_none() {
            self.deadends.insert(hash);
        }
        result
    }

    pub fn solve_attacks(&mut self, state: &mut State, d: u8, b: u8) -> Option<Mate> {
        if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
            return match last_win_or_abs {
                Ok(_) => None,
                Err(abs) => self.solve_attack(state, d, b, abs),
            };
        }

        if let Some(vcf) = self.solve_vcf(state, state.turn(), d) {
            return Some(vcf);
        }

        let mut attacks = state.potential_points();
        if let Some(op_threat) = self.solve_vcf(state, state.last(), u8::MAX) {
            let op_threat_defences = state.threat_defences(&op_threat);
            let op_threat_defences = op_threat_defences.into_iter().collect::<HashSet<_>>();
            attacks.retain(|(p, _)| op_threat_defences.contains(p));
        }
        attacks.sort_by(|a, b| b.1.cmp(&a.1));
        attacks.truncate(b as usize);

        for (attack, _) in attacks {
            let result = self.solve_attack(state, d, b, attack);
            if result.is_some() {
                return result;
            }
        }

        None
    }

    fn solve_attack(&mut self, state: &mut State, d: u8, b: u8, attack: Point) -> Option<Mate> {
        if state.game().is_forbidden_move(attack) {
            return None;
        }

        let last2_move = state.game().last2_move();
        state.play_mut(attack);

        let result = self.solve_defences(state, d, b).map(|m| m.unshift(attack));

        state.undo_mut(last2_move);
        return result;
    }

    fn solve_defences(&mut self, state: &mut State, d: u8, b: u8) -> Option<Mate> {
        if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
            return match last_win_or_abs {
                Ok(win) => Some(Mate::new(win, vec![])),
                Err(abs) => self.solve_defence(state, d, b, abs),
            };
        }

        if self.solve_vcf(state, state.turn(), u8::MAX).is_some() {
            return None;
        }

        let may_threat = self.solve_vcf(state, state.last(), d - 1);
        if may_threat.is_none() {
            return None;
        }

        let mut defences = state.threat_defences(&may_threat.unwrap());
        let mut potentials: HashMap<Point, u8> = HashMap::new();
        for (p, o) in state.potential_points() {
            potentials.insert(p, o);
        }
        defences.sort_by(|a, b| {
            let oa = potentials.get(a).unwrap_or(&0);
            let ob = potentials.get(b).unwrap_or(&0);
            ob.cmp(oa)
        });

        let mut result = Some(Mate::new(Win::Unknown(), vec![]));
        for defence in defences {
            let new_result = self.solve_defence(state, d, b, defence);
            if new_result.is_none() {
                result = None;
                break;
            }
            let new_mate = new_result.unwrap();
            result = result.map(|mate| {
                if mate.win == Win::Unknown() || new_mate.path.len() > mate.path.len() {
                    new_mate
                } else {
                    mate
                }
            });
        }

        result
    }

    fn solve_defence(&mut self, state: &mut State, d: u8, b: u8, defence: Point) -> Option<Mate> {
        if state.game().is_forbidden_move(defence) {
            return Some(Mate::new(Win::Forbidden(defence), vec![]));
        }

        let last2_move = state.game().last2_move();
        state.play_mut(defence);

        let result = self
            .solve_depth_breadth(state, d - 1, b)
            .map(|m| m.unshift(defence));

        state.undo_mut(last2_move);
        result
    }

    fn solve_vcf(&mut self, state: &mut State, turn: Player, max_depth: u8) -> Option<Mate> {
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
            self.op_vcf_solver.solve(state, max_depth)
        }
    }

    fn depth_breadth_priorities(max_depth: u8) -> Vec<(u8, u8)> {
        let mut dws = (1..=max_depth)
            .flat_map(|d| {
                [1 as u8, 2, 3, 4, 5].map(|lb| {
                    let b = (2 as u32).pow(lb as u32) as u8;
                    d.checked_add(lb).map(|n| (d, b, n))
                })
            })
            .flatten()
            .collect::<Vec<_>>();
        dws.sort_by(|a, b| a.2.cmp(&b.2));
        dws.into_iter().map(|(d, b, _)| (d, b)).collect()
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

    pub fn play_mut(&mut self, next_move: Point) {
        self.game.play_mut(next_move);
        self.field.update_along(next_move, self.game.board());
    }

    pub fn undo_mut(&mut self, last2_move: Point) {
        let last_move = self.game.last_move();
        self.game.undo_mut(last2_move);
        self.field.update_along(last_move, self.game.board());
    }

    pub fn potential_points(&self) -> Vec<(Point, u8)> {
        let min = if self.attacker == Player::Black { 4 } else { 3 };
        self.field.collect(min)
    }

    pub fn threat_defences(&mut self, threat: &Mate) -> Vec<Point> {
        let mut result = self.direct_defences(threat);
        result.extend(self.counter_defences(threat));
        result.extend(self.sword_eyes());
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
            game.play_mut(p);
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

    fn sword_eyes(&self) -> Vec<Point> {
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

    #[ignore]
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
        let expected = Some("G12,E10,F12,E12,H14,H13,F14,G13,F13,F11,E14,D15,G14".to_string());
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
        let expected = Some("J4,K3,I4,I3,F8,G7,E6,G9,G6".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 4);
        assert_eq!(result, None);

        Ok(())
    }
}
