mod eager_dfpns;
mod eager_dfs;
mod eager_pns;
mod lazy_dfpns;

pub use eager_dfpns::EagerDFPNSSolver;
pub use eager_dfs::EagerDFSSolver;
pub use eager_pns::EagerPNSSolver;
pub use lazy_dfpns::LazyDFPNSSolver;

use super::resolver::Resolver;
use super::searcher::Searcher;
use super::state::VCTState;
use crate::mate::mate::Mate;

pub trait Solver: Searcher + Resolver {
    fn solve(&mut self, state: &mut VCTState) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}
