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

    if state.game().won_by_last().is_some() {
        deadends.insert(board_hash);
        return None;
    }

    // TODO: check opponents' four

    if let Some(vcf) = state.solve_vcf(depth, vcf_deadends) {
        return Some(vcf);
    }

    // TODO: check opponents' threat

    let attacks = state.field().collect_sorted().into_iter().map(|t| t.0);
    for attack in attacks {
        let may_solution = solve_one(
            state,
            depth - 1,
            attack,
            deadends,
            vcf_deadends,
            op_deadends,
            op_vcf_deadends,
        );
        if may_solution.is_some() {
            return may_solution;
        }
    }

    deadends.insert(board_hash);
    None
}

fn solve_one(
    state: &mut State,
    depth: u8,
    attack: Point,
    deadends: &mut HashSet<u64>,
    vcf_deadends: &mut HashSet<u64>,
    op_deadends: &mut HashSet<u64>,
    op_vcf_deadends: &mut HashSet<u64>,
) -> Option<Solution> {
    let last2_move_attack = state.game().last2_move();
    if state.game().is_forbidden_move(attack) {
        return None;
    }

    state.play_mut(attack);

    if let Some(win) = state.game().won_by_last() {
        state.undo_mut(last2_move_attack);
        return Some(Solution::new(win, vec![attack]));
    }

    // TODO: check four

    if state.solve_vcf(u8::MAX, op_vcf_deadends).is_some() {
        state.undo_mut(last2_move_attack);
        return None;
    }

    let may_threat = state.solve_threat(depth, vcf_deadends);
    if may_threat.is_none() {
        state.undo_mut(last2_move_attack);
        return None;
    }
    let threat = may_threat.unwrap();

    let mut may_solution = Some(Solution::new(Win::Unknown(), vec![attack]));
    let defences = state.threat_defences(threat);
    for defence in defences {
        if state.game().is_forbidden_move(defence) {
            continue;
        }

        let last2_move_defence = state.game().last2_move();
        state.play_mut(defence);

        let may_vct = solve(
            state,
            depth,
            deadends,
            vcf_deadends,
            op_deadends,
            op_vcf_deadends,
        );
        if may_vct.is_none() {
            may_solution = None;
            state.undo_mut(last2_move_defence);
            break;
        }

        let solution = may_solution.unwrap();
        let mut rest_vct = may_vct.unwrap();
        if rest_vct.path.len() + 2 > solution.path.len() {
            let mut new_path = vec![attack, defence];
            new_path.append(&mut rest_vct.path);
            may_solution = Some(Solution::new(rest_vct.win, new_path));
        } else {
            may_solution = Some(solution)
        }

        state.undo_mut(last2_move_defence);
    }

    state.undo_mut(last2_move_attack);
    may_solution
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
        // TODO: last five moves, four, counter, forbidden breaker
        threat.path
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
        let expected = Some("I10,I8,J11,F7,K12".to_string());
        assert_eq!(result, expected);

        let result = solve(state, 3, &mut emp(), &mut emp(), &mut emp(), &mut emp());
        assert_eq!(result, None);

        Ok(())
    }

    fn emp() -> HashSet<u64> {
        HashSet::new()
    }
}
