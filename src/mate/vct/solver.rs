pub mod eager_dfpns;
pub mod eager_dfs;
pub mod eager_pns;
pub mod lazy_dfpns;

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
