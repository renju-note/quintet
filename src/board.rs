#[allow(clippy::module_inception)]
mod board;
mod forbidden;
mod line;
mod player;
mod point;
mod potential;
mod sequence;
mod square;
mod structure;
mod zobrist;

pub use board::Board;
pub use forbidden::ForbiddenKind;
pub use line::Line;
pub use player::Player;
pub use point::{Direction, Index, Point, Points, RANGE};
pub use potential::{Potentials, VICTORY};
pub use square::Square;
pub use structure::{Structure, StructureKind};
// Search-key helpers: the turn and a search's attacker are not properties of
// the board, so the mate solvers mix them into the position hash themselves.
pub(crate) use zobrist::{apply_attacker, apply_turn};
