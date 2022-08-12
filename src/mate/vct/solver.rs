mod dfpns;
mod dfs;
mod pns;

pub use dfpns::DFPNSVCTSolver;
pub use dfs::DFSVCTSolver;
pub use pns::PNSVCTSolver;

use super::resolver::Resolver;
use super::searcher::Searcher;
use super::state::VCTState;
use crate::mate::mate::Mate;

pub trait VCTSolver: Searcher + Resolver {
    fn solve(&mut self, state: &mut VCTState) -> Option<Mate> {
        if self.search(state) {
            self.resolve(state)
        } else {
            None
        }
    }
}
