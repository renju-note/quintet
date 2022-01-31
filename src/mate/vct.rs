use super::super::board::SequenceKind::*;
use super::super::board::*;
use super::field::*;
use super::game::*;
use super::vcf;
use std::collections::HashSet;

pub fn solve(
    state: &mut State,
    depth: u8,
    deadends: &mut HashSet<u64>,
    vcf_deadends: &mut HashSet<u64>,
    op_deadends: &mut HashSet<u64>,
    op_vcf_deadends: &mut HashSet<u64>,
) -> Option<Solution> {
    if depth == 0 {
        return None;
    }

    let board_hash = state.game().board().zobrist_hash();
    if deadends.contains(&board_hash) {
        return None;
    }

    if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
        match last_win_or_abs {
            Ok(_) => {
                deadends.insert(board_hash);
                return None;
            }
            Err(abs) => {
                return solve_attack(
                    state,
                    depth,
                    abs,
                    deadends,
                    vcf_deadends,
                    op_deadends,
                    op_vcf_deadends,
                )
            }
        }
    }

    if let Some(solution) = state.inspect_next_open_four() {
        return Some(solution);
    }

    if let Some(vcf) = state.solve_vcf(depth, vcf_deadends) {
        return Some(vcf);
    }

    let mut candidates = state.field().collect_nonzeros();
    if let Some(op_threat) = state.solve_threat(u8::MAX, op_vcf_deadends) {
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

    for attack in candidates.into_iter().map(|t| t.0) {
        let result = solve_attack(
            state,
            depth,
            attack,
            deadends,
            vcf_deadends,
            op_deadends,
            op_vcf_deadends,
        );
        if result.is_some() {
            return result;
        }
    }

    deadends.insert(board_hash);
    None
}

fn solve_attack(
    state: &mut State,
    depth: u8,
    attack: Point,
    deadends: &mut HashSet<u64>,
    vcf_deadends: &mut HashSet<u64>,
    op_deadends: &mut HashSet<u64>,
    op_vcf_deadends: &mut HashSet<u64>,
) -> Option<Solution> {
    if state.game().is_forbidden_move(attack) {
        return None;
    }

    let last2_move = state.game().last2_move();
    state.play_mut(attack);

    if let Some(last_win_or_abs) = state.game().inspect_last_win_or_abs() {
        match last_win_or_abs {
            Ok(win) => {
                state.undo_mut(last2_move);
                return Some(Solution::new(win, vec![attack]));
            }
            Err(abs) => {
                let result = solve_defence(
                    state,
                    depth,
                    abs,
                    deadends,
                    vcf_deadends,
                    op_deadends,
                    op_vcf_deadends,
                );
                state.undo_mut(last2_move);
                return result.map(|solution| solution.prepend(vec![attack, abs]));
            }
        }
    }

    if state.solve_vcf(u8::MAX, op_vcf_deadends).is_some() {
        state.undo_mut(last2_move);
        return None;
    }

    let may_threat = state.solve_threat(depth, vcf_deadends);
    if may_threat.is_none() {
        state.undo_mut(last2_move);
        return None;
    }
    let threat = may_threat.unwrap();

    let mut result = Some(Solution::new(Win::Unknown(), vec![attack]));
    let defences = state.threat_defences(threat);
    for defence in defences {
        let new_result = solve_defence(
            state,
            depth,
            defence,
            deadends,
            vcf_deadends,
            op_deadends,
            op_vcf_deadends,
        );
        if new_result.is_none() {
            result = None;
            break;
        }
        let solution = result.unwrap();
        let new_solution = new_result.unwrap();
        result = if new_solution.path.len() + 2 > solution.path.len() {
            Some(new_solution.prepend(vec![attack, defence]))
        } else {
            Some(solution)
        }
    }

    state.undo_mut(last2_move);
    result
}

fn solve_defence(
    state: &mut State,
    depth: u8,
    defence: Point,
    deadends: &mut HashSet<u64>,
    vcf_deadends: &mut HashSet<u64>,
    op_deadends: &mut HashSet<u64>,
    op_vcf_deadends: &mut HashSet<u64>,
) -> Option<Solution> {
    if state.game().is_forbidden_move(defence) {
        return Some(Solution::new(Win::Forbidden(defence), vec![]));
    }

    let last2_move = state.game().last2_move();
    state.play_mut(defence);

    let result = solve(
        state,
        depth - 1,
        deadends,
        vcf_deadends,
        op_deadends,
        op_vcf_deadends,
    );

    state.undo_mut(last2_move);
    result
}

pub struct State {
    attacker: Player,
    game: GameState,
    field: PotentialField,
}

impl State {
    pub fn new(attacker: Player, game: GameState, field: PotentialField) -> Self {
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
        let game = GameState::init(board, turn);
        Self::new(attacker, game, field)
    }

    pub fn attacker(&self) -> Player {
        self.attacker
    }

    pub fn game(&self) -> &'_ GameState {
        &self.game
    }

    pub fn field(&self) -> &'_ PotentialField {
        &self.field
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

    pub fn solve_vcf(&self, depth: u8, vcf_deadends: &mut HashSet<u64>) -> Option<Solution> {
        let state = &mut vcf::State::new(self.game.clone());
        vcf::solve(state, depth, vcf_deadends)
    }

    pub fn solve_threat(&self, depth: u8, vcf_deadends: &mut HashSet<u64>) -> Option<Solution> {
        let threat_state = self.game.pass();
        let state = &mut vcf::State::new(threat_state);
        vcf::solve(state, depth - 1, vcf_deadends)
    }

    pub fn threat_defences(&self, threat: Solution) -> Vec<Point> {
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
        let turn = self.game().turn();
        let sword_eyes = self
            .game()
            .board()
            .sequences(turn, Single, 4, turn.is_black())
            .flat_map(|(i, s)| i.mapped(s.eyes()).map(|i| i.to_point()));
        result.extend(sword_eyes);
        result
    }

    pub fn inspect_next_open_four(&self) -> Option<Solution> {
        let turn = self.game().turn();
        self.game()
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
            .filter(|&(p, _)| !self.game().is_forbidden_move(p))
            .map(|(e, (p1, p2))| Solution::new(Win::Fours(p1, p2), vec![e]))
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

        let result = solve(state, 4, &mut emp(), &mut emp(), &mut emp(), &mut emp());
        let result = result.map(|s| Points(s.path).to_string());
        let expected = Some("F10,G9,I10,G10,H11,H12,G12".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3, &mut emp(), &mut emp(), &mut emp(), &mut emp());
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

        let result = solve(state, 4, &mut emp(), &mut emp(), &mut emp(), &mut emp());
        let result = result.map(|s| Points(s.path).to_string());
        let expected = Some("I10,I6,I11,I8,J11,F7,K12".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3, &mut emp(), &mut emp(), &mut emp(), &mut emp());
        let result = result.map(|s| Points(s.path).to_string());
        assert_eq!(result, None);

        Ok(())
    }

    fn emp() -> HashSet<u64> {
        HashSet::new()
    }
}
