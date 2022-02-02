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

    pub fn solve(&mut self, state: &mut State, depth: u8) -> Option<Mate> {
        if depth == 0 {
            return None;
        }

        let board_hash = state.game().board().zobrist_hash();
        if self.deadends.contains(&board_hash) {
            return None;
        }

        let result = self.solve_all(state, depth);

        if result.is_none() {
            self.deadends.insert(board_hash);
        }
        result
    }

    pub fn solve_all(&mut self, state: &mut State, depth: u8) -> Option<Mate> {
        if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
            return match last_win_or_abs {
                Ok(_) => None,
                Err(abs) => self.solve_attack(state, depth, abs),
            };
        }

        if let Some(mate) = state.inspect_next_open_four() {
            return Some(mate);
        }

        if let Some(vcf) = self.solve_vcf(state, state.turn(), depth) {
            return Some(vcf);
        }

        let mut candidates = state.field().collect_nonzeros();
        if let Some(op_threat) = self.solve_vcf(state, state.last(), u8::MAX) {
            let op_threat_defences = state
                .threat_defences(op_threat)
                .into_iter()
                .collect::<HashSet<Point>>();
            candidates = candidates
                .into_iter()
                .filter(|(p, _)| op_threat_defences.contains(p))
                .collect();
        }
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // TODO: IDDFS
        for attack in candidates.into_iter().map(|t| t.0) {
            let result = self.solve_attack(state, depth, attack);
            if result.is_some() {
                return result;
            }
        }

        None
    }

    fn solve_attack(&mut self, state: &mut State, depth: u8, attack: Point) -> Option<Mate> {
        if state.game().is_forbidden_move(attack) {
            return None;
        }

        let last2_move = state.game().last2_move();
        state.play_mut(attack);

        let result = self.solve_defences(state, depth).map(|s| s.prepend(attack));

        state.undo_mut(last2_move);
        return result;
    }

    fn solve_defences(&mut self, state: &mut State, depth: u8) -> Option<Mate> {
        if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
            return match last_win_or_abs {
                Ok(win) => Some(Mate::new(win, vec![])),
                Err(abs) => self.solve_defence(state, depth, abs),
            };
        }

        if self.solve_vcf(state, state.turn(), u8::MAX).is_some() {
            return None;
        }

        let may_threat = self.solve_vcf(state, state.last(), depth - 1);
        if may_threat.is_none() {
            return None;
        }

        let defences = state.threat_defences(may_threat.unwrap());
        let mut result = Some(Mate::new(Win::Unknown(), vec![]));
        for defence in defences {
            let new_result = self.solve_defence(state, depth, defence);
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

    fn solve_defence(&mut self, state: &mut State, depth: u8, defence: Point) -> Option<Mate> {
        if state.game().is_forbidden_move(defence) {
            return Some(Mate::new(Win::Forbidden(defence), vec![]));
        }

        let last2_move = state.game().last2_move();
        state.play_mut(defence);

        let result = self.solve(state, depth - 1).map(|s| s.prepend(defence));

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
        let attacker = turn;
        // TODO: opponent potential field if white
        let field = PotentialField::new(board.potentials(attacker, 3, attacker.is_black()));
        let game = Game::init(board, turn);
        Self::new(attacker, game, field)
    }

    pub fn attacker(&self) -> Player {
        self.attacker
    }

    pub fn game(&self) -> &'_ Game {
        &self.game
    }

    pub fn field(&self) -> &'_ PotentialField {
        &self.field
    }

    pub fn turn(&self) -> Player {
        self.game.turn()
    }

    pub fn last(&self) -> Player {
        self.game.last()
    }

    pub fn play_mut(&mut self, next_move: Point) {
        self.game.play_mut(next_move);
        self.update_potentials_along(next_move);
    }

    pub fn undo_mut(&mut self, last2_move: Point) {
        let last_move = self.game.last_move();
        self.game.undo_mut(last2_move);
        self.update_potentials_along(last_move);
    }

    pub fn threat_defences(&self, threat: Mate) -> Vec<Point> {
        // TODO: counter
        let mut result = threat.path.clone();
        match threat.win {
            Win::Fours(p1, p2) => {
                result.extend([p1, p2]);
            }
            Win::Forbidden(p) => {
                result.push(p);
            } // TODO: points on p
            _ => (),
        }
        let turn = self.turn();
        let sword_eyes = self
            .game
            .board()
            .sequences(turn, Single, 4, turn.is_black())
            .flat_map(|(i, s)| i.mapped(s.eyes()).map(|i| i.to_point()));
        result.extend(sword_eyes);
        result
    }

    pub fn inspect_next_open_four(&self) -> Option<Mate> {
        let turn = self.turn();
        self.game
            .board()
            .sequences(turn, Compact, 3, turn.is_black())
            .map(|(i, s)| {
                (
                    i.walk(s.eyes()[0]).to_point(),
                    (
                        i.walk_checked(-1).unwrap().to_point(),
                        i.walk_checked(4).unwrap().to_point(),
                    ),
                )
            })
            .filter(|&(p, _)| !self.game.is_forbidden_move(p))
            .map(|(e, (p1, p2))| Mate::new(Win::Fours(p1, p2), vec![e]))
            .next()
    }

    fn update_potentials_along(&mut self, p: Point) {
        let r = self.attacker();
        let potentials = self.game.board().potentials_along(p, r, 3, r.is_black());
        self.field.update_along(p, potentials);
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::board::Player::*;
    use super::*;

    #[test]
    fn test_black() -> Result<(), String> {
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
        let result = result.map(|s| Points(s.path).to_string());
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
        let result = result.map(|s| Points(s.path).to_string());
        let expected = Some("I10,I6,I11,I8,J11,F7,K12".to_string());
        assert_eq!(result, expected);

        let result = solver.solve(state, 3);
        let result = result.map(|s| Points(s.path).to_string());
        assert_eq!(result, None);

        Ok(())
    }
}
