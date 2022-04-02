pub mod dfpns;
pub mod dfs;
pub mod lazy;
pub mod pns;

use super::resolver::Resolver;
use super::searcher::Searcher;
use super::state::State;
use crate::mate::mate::Mate;

pub trait Solver: Searcher + Resolver {
    fn solve(&mut self, state: &mut State) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}
