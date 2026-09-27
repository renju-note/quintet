use super::state::VCFState;
use crate::board::*;
use crate::feature::area::Area;
use crate::mate::budget::NodeBudget;
use crate::mate::game::*;
use crate::mate::mate::*;
use crate::mate::memo::Memo;
use crate::mate::solver::Solver;
use crate::mate::state::State;

/// How many dead ends a solver carries into a new search before it starts
/// dropping what older searches left behind. See [`Memo`].
pub const DEFAULT_CARRY_CAPACITY: usize = 1 << 16;

/// Depth-first VCF search.
///
/// The attacker plays four-making moves only, so every defender reply is
/// forced: the tree is a chain of `(attack, defence)` pairs and the search is
/// a plain DFS over them. `search` is the recursive routine; [`Solver::solve`]
/// wraps it in a generation.
pub struct DFSSolver {
    /// For each position already shown to have no VCF, the largest limit it
    /// was shown for — and so, since no VCF within `limit` is no VCF within
    /// less, every limit up to it.
    ///
    /// Keyed by [`Key::position`](crate::mate::Key::position) alone. Nothing this solver generates
    /// depends on `limit` (`move_pairs` and `neighbor_move_pairs` read only
    /// the board), so the tree at one limit is the tree at a larger one cut
    /// short, and the bound holds exactly. `search` is only ever called with
    /// the attacker to move, so the attacker in the position pins the turn.
    ///
    /// Each dead end also keeps the [zone](Self::search_zone) of the search
    /// that showed it, so that a search running into it can still tell its
    /// own.
    dead_ends: Memo<(u8, Area)>,
}

impl DFSSolver {
    pub fn init() -> Self {
        Self::with_carry_capacity(DEFAULT_CARRY_CAPACITY)
    }

    pub fn with_carry_capacity(carry_capacity: usize) -> Self {
        Self {
            dead_ends: Memo::new(carry_capacity),
        }
    }

    /// The search itself, with the attacker to move. Unlike
    /// [`Solver::solve`] it opens no generation, so a caller that asks it
    /// many times within one question — [`IDDFSSolver`] deepening, the VCT
    /// solver evaluating threats — does not churn the memo.
    ///
    /// [`IDDFSSolver`]: super::IDDFSSolver
    pub fn search(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        self.search_zone(state, budget, &mut Area::new())
    }

    /// [`Self::search`], also adding to `zone` the points where one more
    /// attacker stone could change its answer.
    ///
    /// When the search finds no VCF (and the budget did not run out), a
    /// stone put on any other empty point — besides one in a segment that
    /// already holds two of the attacker's stones, which the caller has to
    /// check itself — leaves the attacker without a VCF too. The zone is
    /// what the search looked at:
    ///
    /// - around each attack, its four lines up to four cells away: the
    ///   segments the attack joins, where the stone may find two of the
    ///   attacker's stones that were not there before;
    /// - the eyes of the defender's fours, where the stone would block
    ///   them;
    /// - around each point whose forbiddenness mattered, its four lines up
    ///   to five cells away: an attack of Black's found forbidden, the eye
    ///   of White's four that Black could not block, and each of Black's
    ///   blocks when White attacks. This one is an approximation, as in
    ///   `VCTState::threat_defences`: a three there may be no real three
    ///   because of a point further away.
    ///
    /// Nothing else is looked at, and the rest of the board plays no part
    /// in the answer. After a VCF was found `zone` means nothing.
    pub fn search_zone(
        &mut self,
        state: &mut VCFState,
        budget: &mut NodeBudget,
        zone: &mut Area,
    ) -> Option<Mate> {
        if state.limit() == 0 {
            return None;
        }
        if !budget.consume() {
            return None;
        }

        let key = state.key();
        if let Some((limit, known)) = self.dead_ends.get(key.position)
            && key.limit <= *limit
        {
            *zone |= known;
            return None;
        }
        let mut own = Area::new();
        let result = self.search_move_pairs(state, budget, &mut own);
        *zone |= own;
        // A search that gave up proves nothing, so it must not be memoized.
        // Any dead end already there is for a smaller limit: its tree is cut
        // shorter than this one, and so is its zone.
        if result.is_none() && !budget.is_exhausted() {
            self.dead_ends.insert(key.position, (key.limit, own));
        }
        result
    }

