use super::field::*;
use crate::board::StructureKind::*;
use crate::board::*;
use crate::mate::game::*;
use crate::mate::vcf;
use std::collections::{HashMap, HashSet};

pub struct State {
    attacker: Player,
    game: Game,
    field: PotentialField,
}

impl State {
    pub fn new(attacker: Player, game: Game, field: PotentialField) -> Self {
        Self {
            attacker: attacker,
            game: game,
            field: field,
        }
    }

    pub fn init(board: Board, turn: Player) -> Self {
        let field = PotentialField::init(turn, 2, &board);
        let game = Game::init(board, turn);
        Self::new(turn, game, field)
    }

    pub fn attacker(&self) -> Player {
        self.attacker
    }

    pub fn game(&self) -> &'_ Game {
        &self.game
    }

    pub fn turn(&self) -> Player {
        self.game.turn()
    }

    pub fn last(&self) -> Player {
        self.game.last()
    }

    pub fn play(&mut self, next_move: Point) {
        self.game.play(next_move);
        self.field.update_along(next_move, self.game.board());
    }

    pub fn undo(&mut self, last2_move: Point) {
        let last_move = self.game.last_move();
        self.game.undo(last2_move);
        self.field.update_along(last_move, self.game.board());
    }

    pub fn sorted_attacks(&self, maybe_threat: Option<Mate>) -> Vec<Point> {
        let mut potentials = self.potentials();
        if let Some(threat) = maybe_threat {
            let threat_defences = self.threat_defences(&threat);
            let threat_defences = threat_defences.into_iter().collect::<HashSet<_>>();
            potentials.retain(|(p, _)| threat_defences.contains(p));
        }
        potentials.sort_by(|a, b| b.1.cmp(&a.1));
        potentials
            .into_iter()
            .map(|t| t.0)
            .filter(|&p| !self.game.is_forbidden_move(p))
            .collect()
    }

    pub fn sorted_defences(&self, threat: Mate) -> Vec<Point> {
        let mut result = self.threat_defences(&threat);
        let mut potential_map = HashMap::new();
        for (p, o) in self.potentials() {
            potential_map.insert(p, o);
        }
        result.sort_by(|a, b| {
            let oa = potential_map.get(a).unwrap_or(&0);
            let ob = potential_map.get(b).unwrap_or(&0);
            ob.cmp(oa)
        });
        result
            .into_iter()
            .filter(|&p| !self.game.is_forbidden_move(p))
            .collect()
    }

    pub fn as_vcf(&self) -> vcf::State {
        vcf::State::new(self.game().clone())
    }

    pub fn as_threat(&self) -> vcf::State {
        vcf::State::new(self.game().pass())
    }

    fn potentials(&self) -> Vec<(Point, u8)> {
        let min = if self.attacker == Player::Black { 4 } else { 3 };
        self.field.collect(min)
    }

    fn threat_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = self.direct_defences(threat);
        result.extend(self.counter_defences(threat));
        result.extend(self.four_moves());
        result
    }

    fn direct_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut result = threat.path.clone();
        match threat.win {
            Win::Fours(p1, p2) => {
                result.extend([p1, p2]);
            }
            Win::Forbidden(p) => {
                result.push(p);
                result.extend(self.game.board().neighbors(p, 5, true));
            }
            _ => (),
        }
        result
    }

    fn counter_defences(&self, threat: &Mate) -> Vec<Point> {
        let mut game = self.game.pass();
        let threater = game.turn();
        let mut result = vec![];
        for &p in &threat.path {
            let turn = game.turn();
            game.play(p);
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
        self.game
            .board()
            .structures(self.turn(), Sword)
            .flat_map(|s| s.eyes())
            .collect()
    }
}
