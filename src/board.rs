#[allow(clippy::module_inception)]
mod board;
mod forbidden;
mod grid;
mod line;
mod player;
mod point;
mod segment;
mod structure;
mod zobrist;

pub use board::Board;
pub use forbidden::ForbiddenKind;
pub use grid::{Grid, LINE_NUM};
pub use line::{Line, Structures};
pub use player::Player;
pub use point::{Direction, Index, Point, Points, RANGE};
pub use segment::{Segment, VICTORY};
pub use structure::{Structure, StructureKind};
// Search-key helpers: the turn and a search's attacker are not properties of
// the board, so the mate solvers mix them into the position hash themselves.
pub(crate) use zobrist::{apply_attacker, apply_move, apply_n, apply_turn};
