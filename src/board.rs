mod line;
mod square;

pub use line::{Line, Stones};
pub use square::{to_index, to_point, Direction, Facet, Index, Point, Square, BOAD_SIZE};
