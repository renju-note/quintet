use super::super::board::RowKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf::solve_vcf;
use std::collections::HashSet;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MoveKind {
    LastFourCloser,
    NextFourMove,
    LastThreeCloser,
    NextThreeMove,
}

pub struct MoveSearcher {
    state: GameState,
    last_four_inited: bool,
    last_four_count: usize,
    last_four_closer: Option<Point>,
    next_four_inited: bool,
    next_four_moves: VecDeque<Point>,
    last_three_inited: bool,
    last_three_closers: VecDeque<Point>,
    next_three_inited: bool,
    next_three_moves: VecDeque<Point>,
    popped: HashSet<Point>,
}

impl MoveSearcher {
    pub fn new(state: &GameState) -> Self {
        Self {
            state: state.clone(),
            last_four_inited: false,
            last_four_count: 0,
            last_four_closer: None,
            next_four_inited: false,
            next_four_moves: VecDeque::new(),
            last_three_inited: false,
            last_three_closers: VecDeque::new(),
            next_three_inited: false,
            next_three_moves: VecDeque::new(),
            popped: HashSet::new(),
        }
    }

    pub fn last_four_found(&mut self) -> bool {
        self.init_last_four();
        self.last_four_count >= 1
    }

    pub fn pop(&mut self, kind: MoveKind) -> Option<Point> {
        match kind {
            MoveKind::LastFourCloser => {
                self.init_last_four();
                self.last_four_closer
                    .take()
                    .filter(|&p| !self.state.is_forbidden_move(p))
            }
            MoveKind::NextFourMove => {
                self.init_next_four();
                Self::pop_valid(&self.state, &mut self.next_four_moves, &mut self.popped)
            }
            MoveKind::LastThreeCloser => {
                self.init_last_three();
                Self::pop_valid(&self.state, &mut self.last_three_closers, &mut self.popped)
            }
            MoveKind::NextThreeMove => {
                self.init_next_three();
                Self::pop_valid(&self.state, &mut self.next_three_moves, &mut self.popped)
            }
        }
    }

    pub fn get_threat(&self, p: Point) -> Option<Vec<Point>> {
        let state = self.state.play(p);
        solve_vcf(
            &state.board(),
            self.state.next_player(),
            u8::MAX, // TODO
            false,
        )
    }

    fn init_last_four(&mut self) {
        if self.last_four_inited {
            return;
        }
        let mut last_four_eyes = self.state.row_eyes_along_last_move(Four);
        self.last_four_count = last_four_eyes.len();
        if self.last_four_count == 1 {
            self.last_four_closer = last_four_eyes.pop();
        }
        self.last_four_inited = true;
    }

    fn init_next_four(&mut self) {
        if self.next_four_inited {
            return;
        }
        // TODO: find three eyes first
        self.next_four_moves = self.state.row_eyes(self.state.next_player(), Sword).into();
        self.next_four_inited = true;
    }

    fn init_last_three(&mut self) {
        if self.last_three_inited {
            return;
        }
        let last_threes = self.state.rows(self.state.last_player(), Three);
        for r in last_threes {
            self.last_three_closers.extend(r.into_iter_eyes());
        }
        // TODO: outer closer and summer closer
        self.last_three_inited = true;
    }

    fn init_next_three(&mut self) {
        if self.next_three_inited {
            return;
        }
        self.next_three_moves = self.state.row_eyes(self.state.next_player(), Two).into();
        // TODO: remove fake three (= another eye is forbidden)
        self.next_three_inited = true;
    }

    fn pop_valid(
        state: &GameState,
        queue: &mut VecDeque<Point>,
        popped: &mut HashSet<Point>,
    ) -> Option<Point> {
        while let Some(p) = queue.pop_front() {
            if popped.contains(&p) {
                continue;
            }
            popped.insert(p);
            if !state.is_forbidden_move(p) {
                return Some(p);
            }
        }
        None
    }
}
