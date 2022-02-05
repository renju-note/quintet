use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::field::*;
use super::game::*;
use super::vcf;
use std::collections::HashSet;

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
        for (d, w) in Self::depth_width_priorities(max_depth) {
            let result = self.solve_depth_width(state, d, w);
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn solve_depth_width(&mut self, state: &mut State, depth: u8, width: u8) -> Option<Mate> {
        if depth == 0 {
            return None;
        }

        let hash = state.game().get_hash(depth, width);
        if self.deadends.contains(&hash) {
            return None;
        }

        let result = self.solve_attacks(state, depth, width);

        if result.is_none() {
            self.deadends.insert(hash);
        }
        result
    }

    pub fn solve_attacks(&mut self, state: &mut State, depth: u8, width: u8) -> Option<Mate> {
        if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
            return match last_win_or_abs {
                Ok(_) => None,
                Err(abs) => self.solve_attack(state, depth, width, abs),
            };
        }

        if let Some(vcf) = self.solve_vcf(state, state.turn(), depth) {
            return Some(vcf);
        }

        let mut candidates = state.attack_candidates();
        if let Some(op_threat) = self.solve_vcf(state, state.last(), u8::MAX) {
            let op_threat_defences = state.threat_defences(&op_threat);
            let op_threat_defences = op_threat_defences.into_iter().collect::<HashSet<_>>();
            candidates.retain(|(p, _)| op_threat_defences.contains(p));
        }
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let attacks = candidates
            .into_iter()
            .take(width as usize)
            .map(|t| t.0)
            .collect::<Vec<_>>();

        for attack in attacks {
            let result = self.solve_attack(state, depth, width, attack);
            if result.is_some() {
                return result;
            }
        }

        None
    }

    fn solve_attack(
        &mut self,
        state: &mut State,
        depth: u8,
        width: u8,
        attack: Point,
    ) -> Option<Mate> {
        if state.game().is_forbidden_move(attack) {
            return None;
        }

        let last2_move = state.game().last2_move();
        state.play_mut(attack);

        let result = self
            .solve_defences(state, depth, width)
            .map(|m| m.unshift(attack));

        state.undo_mut(last2_move);
        return result;
    }

    fn solve_defences(&mut self, state: &mut State, depth: u8, width: u8) -> Option<Mate> {
        if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
            return match last_win_or_abs {
                Ok(win) => Some(Mate::new(win, vec![])),
                Err(abs) => self.solve_defence(state, depth, width, abs),
            };
        }

        if self.solve_vcf(state, state.turn(), u8::MAX).is_some() {
            return None;
        }

        let may_threat = self.solve_vcf(state, state.last(), depth - 1);
        if may_threat.is_none() {
            return None;
        }

        let defences = state.threat_defences(&may_threat.unwrap());
        let mut result = Some(Mate::new(Win::Unknown(), vec![]));
        for defence in defences {
            let new_result = self.solve_defence(state, depth, width, defence);
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

    fn solve_defence(
        &mut self,
        state: &mut State,
        depth: u8,
        width: u8,
        defence: Point,
    ) -> Option<Mate> {
        if state.game().is_forbidden_move(defence) {
            return Some(Mate::new(Win::Forbidden(defence), vec![]));
        }

        let last2_move = state.game().last2_move();
        state.play_mut(defence);

        let result = self
            .solve_depth_width(state, depth - 1, width)
            .map(|m| m.unshift(defence));

        state.undo_mut(last2_move);
        result
    }

    fn solve_vcf(&mut self, state: &mut State, turn: Player, depth: u8) -> Option<Mate> {
        let attacker = state.attacker();
        let game = state.game();
        let state = &mut vcf::State::new(if turn == game.last() {
            game.pass()
        } else {
            game.clone()
        });
        if turn == attacker {
            self.vcf_solver.solve(state, depth)
        } else {
            self.op_vcf_solver.solve(state, depth)
        }
    }

    fn depth_width_priorities(max_depth: u8) -> Vec<(u8, u8)> {
        let mut dws = (1..=max_depth)
            .flat_map(|d| {
                [1 as u8, 2, 3, 4, 5].map(|lw| {
                    let w = (2 as u32).pow(lw as u32) as u8;
                    d.checked_add(lw).map(|n| (d, w, n))
                })
            })
            .flatten()
            .collect::<Vec<_>>();
        dws.sort_by(|a, b| a.2.cmp(&b.2));
        dws.into_iter().map(|(d, w, _)| (d, w)).collect()
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

    pub fn attack_candidates(&self) -> Vec<(Point, u8)> {
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
            let swords = game
                .board()
                .sequences_on(p, turn, Single, 3, turn.is_black());
            for (i, s) in swords {
                let eyes = i.mapped(s.eyes()).map(|i| i.to_point());
                result.extend(eyes);
            }
        }
        result
    }

    fn sword_eyes(&self) -> Vec<Point> {
        let turn = self.turn();
        self.game
            .board()
            .sequences(turn, Single, 4, turn.is_black())
            .flat_map(|(i, s)| i.mapped(s.eyes()).map(|i| i.to_point()))
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
        let expected = Some("I10,I6,I11,I8,J11,F7,K12".to_string());
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
        let expected = Some("J4,K3,I4,I3,F8,G7,E6,G1,F6".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 4);
        assert_eq!(result, None);

        Ok(())
    }
}
