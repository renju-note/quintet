use crate::board::RowKind::*;
use crate::board::*;
use crate::feature::potential::PotentialField;
use crate::feature::shape::{Shape, ShapeMap};
use crate::feature::sword::SwordMap;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::state::{Key, State};
use crate::mate::vcf::VCFState;

pub struct VCTState {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
    field: PotentialField,
    shapes: ShapeMap,
    /// The swords, kept only to hand to the nested VCF states.
    swords: SwordMap,
}

impl VCTState {
    pub fn new(game: Game, limit: u8, field: PotentialField) -> Self {
        let swords = SwordMap::init(game.board());
        let shapes = ShapeMap::init(game.board());
        Self {
            attacker: game.turn,
            game,
            limit,
            field,
            shapes,
            swords,
        }
    }

    pub fn init(board: &Board, attacker: Player, limit: u8) -> Self {
        let game = Game::init(board, attacker);
        let field = PotentialField::init(attacker, 2, board);
        Self::new(game, limit, field)
    }

    pub fn vcf_state(&mut self, max_limit: u8) -> VCFState {
        let game = self.game.clone();
        let limit = self.limit.min(max_limit);
        VCFState::with_swords(game, limit, self.synced_swords())
    }

    pub fn threat_state(&mut self, max_limit: u8) -> VCFState {
        let swords = self.synced_swords();
        let mut game = self.game.clone();
        game.play(None);
        let limit = if self.attacking() {
            self.limit - 1
        } else {
            self.limit
        }
        .min(max_limit);
        // A pass puts no stone, so the swords are still those of the board.
        VCFState::with_swords(game, limit, swords)
    }

    /// A copy of the swords for a nested VCF state. Synced here rather than
    /// in the copy, so that the next copy starts from what this one computed.
    fn synced_swords(&mut self) -> SwordMap {
        self.swords.sync(self.game.board());
        self.swords.clone()
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.game().is_forbidden_move(p)
    }

    pub fn check_event(&self) -> Option<Event> {
        self.game().check_event()
    }

    /// The key the child after `next_move` would have, without building it.
    /// Must stay in step with [`State::key`]. `next_move` must be an empty
    /// point: the stone is XORed into the board's hash, not played.
    pub fn next_key(&mut self, next_move: Option<Point>) -> Key {
        let limit = self.limit;
        let next_limit = if !self.attacking() { limit - 1 } else { limit };
        let attacker = self.attacker;
        let turn = self.game.turn;
        let stones = match next_move {
            Some(p) => apply_move(self.game.board().zobrist_hash(), turn, p),
            None => self.game.board().zobrist_hash(),
        };
        // The child has the other side to move.
        let position = apply_turn(stones, turn.opponent());
        Key::new(apply_attacker(position, attacker), next_limit)
    }

    /// The attacker's candidate moves, best first ([`Self::priority`]):
    /// the points where the attacker's potential is at least 3, or only
    /// those of `only`.
    pub fn sorted_attacks(&mut self, only: Option<Vec<Point>>) -> Vec<Point> {
        self.field.sync(self.game.board());
        let points = match only {
            Some(only) => only
                .into_iter()
                .map(|p| (p, self.field.get(p)))
                .filter(|&(_, o)| o >= 3)
                .collect(),
            None => self.field.collect(3),
        };
        self.sort_by_priority(points)
    }

    /// The defender's candidate moves `points`, best first
    /// ([`Self::priority`]).
    pub fn sorted_defences(&mut self, points: Vec<Point>) -> Vec<Point> {
        self.field.sync(self.game.board());
        let points = points.into_iter().map(|p| (p, self.field.get(p))).collect();
        self.sort_by_priority(points)
    }

    /// `points`, each with the attacker's potential there, without
    /// duplicates and by [`Self::priority`], then by potential, highest
    /// first.
    fn sort_by_priority(&mut self, points: Vec<(Point, u8)>) -> Vec<Point> {
        self.shapes.sync(self.game.board());
        let mut seen = [0u64; 4];
        let mut result: Vec<_> = points
            .into_iter()
            .filter(|&(p, _)| {
                let i = u8::from(p) as usize;
                let (word, bit) = (i / 64, 1 << (i % 64));
                let new = seen[word] & bit == 0;
                seen[word] |= bit;
                new
            })
            .map(|(p, o)| (p, (self.priority(p, o), o)))
            .collect();
        result.sort_by_key(|&(_, key)| std::cmp::Reverse(key));
        result.into_iter().map(|(p, _)| p).collect()
    }

    /// How promising a move to `p` is for the side to move, for move
    /// ordering: the attacker's `potential` there, plus what the move
    /// makes for the side to move ([`ShapeMap`]), the way a player sizes
    /// up a move:
    ///
    /// - a four or a three narrows the opponent's replies;
    /// - a move making threats along several lines at once is stronger
    ///   than its lines one by one;
    /// - for Black, a move that looks like a double-three or double-four,
    ///   or makes a three whose straight-four point does, is weak (the
    ///   move itself, if really forbidden, is not a candidate at all);
    /// - for White, a four or three with an eye where Black's stone looks
    ///   forbidden is strong, as Black cannot answer there.
    ///
    /// A defence is foremost a block, so the defender's own shapes count
    /// half.
    fn priority(&self, p: Point, potential: u8) -> i32 {
        let mover = self.game.turn;
        let mine = self.shapes.get(p, mover);
        let mut bonus = 5 * mine.count(Shape::Four) as i32
            + 2 * mine.count(Shape::Three) as i32
            + 5 * mine.count_from(Shape::Sword).saturating_sub(1) as i32;
        let (fours, threes) = self.shapes.forbidden_eyes(self.game.board(), p, mover);
        if mover.is_white() {
            bonus += 20 * fours as i32 + 10 * threes as i32;
        } else {
            bonus -= 2 * threes as i32;
            if mine.looks_forbidden() {
                bonus -= 20;
            }
        }
        if !self.attacking() {
            bonus /= 2;
        }
        potential as i32 + bonus
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
            let swords = game.board().rows_on(p, turn, Sword);
            for s in swords {
                result.extend(s.eyes());
            }
        }
        result
    }

    fn four_moves(&self) -> Vec<Point> {
        self.game()
            .board()
            .rows(self.game().turn, Sword)
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
            self.field.mark_stale(next_move);
            self.shapes.mark_stale(next_move);
            self.swords.mark_stale(next_move);
        }
    }

    fn after_undo(&mut self, maybe_last_move: Option<Point>) {
        if let Some(last_move) = maybe_last_move {
            self.field.mark_stale(last_move);
            self.shapes.mark_stale(last_move);
            self.swords.mark_stale(last_move);
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

    /// The field is only brought up to date when it is read, so after any
    /// run of plays and undos it has to read as a fresh one would.
    #[test]
    fn test_lazy_field_matches_a_fresh_one() -> Result<(), String> {
        let moves = "G7,K9,H9,F6".parse::<Points>()?.into_vec();
        let mut state = VCTState::init(&board(), Black, 4);
        for (k, &m) in moves.iter().enumerate() {
            state.play(Some(m));
            if k % 2 == 1 {
                // Read only every other move, so that several points are
                // stale at once.
                let fresh = &mut VCTState::init(state.game().board(), Black, 4);
                assert_eq!(state.sorted_attacks(None), fresh.sorted_attacks(None));
            }
        }
        for _ in &moves {
            state.undo();
        }
        let fresh = &mut VCTState::init(&board(), Black, 4);
        assert_eq!(state.sorted_attacks(None), fresh.sorted_attacks(None));
        Ok(())
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
