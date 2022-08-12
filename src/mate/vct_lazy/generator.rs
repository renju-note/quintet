mod eager;
mod lazy;

pub use eager::EagerGenerator;
pub use lazy::LazyGenerator;

use crate::board::Point;
use crate::mate::vct_lazy::proof::Node;
use crate::mate::vct_lazy::state::VCTState;

pub trait Generator {
    fn generate_attacks(
        &mut self,
        state: &mut VCTState,
        threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node>;

    fn generate_defences(
        &mut self,
        state: &mut VCTState,
        threshold: Node,
    ) -> Result<Vec<(Point, Node)>, Node>;
}
