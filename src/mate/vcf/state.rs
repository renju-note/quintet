use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::state::State;

#[derive(Clone)]
pub struct VCFState {
    game: Game,
    pub attacker: Player,
    pub limit: u8,
}

impl VCFState {
    pub fn new(game: Game, limit: u8) -> Self {
        Self {
            attacker: game.turn,
            game,
            limit,
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

    pub fn neighbor_move_pairs(&self) -> Vec<(Point, Point)> {
        let mut result = vec![];
        if let Some(last2_move) = self.game.last2_move() {
            let swords = self
                .game
                .board()
                .structures_on(last2_move, self.game.turn, Sword);
            Self::push_eyes_pairs(swords, &mut result);
        }
        result
    }

    pub fn move_pairs(&self) -> Vec<(Point, Point)> {
        let mut result = vec![];
        let swords = self.game.board().structures(self.game.turn, Sword);
        Self::push_eyes_pairs(swords, &mut result);
        result
    }

    /// Both ways round for each sword, written as a loop rather than
    /// `flat_map(..).collect()`: this runs at every node, and driving the
    /// nested iterator costs more than the two pushes it ends in.
    fn push_eyes_pairs(swords: impl Iterator<Item = Structure>, out: &mut Vec<(Point, Point)>) {
        for sword in swords {
            let (e1, e2) = Self::sword_eyes(&sword);
            out.push((e1, e2));
            out.push((e2, e1));
        }
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
        let state = VCFState::init(&board(), Black, 5);
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