    fn search_move_pairs(
        &mut self,
        state: &mut VCFState,
        budget: &mut NodeBudget,
        zone: &mut Area,
    ) -> Option<Mate> {
        if let Some(event) = state.check_event() {
            return match event {
                Defeated(end) => {
                    match end {
                        Fours(e1, e2) => {
                            zone.insert(e1);
                            zone.insert(e2);
                        }
                        Forbidden(e) => *zone |= Area::around(e, 5),
                        Unknown => {}
                    }
                    None
                }
                Forced(p) => {
                    zone.insert(p);
                    state
                        .forced_move_pair(p)
                        .and_then(|(a, d)| self.search_attack(state, a, d, budget, zone))
                }
            };
        }

        let neighbor_pairs = state.neighbor_move_pairs();
        for &(attack, defence) in &neighbor_pairs {
            let result = self.search_attack(state, attack, defence, budget, zone);
            if result.is_some() {
                return result;
            }
            if budget.is_exhausted() {
                return None;
            }
        }

        let pairs = state.move_pairs();
        for &(attack, defence) in &pairs {
            if neighbor_pairs.iter().any(|(a, _)| *a == attack) {
                continue;
            }
            let result = self.search_attack(state, attack, defence, budget, zone);
            if result.is_some() {
                return result;
            }
            if budget.is_exhausted() {
                return None;
            }
        }

        None
    }

    fn search_attack(
        &mut self,
        state: &mut VCFState,
        attack: Point,
        defence: Point,
        budget: &mut NodeBudget,
        zone: &mut Area,
    ) -> Option<Mate> {
        // The defence is on the same segment, within four cells.
        *zone |= Area::around(attack, 4);
        let result = state.with_move(Some(attack), |s| {
            self.search_defence(s, defence, budget, zone)
                .map(|m| m.prepend(attack))
        });

        // Few attacks are forbidden, so whether this one is is only asked
        // once it would win. A forbidden attack whose search found nothing
        // has its zone all the same: whether or not it is forbidden, a stone
        // outside it leaves the search finding nothing.
        if result.is_some() && state.is_forbidden_move(attack) {
            *zone |= Area::around(attack, 5);
            return None;
        }
        result
    }

    fn search_defence(
        &mut self,
        state: &mut VCFState,
        defence: Point,
        budget: &mut NodeBudget,
        zone: &mut Area,
    ) -> Option<Mate> {
        if let Some(Defeated(end)) = state.check_event() {
            return Some(Mate::new(end, vec![]));
        }

        // The block was not forbidden, but a White stone nearby might make it.
        if state.game().turn.is_black() {
            *zone |= Area::around(defence, 5);
        }
        state.with_move(Some(defence), |s| {
            self.search_zone(s, budget, zone)
                .map(|m| m.prepend(defence))
        })
    }
}

impl Solver for DFSSolver {
    type State = VCFState;

    fn solve(&mut self, state: &mut VCFState, budget: &mut NodeBudget) -> Option<Mate> {
        self.advance_generation();
        self.search(state, budget)
    }

    fn clear(&mut self) {
        self.dead_ends.clear();
    }

    fn advance_generation(&mut self) {
        self.dead_ends.advance_generation();
    }

    fn memo_len(&self) -> usize {
        self.dead_ends.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Player::{Black, White};
    use crate::feature::shape::{Shape, ShapeMap};

    /// The boards of `test_vct_black`, `test_vct_white` and
    /// `test_vct_dual_forbiddens` in `solve.rs`.
    fn boards() -> Vec<Board> {
        [
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . x . . . . . .
             . . . . . . . o . . . . . . .
             . . . . . . . o x o . . . . .
             . . . . . . x o . x . . . . .
             . . . . . . . x o . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . o . . o . . . . .
             . . . . . . o x x . . . . . .
             . . . . . . . o . . . . . . .
             . . . . . . . . x . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
            "
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . x . . . . . . .
             . . . . . . . o o . . . . . .
             . . . . . . . o x . . . . . .
             . . . . . . . x x o . . . . .
             . . . . . . o o x . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
             . . . . . . . . . . . . . . .
            ",
        ]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect()
    }

    /// Where the attacker has no VCF, one more stone gives it one only in
    /// the zone or in a segment already holding two of its stones.
    #[test]
    fn test_zone_holds_every_stone_that_gives_a_vcf() {
        let budget = &mut NodeBudget::unlimited();
        let mut tested = 0;
        for board in boards() {
            let shapes = ShapeMap::init(&board);
            for attacker in [Black, White] {
                let mut zone = Area::new();
                let state = &mut VCFState::init(&board, attacker, 5);
                if DFSSolver::init()
                    .search_zone(state, budget, &mut zone)
                    .is_some()
                {
                    continue;
                }
                tested += 1;

                let mut outside = 0;
                for p in board.empties() {
                    let next = board.put(attacker, p);
                    let state = &mut VCFState::init(&next, attacker, 5);
                    let gives = DFSSolver::init().search(state, budget).is_some();
                    let ruled_out =
                        !zone.contains(p) && shapes.get(p, attacker).count_from(Shape::Sword) == 0;
                    assert!(!(gives && ruled_out), "{attacker:?} {p}");
                    outside += ruled_out as usize;
                }
                // The zone does rule something out.
                assert!(outside > 0, "{attacker:?}");
            }
        }
        assert!(tested >= 3);
    }
}
