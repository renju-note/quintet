use crate::analysis::sword::SwordField;
use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::state::State;

#[derive(Clone)]
pub struct VCFState {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
    /// The attacker's swords, where the moves come from.
    swords: SwordField,
}

impl VCFState {
    pub fn new(game: Game, limit: u8) -> Self {
        let swords = SwordField::init(game.turn, game.board());
        Self::with_swords(game, limit, swords)
    }

    /// Like [`Self::new`], reusing swords someone already keeps for the side
    /// to move — the VCT solver does, for its nested searches. They need not
    /// be in sync with the board yet.
    pub fn with_swords(game: Game, limit: u8, swords: SwordField) -> Self {
        assert_eq!(swords.player(), game.turn);
        Self {
            attacker: game.turn,
            game,
            limit,
            swords,
        }
    }

    pub fn init(board: &Board, attacker: Player, limit: u8) -> Self {
        let game = Game::init(board, attacker);
        Self::new(game, limit)
    }

    pub fn is_forbidden_move(&self, p: Point) -> bool {
        self.game().is_forbidden_move(p)
    }

    pub fn check_event(&self) -> Option<Event> {
        self.game().check_event()
    }

    pub fn forced_move_pair(&self, forced_move: Point) -> Option<(Point, Point)> {
        self.game
            .board()
            .structures_on(forced_move, self.game.turn, Sword)
            .find_map(|sword| match Self::sword_eyes(&sword) {
                (e1, e2) if e1 == forced_move => Some((e1, e2)),
                (e1, e2) if e2 == forced_move => Some((e2, e1)),
                _ => None,
            })
    }

    /// The swords through the attacker's previous stone, both ways round.
    /// Called with the attacker to move.
    pub fn neighbor_move_pairs(&mut self) -> Vec<(Point, Point)> {
        match self.game.last2_move() {
            Some(last2_move) => self.synced_swords().move_pairs_on(last2_move),
            None => vec![],
        }
    }

    /// Every sword of the attacker, both ways round. Called with the
    /// attacker to move.
    pub fn move_pairs(&mut self) -> Vec<(Point, Point)> {
        self.synced_swords().move_pairs()
    }

    fn synced_swords(&mut self) -> &SwordField {
        debug_assert_eq!(self.game.turn, self.attacker);
        self.swords.sync(self.game.board());
        &self.swords
    }

    fn sword_eyes(sword: &Structure) -> (Point, Point) {
        let mut eyes = sword.eyes();
        // A `Sword` is three stones in five cells with no opponent stone, so
        // it has exactly two eyes.
        let e1 = eyes.next().unwrap();
        let e2 = eyes.next().unwrap();
        (e1, e2)
    }
}

impl State for VCFState {
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
        self.swords.play(next_move);
    }

    fn after_undo(&mut self, _maybe_last_move: Option<Point>) {
        self.swords.undo();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::{Black, White};

    fn point(s: &str) -> Point {
        s.parse().unwrap()
    }

    fn pairs(ps: &[(&str, &str)]) -> Vec<(Point, Point)> {
        ps.iter().map(|&(a, d)| (point(a), point(d))).collect()
    }

    /// Two swords for Black, each blocked on one side by White, so each has
    /// exactly one five-cell window: C3-C5 (eyes C6, C7) and H8-J8 (eyes
    /// K8, L8).
    fn board() -> Board {
        "C3,C4,C5,H8,I8,J8/C2,G8".parse().unwrap()
    }

    #[test]
    fn test_move_pairs() {
        // Each sword gives a four at either eye; the other eye is the
        // forced defence.
        let mut state = VCFState::init(&board(), Black, 5);
        let expected = pairs(&[("C6", "C7"), ("C7", "C6"), ("K8", "L8"), ("L8", "K8")]);
        assert_eq!(state.move_pairs(), expected);

        // White has no sword.
        assert_eq!(VCFState::init(&board(), White, 5).move_pairs(), []);
    }

    #[test]
    fn test_forced_move_pair() {
        // When the attacker has to block at `p`, the VCF only goes on if
        // `p` also makes a four: an eye of one of its swords.
        let state = VCFState::init(&board(), Black, 5);
        assert_eq!(
            state.forced_move_pair(point("L8")),
            Some((point("L8"), point("K8")))
        );
        assert_eq!(
            state.forced_move_pair(point("C6")),
            Some((point("C6"), point("C7")))
        );
        assert_eq!(state.forced_move_pair(point("M8")), None);
    }

    #[test]
    fn test_neighbor_move_pairs() {
        // Only the swords through the attacker's previous move, which is
        // where a continuation is most likely.
        let mut state = VCFState::init(&"C3,C4,C5,H8,I8/C2,G8".parse().unwrap(), Black, 5);
        assert_eq!(state.neighbor_move_pairs(), []);
        state.play(Some(point("J8")));
        state.play(Some(point("A1")));
        let expected = pairs(&[("K8", "L8"), ("L8", "K8")]);
        assert_eq!(state.neighbor_move_pairs(), expected);
        assert_eq!(state.move_pairs().len(), 4);
    }
}
