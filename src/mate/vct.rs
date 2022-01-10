use super::super::board::RowKind::*;
use super::super::board::*;
use super::state::*;
use super::vcf;
use std::collections::HashSet;
use std::collections::VecDeque;

pub fn solve(state: &GameState, depth: u8, searched: &mut HashSet<u64>) -> Option<Vec<Point>> {
    if depth == 0 {
        return None;
    }

    // check if already searched (and was dead-end)
    let hash = state.board_hash();
    if searched.contains(&hash) {
        return None;
    }
    searched.insert(hash);

    if let Some(result) = vcf::solve(state, depth, searched) {
        return Some(result);
    }

    for attack in VCTAttacks::new(state, depth) {
        let state = state.play(attack);
        let defences = VCTDefences::new(&state);
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

struct VCTAttacks {
    searcher: MoveSearcher,
    threat_depth: u8,
}

impl VCTAttacks {
    fn new(state: &GameState, threat_depth: u8) -> Self {
        Self {
            searcher: MoveSearcher::new(state),
            threat_depth: threat_depth,
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
                .filter(|&p| self.searcher.get_threat(p, self.threat_depth).is_some());
        }
        if let Some(e) = self.searcher.pop(MoveKind::NextThreeMove) {
            return Some(e);
        }
        if let Some(e) = self.searcher.pop(MoveKind::NextFourMove) {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MoveKind {
    LastFourCloser,
    NextFourMove,
    LastThreeCloser,
    NextThreeMove,
}

struct MoveSearcher {
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

    pub fn get_threat(&self, p: Point, depth: u8) -> Option<Vec<Point>> {
        let mut state = self.state.play(p);
        state.pass_mut();
        vcf::solve(&state, depth, &mut HashSet::new())
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
        let state = GameState::new(&board, Player::Black, Point(8, 10));
        let result = solve(&state, 4, &mut HashSet::new()).map(|ps| Points(ps).to_string());
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
        let state = GameState::new(&board, Player::White, Point(6, 9));
        let result = solve(&state, 3, &mut HashSet::new()).map(|ps| Points(ps).to_string());
        let solution = "I10,I8,J11,K12,F7".to_string();
        assert_eq!(result, Some(solution));

        Ok(())
    }
}
