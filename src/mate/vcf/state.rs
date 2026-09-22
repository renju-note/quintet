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
