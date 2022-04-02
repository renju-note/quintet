pub mod dfpns;
pub mod dfs;
pub mod lazy;
pub mod pns;

use super::resolver;
use super::searcher;
use super::state::State;
use crate::mate::mate::*;

pub trait Solver: searcher::Searcher + resolver::Resolver {
    fn solve(&mut self, state: &mut State) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}
