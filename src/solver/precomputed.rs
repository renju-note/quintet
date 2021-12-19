use super::super::board::{Bits, Player, RowKind, Sequence, SequenceCache};

const MAX_LENGTH: u8 = 8;
const SIZE: usize = 65536;

pub struct PrecomputedSequenceCache {
  pub player: Player,
  pub kind: RowKind,
  cache: [Option<Sequence>; SIZE],
}

impl PrecomputedSequenceCache {
  pub fn new(player: Player, kind: RowKind) -> Self {
    const DEFAULT: Option<Sequence> = None;
    let mut result = PrecomputedSequenceCache {
      player: player,
      kind: kind,
      cache: [DEFAULT; SIZE],
    };
    result.initialize();
    result
  }

  pub fn get(&self, stones: Bits, blanks: Bits) -> Option<Sequence> {
    let key = self.key(stones, blanks);
    self.cache[key].clone()
  }

  fn key(&self, stones: Bits, blanks: Bits) -> usize {
    let filter = (0b1 << MAX_LENGTH) - 1;
    let stones = stones & filter;
    let blanks = blanks & filter;
    ((stones << MAX_LENGTH) + blanks) as usize
  }

  fn initialize(&mut self) {
    for stones in 0..(0b1 << MAX_LENGTH) {
      for blanks in 0..(0b1 << MAX_LENGTH) {
        // stones and blanks are never overlaps
        if stones & blanks != 0b0 {
          continue;
        }
        let key = self.key(stones, blanks);
        self.cache[key] = Sequence::find(self.player, self.kind, stones, blanks);
      }
    }
  }
}

impl SequenceCache for PrecomputedSequenceCache {
  fn player(&self) -> Player {
    self.player
  }

  fn kind(&self) -> RowKind {
    self.kind
  }

  fn new(player: Player, kind: RowKind) -> Self {
    PrecomputedSequenceCache::new(player, kind)
  }

  fn get(&self, stones: Bits, blanks: Bits) -> Option<Sequence> {
    self.get(stones, blanks)
  }
}

#[cfg(test)]
mod tests {
  use super::Player::*;
  use super::RowKind::*;
  use super::*;

  #[test]
  fn test_new_get() {
    let cache = PrecomputedSequenceCache::new(Black, Sword);

    let result = cache.get(0b0100110, 0b0011000);
    let expected = Some(Sequence::new(1, 5, Some(3), Some(4)));
    assert_eq!(result, expected);

    let result = cache.get(0b1100110, 0b0011000);
    let expected = None;
    assert_eq!(result, expected);
  }
}
