use crate::board::RowKind::*;
use crate::board::*;
use crate::feature::area::Area;
use crate::feature::shape::{Shape, ShapeMap};
use crate::feature::sword::SwordMap;
use crate::mate::game::*;
use crate::mate::mate::Mate;
use crate::mate::state::{Key, State};
use crate::mate::vcf::VCFState;

pub struct VCTState {
    game: Game,
    attacker: Player,
    limit: u8,
    shapes: ShapeMap,
    /// The swords, kept only to hand to the nested VCF states.
    swords: SwordMap,
}

impl VCTState {
    pub fn new(game: Game, limit: u8) -> Self {
        let swords = SwordMap::init(game.board());
        let shapes = ShapeMap::init(game.board());
        Self {
            attacker: game.turn,
            game,
            limit,
            shapes,
            swords,
        }
    }

    pub fn init(board: &Board, attacker: Player, limit: u8) -> Self {
        let game = Game::init(board, attacker);
        Self::new(game, limit)
    }

    pub fn vcf_state(&mut self, max_limit: u8) -> VCFState {
        let limit = self.limit.min(max_limit);
        // Two plies per attack.
        let game = self.game.fork(2 * limit as usize);
        VCFState::with_swords(game, limit, self.synced_swords())
    }

    pub fn threat_state(&mut self, max_limit: u8) -> VCFState {
        let swords = self.synced_swords();
        let limit = if self.attacking() {
            self.limit - 1
        } else {
            self.limit
        }
        .min(max_limit);
        // The pass, and two plies per attack.
        let mut game = self.game.fork(1 + 2 * limit as usize);
        game.play(None);
        // A pass puts no stone, so the swords are still those of the board.
        VCFState::with_swords(game, limit, swords)
    }

    /// A copy of the swords for a nested VCF state. Synced here rather than
    /// in the copy, so that the next copy starts from what this one computed.
    fn synced_swords(&mut self) -> SwordMap {
        self.swords.sync(self.game.board());
        self.swords.clone()
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
    /// the points where the attacker makes at least a [`Shape::Two`] on
    /// some line, or only those of `only`.
    pub fn sorted_attacks(&mut self, only: Option<Vec<Point>>) -> Vec<Point> {
        self.shapes.sync(self.game.board());
        let (shapes, attacker) = (&self.shapes, self.attacker);
        let makes_two = |&p: &Point| shapes.get(p, attacker).count_from(Shape::Two) > 0;
        // Filtered in place or into room for every point: this runs at
        // every attacker node, and a `Vec` grown step by step was a good
        // part of its cost.
        let points = match only {
            Some(mut only) => {
                only.retain(makes_two);
                only
            }
            None => {
                let mut points = Vec::with_capacity(POINTS);
                points.extend(self.game.board().empties().filter(makes_two));
                points
            }
        };
        self.sort_by_priority(points)
    }

    /// Whether the attacker's move to `p` may be a threat, given that the
    /// attacker has no VCF now and `zone` is the zone of the search that
    /// showed it ([`DFSSolver::search_zone`](crate::mate::DFSSolver::search_zone)).
    /// For a VCF to appear, the stone has to be in the zone or in a segment
    /// already holding two of the attacker's stones ([`Shape::Sword`] or
    /// more along some line), which the zone leaves out. The attacker must
    /// be to move.
    pub fn may_threaten(&mut self, p: Point, zone: &Area) -> bool {
        debug_assert!(self.attacking());
        self.shapes.sync(self.game.board());
        zone.contains(p) || self.shapes.get(p, self.attacker).count_from(Shape::Sword) > 0
    }

    /// The defender's candidate moves `points`, best first
    /// ([`Self::priority`]).
    pub fn sorted_defences(&mut self, points: Vec<Point>) -> Vec<Point> {
        self.sort_by_priority(points)
    }

    /// `points` without duplicates, by [`Self::priority`], highest first;
    /// equal ones keep their order.
    fn sort_by_priority(&mut self, points: Vec<Point>) -> Vec<Point> {
        self.shapes.sync(self.game.board());
        let mut points = points;
        let mut seen = Area::new();
        points.retain(|&p| seen.insert(p));
        // Sized exactly, where a `filter` before the `map` left the `Vec`
        // to grow.
        let mut result: Vec<_> = points.into_iter().map(|p| (p, self.priority(p))).collect();
        result.sort_by_key(|&(_, key)| std::cmp::Reverse(key));
        result.into_iter().map(|(p, _)| p).collect()
    }

    /// How promising a move to `p` is for the side to move, for move
    /// ordering: what the move makes for the side to move ([`ShapeMap`]),
    /// the way a player sizes up a move:
    ///
    /// - every line counts by the shape it makes
    ///   ([`Shapes::total`](crate::feature::shape::Shapes::total)), a two
    ///   or a sword too, as the next threats are made of them;
    /// - a four or a three narrows the opponent's replies;
    /// - a move making threats along several lines at once is stronger
    ///   than its lines one by one;
    /// - for Black, a move that looks like a double-three or double-four,
    ///   or makes a three whose straight-four point does, is weak (the
    ///   move itself, if really forbidden, is disproven when it is first
    ///   searched);
    /// - for White, a four or three with an eye where Black's stone looks
    ///   forbidden is strong, as Black cannot answer there.
    ///
    /// A defence leaves out the first. Ordering defences by how much the
    /// attacker wants the point, by its stones or its shapes there, made
    /// the search larger on the benchmark than these terms alone.
    fn priority(&self, p: Point) -> i32 {
        let mover = self.game.turn;
        let mine = self.shapes.get(p, mover);
        let mut value = if self.attacking() {
            mine.total() as i32
        } else {
            0
        };
        value += 5 * mine.count(Shape::Four) as i32
            + 2 * mine.count(Shape::Three) as i32
            + 5 * mine.count_from(Shape::Sword).saturating_sub(1) as i32;
        let (fours, threes) = self.shapes.forbidden_eyes(self.game.board(), p, mover);
        if mover.is_white() {
            value += 20 * fours as i32 + 10 * threes as i32;
        } else {
            value -= 2 * threes as i32;
            if mine.looks_forbidden() {
                value -= 20;
            }
        }
        value
    }

    /// The biggest [`Shape`] a stone of the side to move at `p` makes on
    /// any line. Read from the shapes as of the last
    /// [`Self::sorted_attacks`] or [`Self::sorted_defences`] of this
    /// position.
    pub fn best_shape(&self, p: Point) -> Shape {
        debug_assert!(self.shapes.is_synced());
        self.shapes.get(p, self.game.turn).best()
    }

    pub fn empties(&self) -> Vec<Point> {
        self.game().board().empties().collect()
    }

    pub fn threat_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = threat.path.clone();
        result.extend(self.end_breakers(threat.end.clone()));
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
            self.shapes.mark_stale(next_move);
            self.swords.mark_stale(next_move);
        }
    }

    fn after_undo(&mut self, maybe_last_move: Option<Point>) {
        if let Some(last_move) = maybe_last_move {
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

    /// The shapes are only brought up to date when they are read, so after
    /// any run of plays and undos they have to read as fresh ones would.
    #[test]
    fn test_lazy_shapes_match_fresh_ones() -> Result<(), String> {
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
        assert_eq!(attacking_black.limit(), attacking_white.limit());
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
