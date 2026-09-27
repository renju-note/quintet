mod bits;
#[allow(clippy::module_inception)]
mod board;
mod forbidden;
mod grid;
mod line;
mod player;
mod point;
mod row;
mod segment;
mod zobrist;

pub use bits::Bits;
pub use board::Board;
pub use forbidden::ForbiddenKind;
pub use grid::{Grid, LINE_COUNT};
pub use line::{Line, Rows};
pub use player::Player;
pub use point::{Direction, Index, POINT_COUNT, Point, Points, SIZE};
pub use row::{Row, RowKind};
pub use segment::{FIVE, Segment};
// Search-key helpers: the turn and a search's attacker are not properties of
// the board, so the mate solvers mix them into the position hash themselves.
pub(crate) use zobrist::{apply_attacker, apply_limit, apply_move, apply_turn};
