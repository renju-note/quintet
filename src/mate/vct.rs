use super::super::board::Player::*;
use super::super::board::RowKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf::*;
use std::collections::HashSet;

pub fn solve_vct(board: &Board, player: Player, depth: u8, vcf_depth: u8) -> Option<Vec<Point>> {
    let opponent = player.opponent();

    // Already exists five
    if board.rows(player, Five).len() >= 1 || board.rows(opponent, Five).len() >= 1 {
        return None;
    }

    // Already exists overline
    if board.rows(Black, Overline).len() >= 1 {
        return None;
    }

    // Already exists four
    if board.rows(player, Four).len() >= 1 {
        return Some(vec![]);
    }

    let state = VCTState::new(board, player, vcf_depth);
    let mut searched = HashSet::new();
    solve(&state, depth, &mut searched)
}

fn solve(state: &VCTState, depth: u8, searched: &mut HashSet<u64>) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // check if already searched (and was dead-end)
    let hash = state.board_hash();
    if searched.contains(&hash) {
        return None;
    }
    searched.insert(hash);

    if let Some(result) = solve_vcf(&state.board(), state.attacker, state.vcf_depth, false) {
        return Some(result);
    }

    for attack in state.valid_moves() {
        let state = state.play(attack);
        let defences = state.valid_moves();
        if defences.is_empty() {
            return Some(vec![attack]);
        }

        let mut solution: Option<Vec<Point>> = None;
        for defence in defences {
            let state = state.play(defence);
            if let Some(mut ps) = solve(&state, depth - 1, searched) {
                let mut new_result = vec![attack, defence];
                new_result.append(&mut ps);
                solution = if solution.is_none() {
                    Some(new_result)
                } else {
                    let result = solution.unwrap();
                    if new_result.len() > result.len() {
                        Some(new_result)
                    } else {
                        Some(result)
                    }
                }
            } else {
                solution = None;
                break;
            }
        }
        if let Some(result) = solution {
            return Some(result);
        }
    }
    None
}

#[derive(Clone)]
struct VCTState {
    game_state: GameState,
    attacker: Player,
    vcf_depth: u8,
}

impl VCTState {
    pub fn new(board: &Board, player: Player, vcf_depth: u8) -> Self {
        let last_move = Self::choose_last_move(board, player);
        let game_state = GameState::from_board(board, player, last_move);
        VCTState {
            game_state: game_state.clone(),
            attacker: game_state.next_player(),
            vcf_depth: vcf_depth,
        }
    }

    pub fn board(&self) -> Board {
        self.game_state.board()
    }

    pub fn board_hash(&self) -> u64 {
        self.game_state.board_hash()
    }

    pub fn play_mut(&mut self, next_move: Point) {
        self.game_state.play_mut(next_move);
    }

    pub fn play(&self, next_move: Point) -> Self {
        let mut result = self.clone();
        result.play_mut(next_move);
        result
    }

    pub fn valid_moves(&self) -> Vec<Point> {
        if self.game_state.next_player() == self.attacker {
            self.attacks()
        } else {
            self.defences()
        }
    }

    fn attacks(&self) -> Vec<Point> {
        let last_four_eyes = self.game_state.row_eyes_along_last_move(Four);
        if last_four_eyes.len() >= 2 {
            return vec![];
        }
        let last_four_eye = last_four_eyes.into_iter().next();

        let mut two_eyes: HashSet<Point> = HashSet::new();
        two_eyes.extend(self.game_state.row_eyes(self.attacker, Two));
        let mut sword_eyes: HashSet<Point> = HashSet::new();
        sword_eyes.extend(self.game_state.row_eyes(self.attacker, Sword));
        self.game_state
            .legal_moves()
            .into_iter()
            .filter(|&p| {
                last_four_eye.map_or(true, |e| e == p)
                    && (two_eyes.contains(&p)
                        || sword_eyes.contains(&p)
                        || self.solve_attacker_vcf_after(p).is_some())
            })
            .collect()
    }

    fn defences(&self) -> Vec<Point> {
        let last_four_eyes = self.game_state.row_eyes_along_last_move(Four);
        if last_four_eyes.len() >= 2 {
            return vec![];
        }
        if last_four_eyes.len() == 1 {
            return last_four_eyes
                .into_iter()
                .filter(|&p| self.game_state.is_legal_move(p))
                .collect();
        }
        self.game_state
            .legal_moves()
            .into_iter()
            .filter(|&p| self.solve_attacker_vcf_after(p).is_none())
            .collect()
    }

    fn solve_attacker_vcf_after(&self, p: Point) -> Option<Vec<Point>> {
        let game_state = self.game_state.play(p);
        solve_vcf(&game_state.board(), self.attacker, self.vcf_depth, false)
    }

    fn choose_last_move(board: &Board, player: Player) -> Point {
        let opponent = player.opponent();
        let stones = board.stones(opponent);
        if let Some(four) = board.rows(opponent, Four).iter().next() {
            stones
                .into_iter()
                .find(|&s| s == four.start || s == four.end)
        } else {
            stones.into_iter().next()
        }
        .unwrap_or(Point(0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_black() -> Result<(), String> {
        // https://renjuportal.com/puzzle/3040/
        let board = "
            ---------------
            ---------------
            ---------------
            ---------------
            --------x------
            -------oo------
            -------oxo-----
            ------xo-x-----
            -------xo------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Board>()?;
        let result = solve_vct(&board, Black, 1, 2);
        let solution = "H11,H12,G12".parse::<Points>()?.into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vct(&board, Black, 0, 2);
        assert_eq!(result, None);

        let result = solve_vct(&board, Black, 1, 1);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_white() -> Result<(), String> {
        // https://renjuportal.com/puzzle/2990/
        let board = "
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ------o--o-----
            ------oxx------
            -------o-------
            --------x------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
            ---------------
        "
        .parse::<Board>()?;
        let result = solve_vct(&board, White, 3, 1);
        let solution = "I10,I6,I11,I8,F7".parse::<Points>()?.into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vct(&board, White, 2, 1);
        assert_eq!(result, None);

        Ok(())
    }
}
