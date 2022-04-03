pub mod eager;
pub mod lazy;

pub use eager::EagerGenerator;
pub use lazy::LazyGenerator;

use crate::board::Point;
use crate::mate::vct::proof::Node;
use crate::mate::vct::state::State;

pub trait Generator {
    fn generate_attacks(
        &mut self,
        state: &mut State,
        threshold: Node,
    ) -> Result<Vec<(Point, u32)>, Node>;

    fn generate_defences(
        &mut self,
        state: &mut State,
        threshold: Node,
    ) -> Result<Vec<(Point, u32)>, Node>;
}
