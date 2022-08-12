mod lazy_dfpns;

pub use lazy_dfpns::LazyDFPNSSolver;

use super::resolver::Resolver;
use super::searcher::Searcher;
use super::state::VCTState;
use crate::mate::mate::Mate;

pub trait LazySolver: Searcher + Resolver {
    fn solve(&mut self, state: &mut VCTState) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}
