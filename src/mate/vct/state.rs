use crate::analysis::field::PotentialField;
use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::state::{Key, State};
use crate::mate::vcf::VCFState;

pub struct VCTState {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
    field: PotentialField,
}

impl VCTState {
    pub fn new(game: Game, limit: u8, field: PotentialField) -> Self {
        Self {
            attacker: game.turn,
            game,
            limit,
            field,
        }
    }

    pub fn init(board: &Board, attacker: Player, limit: u8) -> Self {
        let game = Game::init(board, attacker);
        let field = PotentialField::init(attacker, 2, board);
        Self::new(game, limit, field)
    }

    pub fn vcf_state(&self, max_limit: u8) -> VCFState {
        let game = self.game.clone();
        let limit = self.limit.min(max_limit);
        VCFState::new(game, limit)
    }

    pub fn threat_state(&self, max_limit: u8) -> VCFState {
        let mut game = self.game.clone();
        game.play(None);
        let limit = if self.attacking() {
            self.limit - 1
        } else {
            self.limit
        }
        .min(max_limit);
        VCFState::new(game, limit)
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.game().is_forbidden_move(p)
    }

    pub fn check_event(&self) -> Option<Event> {
        self.game().check_event()
    }

    /// The key the child after `next_move` would have, without building it.
    /// Must stay in step with [`State::key`].
    pub fn next_key(&mut self, next_move: Option<Point>) -> Key {
        // Update only game in order not to cause updating state.field (which costs high)
        let limit = self.limit;
        let next_limit = if !self.attacking() { limit - 1 } else { limit };
        let attacker = self.attacker;
        // `into_play` flips the turn, so the hash is the child's.
        let position = self.game.into_play(next_move, |g| g.position_hash());
        Key::new(apply_attacker(position, attacker), next_limit)
    }

    pub fn sorted_potentials(&self, min: u8, only: Option<Vec<Point>>) -> Vec<(Point, u8)> {
        let mut result = if let Some(only) = only {
            let potentials = only.into_iter().map(|p| (p, self.field.get(p)));
            potentials.filter(|&(_, o)| o >= min).collect()
        } else {
            self.field.collect(min)
        };
        result.sort_by_key(|&a| std::cmp::Reverse(a.1));
        result.dedup();
        result
    }

    pub fn sort_by_potential(&self, points: Vec<Point>) -> Vec<(Point, u8)> {
        let mut result: Vec<_> = points.into_iter().map(|p| (p, self.field.get(p))).collect();
        result.sort_by_key(|&a| std::cmp::Reverse(a.1));
        result.dedup();
        result
    }

    pub fn empties(&self) -> Vec<Point> {
        self.game().board().empties().collect()
    }

    pub fn threat_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = threat.path().clone();
        result.extend(self.end_breakers(threat.end().clone()));
        result.extend(self.counter_defences(threat));
        result.extend(self.four_moves());
        result
    }

    fn end_breakers(&self, end: End) -> Vec<Point> {
        match end {
            Fours(p1, p2) => {
                vec![p1, p2]
            }
            Forbidden(p) => {
                let mut ds = vec![p];
                ds.extend(self.game().board().neighbors(p, 5, true));
                ds
            }
            _ => vec![],
        }
    }

    fn counter_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut game = self.game().clone();
        game.play(None);
        let threater = game.turn;
        let mut result = vec![];
        for &p in &threat.path {
            let turn = game.turn;
            game.play(Some(p));
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
        self.game()
            .board()
            .structures(self.game().turn, Sword)
            .flat_map(|s| s.eyes())
            .collect()
    }
}

impl State for VCTState {
    fn game(&self) -> &Game {
        &self.game
    }

    fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }

    fn attacker(&self) -> Player {
        self.attacker
    }

    fn limit(&self) -> u8 {
        self.limit
    }

    fn set_limit(&mut self, limit: u8) {
        self.limit = limit
    }

    fn after_play(&mut self, next_move: Option<Point>) {
        if let Some(next_move) = next_move {
            self.field.update_along(next_move, self.game.board());
        }
    }

    fn after_undo(&mut self, maybe_last_move: Option<Point>) {
        if let Some(last_move) = maybe_last_move {
            self.field.update_along(last_move, self.game.board());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::{Black, White};

    fn board() -> Board {
        "H8,I9,J9,H7".parse::<Board>().unwrap()
    }

    /// `next_key` peeks at the child's key without building the child, so
    /// it has to agree with what the child itself would say.
    #[test]
    fn test_next_key_matches_the_child() {
        let moves: Vec<Point> = ["G7", "K9", "H9"]
            .iter()
            .map(|s| s.parse().unwrap())
            .collect();
        for attacker in [Black, White] {
            let mut state = VCTState::init(&board(), attacker, 4);
            // Two plies, so that the turn and the limit have both moved.
            for &first in &moves {
                let predicted = state.next_key(Some(first));
                let actual = state.into_play(Some(first), |c| c.key());
                assert_eq!(predicted, actual, "{attacker:?} {first}");

                state.play(Some(first));
                for &second in &moves {
                    if second == first {
                        continue;
                    }
                    let predicted = state.next_key(Some(second));
                    let actual = state.into_play(Some(second), |c| c.key());
                    assert_eq!(predicted, actual, "{attacker:?} {first},{second}");
                }
                state.undo();
            }
        }
    }

    /// The same position is a win for one attacker and not the other, so the
    /// two must not share a memo entry.
    #[test]
    fn test_zobrist_hash_separates_the_attacker() {
        let board = board();
        let as_black = VCTState::init(&board, Black, 4).zobrist_hash();
        let as_white = VCTState::init(&board, White, 4).zobrist_hash();
        assert_ne!(as_black, as_white);

        // Those two also differ in whose turn it is, so pin the attacker on
        // its own: build one state that reaches the same board, turn and
        // limit while attacking as the other colour.
        let next: Point = "G7".parse().unwrap();
        let mut attacking_black = VCTState::init(&board, Black, 4);
        attacking_black.play(Some(next));
        let attacking_white = VCTState::init(&board.put(Black, next), White, 4);
        assert_eq!(attacking_black.game().turn, attacking_white.game().turn);
        assert_eq!(attacking_black.limit, attacking_white.limit);
        assert_eq!(
            attacking_black.game().board().zobrist_hash(),
            attacking_white.game().board().zobrist_hash()
        );
        assert_ne!(
            attacking_black.zobrist_hash(),
            attacking_white.zobrist_hash()
        );
    }
}
