use super::resolver;
use super::searcher2;
use super::state::State;
use crate::mate::mate::*;

pub trait Solver: searcher2::Searcher + resolver::Resolver {
    fn solve(&mut self, state: &mut State) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}
