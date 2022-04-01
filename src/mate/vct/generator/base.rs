use crate::board::*;
use crate::mate::vct::state::State;
use crate::mate::vct::table::*;

pub trait Generator {
    fn find_attacks(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node>;

    fn find_defences(&mut self, state: &mut State, threshold: Node) -> Result<Vec<Point>, Node>;
}
