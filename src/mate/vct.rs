use super::super::board::Player::*;
use super::super::board::RowKind::*;
use super::super::board::*;
use super::move_searcher::*;
use super::state::*;
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

    // TODO: solve VCF first

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
        if self.game_state.next_player() != self.attacker {
            panic!()
        }
        VCTAttacks::new(&self.game_state)
    }

    fn defences(&self) -> impl Iterator<Item = Point> {
        if self.game_state.last_player() != self.attacker {
            panic!()
        }
        VCTDefences::new(&self.game_state)
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
    searcher: MoveSearcher,
}

impl VCTAttacks {
    fn new(state: &GameState) -> Self {
        Self {
            searcher: MoveSearcher::new(state),
        }
    }
}

impl Iterator for VCTAttacks {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.searcher.last_four_found() {
            return self
                .searcher
                .pop(MoveKind::LastFourCloser)
                .filter(|&p| self.searcher.get_threat(p).is_some());
        }
        if let Some(e) = self.searcher.pop(MoveKind::NextFourMove) {
            return Some(e);
        }
        if let Some(e) = self.searcher.pop(MoveKind::NextThreeMove) {
            return Some(e);
        }
        // TODO: threat_moves
        None
    }
}

struct VCTDefences {
    searcher: MoveSearcher,
}

impl VCTDefences {
    fn new(state: &GameState) -> Self {
        Self {
            searcher: MoveSearcher::new(state),
        }
    }
}

impl Iterator for VCTDefences {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if self.searcher.last_four_found() {
            return self.searcher.pop(MoveKind::LastFourCloser);
        }
        if let Some(e) = self.searcher.pop(MoveKind::LastThreeCloser) {
            return Some(e);
        }
        if let Some(e) = self.searcher.pop(MoveKind::NextFourMove) {
            return Some(e);
        }
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
        let result = solve_vct(&board, Black, 4, 1).map(|ps| Points(ps).to_string());
        let solution = "F10,G9,I10,G10,H11,H12,G12".to_string();
        assert_eq!(result, Some(solution));

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
        let result = solve_vct(&board, White, 3, 1).map(|ps| Points(ps).to_string());
        let solution = "I10,I8,J11,K12,F7".to_string();
        assert_eq!(result, Some(solution));

        Ok(())
    }
}
