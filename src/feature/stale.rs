use crate::board::*;

/// The lines a lazily updated map has yet to recompute, one bit per line
/// (by [`Grid::line_key`]).
///
/// A move only changes the (at most four) lines through it, so a map
/// [`mark`](Self::mark)s those as it goes and recomputes them in one pass
/// before its next read, [`take`](Self::take)ing the keys. The searches
/// move far more often than they read, so marking rather than recomputing
/// at once is most of what makes such a map cheap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaleLines(u128);

impl StaleLines {
    /// Every line: what a map that has read nothing yet has to compute.
    pub fn all() -> Self {
        Self((1 << LINE_COUNT) - 1)
    }

    /// Notes that a stone was put on or taken off `p`.
    pub fn mark(&mut self, p: Point) {
        for d in Direction::ALL {
            if let Some(k) = Grid::line_key(d, p.to_index(d).i) {
                self.0 |= 1 << k;
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// The keys of the stale lines, leaving none stale.
    pub fn take(&mut self) -> Bits<u128> {
        Bits(std::mem::take(&mut self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Direction::*;

    #[test]
    fn test_mark_and_take() -> Result<(), String> {
        let mut stale = StaleLines::all();
        assert_eq!(stale.take().len(), LINE_COUNT);
        assert!(stale.is_empty());

        // H8 is on a stored line in every direction; A1's descending
        // diagonal is too short to hold a five, and is not stored.
        stale.mark("H8".parse()?);
        assert_eq!(stale.take().len(), 4);
        stale.mark("A1".parse()?);
        let keys: Vec<_> = stale
            .take()
            .map(|k| Grid::line_from_key(k as usize).0)
            .collect();
        assert_eq!(keys, [Vertical, Horizontal, Ascending]);
        Ok(())
    }
}
