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

    for attack in state.attacks() {
        let state = state.play(attack);
        let defences = state.defences();
        let mut solution: Option<Vec<Point>> = Some(vec![attack]);
        for defence in defences {
            let state = state.play(defence);
            if let Some(mut ps) = solve(&state, depth - 1, searched) {
                let mut new = vec![attack, defence];
                new.append(&mut ps);
                solution = solution.map(|old| if new.len() > old.len() { new } else { old });
            } else {
                solution = None;
                break;
            }
        }
        if solution.is_some() {
            return solution;
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

    fn attacks(&self) -> impl Iterator<Item = Point> {
        VCTAttacks::new(&self)
    }

    fn defences(&self) -> impl Iterator<Item = Point> {
        VCTDefences::new(&self)
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

struct VCTAttacks {
    state: VCTState,
    last_four_inited: bool,
    last_four_eyes_count: usize,
    valid_last_four_closers: Vec<Point>,
    next_four_inited: bool,
    valid_next_four_moves: Vec<Point>,
    next_three_inited: bool,
    valid_next_three_moves: Vec<Point>,
    tried: HashSet<Point>,
}

impl VCTAttacks {
    fn new(state: &VCTState) -> Self {
        Self {
            state: state.clone(),
            last_four_inited: false,
            last_four_eyes_count: 0,
            valid_last_four_closers: vec![],
            next_four_inited: false,
            valid_next_four_moves: vec![],
            next_three_inited: false,
            valid_next_three_moves: vec![],
            tried: HashSet::new(),
        }
    }

    fn init_last_four(&mut self) {
        if !self.last_four_inited {
            let last_four_eyes = self.state.game_state.row_eyes_along_last_move(Four);
            self.last_four_eyes_count = last_four_eyes.len();
            self.valid_last_four_closers = last_four_eyes
                .into_iter()
                .filter(|&p| !self.state.game_state.is_forbidden_move(p))
                .collect();
            self.last_four_inited = true;
        }
    }

    fn init_next_four(&mut self) {
        if !self.next_four_inited {
            self.valid_next_four_moves = self
                .state
                .game_state
                .row_eyes(self.state.attacker, Sword)
                .into_iter()
                .filter(|&p| !self.state.game_state.is_forbidden_move(p))
                .collect();
            self.next_four_inited = true;
        }
    }

    fn init_next_three(&mut self) {
        if !self.next_three_inited {
            // TODO: remove fake three (= another eye is forbidden)
            self.valid_next_three_moves = self
                .state
                .game_state
                .row_eyes(self.state.attacker, Two)
                .into_iter()
                .filter(|&p| !self.state.game_state.is_forbidden_move(p))
                .collect();
            self.next_three_inited = true;
        }
    }

    fn solve_attacker_vcf_after(&self, p: Point) -> Option<Vec<Point>> {
        let game_state = self.state.game_state.play(p);
        solve_vcf(
            &game_state.board(),
            self.state.attacker,
            self.state.vcf_depth,
            false,
        )
    }
}

impl Iterator for VCTAttacks {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.init_last_four();
        if self.last_four_eyes_count >= 2 {
            return None;
        }
        if self.last_four_eyes_count == 1 {
            return self
                .valid_last_four_closers
                .pop()
                .filter(|&e| self.solve_attacker_vcf_after(e).is_some());
        }
        self.init_next_three();
        if let Some(e) = self.valid_next_three_moves.pop() {
            if !self.tried.contains(&e) {
                self.tried.insert(e);
                return Some(e);
            }
        }
        self.init_next_four();
        if let Some(e) = self.valid_next_four_moves.pop() {
            if !self.tried.contains(&e) {
                self.tried.insert(e);
                return Some(e);
            }
        }
        // TODO: threat_moves
        None
    }
}

struct VCTDefences {
    state: VCTState,
    last_four_inited: bool,
    last_four_eyes_count: usize,
    valid_last_four_closers: Vec<Point>,
    last_three_inited: bool,
    last_three_eyes_count: usize,
    valid_last_three_closers: Vec<Point>,
    next_four_inited: bool,
    valid_next_four_moves: Vec<Point>,
    tried: HashSet<Point>,
}

impl VCTDefences {
    fn new(state: &VCTState) -> Self {
        Self {
            state: state.clone(),
            last_four_inited: false,
            last_four_eyes_count: 0,
            valid_last_four_closers: vec![],
            last_three_inited: false,
            last_three_eyes_count: 0,
            valid_last_three_closers: vec![],
            next_four_inited: false,
            valid_next_four_moves: vec![],
            tried: HashSet::new(),
        }
    }

    fn init_last_four(&mut self) {
        if !self.last_four_inited {
            let last_four_eyes = self.state.game_state.row_eyes_along_last_move(Four);
            self.last_four_eyes_count = last_four_eyes.len();
            self.valid_last_four_closers = last_four_eyes
                .into_iter()
                .filter(|&p| !self.state.game_state.is_forbidden_move(p))
                .collect();
            self.last_four_inited = true;
        }
    }

    fn init_last_three(&mut self) {
        if !self.last_three_inited {
            // TODO: outer closer and summer closer
            let last_three_eyes = self
                .state
                .game_state
                .row_eyes(self.state.game_state.last_player(), Three);
            self.last_three_eyes_count = last_three_eyes.len();
            self.valid_last_three_closers = last_three_eyes
                .into_iter()
                .filter(|&p| !self.state.game_state.is_forbidden_move(p))
                .collect();
            self.last_three_inited = true;
        }
    }

    fn init_next_four(&mut self) {
        if !self.next_four_inited {
            self.valid_next_four_moves = self
                .state
                .game_state
                .row_eyes(self.state.game_state.next_player(), Sword)
                .into_iter()
                .filter(|&p| !self.state.game_state.is_forbidden_move(p))
                .collect();
            self.next_four_inited = true;
        }
    }
}

impl Iterator for VCTDefences {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.init_last_four();
        if self.last_four_eyes_count >= 2 {
            return None;
        }
        if self.last_four_eyes_count == 1 {
            return self.valid_last_four_closers.pop();
        }
        self.init_next_four();
        self.init_last_three();
        if self.last_three_eyes_count >= 1 {
            if let Some(e) = self
                .valid_last_three_closers
                .pop()
                .or_else(|| self.valid_next_four_moves.pop())
            {
                if !self.tried.contains(&e) {
                    self.tried.insert(e);
                    return Some(e);
                }
            }
        }
        // TODO: threat_moves
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_black() -> Result<(), String> {
        let board = "
            ---------------
            ---------------
            ---------------
            ---------------
            --------x------
            -------o-------
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
        let result = solve_vct(&board, Black, 4, 2);
        let solution = "F10,E11,I10".parse::<Points>()?.into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vct(&board, Black, 1, 2);
        assert_eq!(result, None);

        let result = solve_vct(&board, Black, 2, 1);
        assert_eq!(result, None);

        Ok(())
    }

    #[test]
    fn test_white() -> Result<(), String> {
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
        let solution = "I10,I6,I11,I8,J11".parse::<Points>()?.into_vec();
        assert_eq!(result, Some(solution));

        let result = solve_vct(&board, White, 2, 1);
        assert_eq!(result, None);

        Ok(())
    }
}
