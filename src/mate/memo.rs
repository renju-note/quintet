use std::collections::HashMap;

/// What the solvers remember between searches, keyed by
/// [`State::zobrist_hash`](crate::mate::State::zobrist_hash).
///
/// Everything stored here stays true for as long as the solver lives: a
/// proof, a disproof and a deadend are properties of the position, the turn,
/// the attacker and the remaining limit, all of which are in the key.
/// Proof and disproof *numbers* short of 0 are estimates rather than facts,
/// but they are lower bounds either way, so reading one a previous search
/// left behind can only save work.
///
/// So a generation is not a validity stamp — it is what decides whose turn it
/// is to be forgotten. A solver that is reused would otherwise grow without
/// bound: one search is held down by its node budget, a hundred searches are
/// held down by nothing.
///
/// [`advance_generation`](Self::advance_generation), called at the start of a
/// search, drops what searches *before the last one* left behind, once the
/// memo has grown past `carry_capacity`. The search that just finished is
/// always kept, so asking the same question again — which is what reuse
/// really buys — stays nearly free. The memo settles at about two searches'
/// worth however many searches are asked of it.
///
/// Dropping only *between* searches is deliberate: `carry_capacity` is not a
/// hard cap. A df-pn node only makes progress once its child's numbers are
/// in the table, so a memo that refused or evicted an entry mid-search would
/// send `expand_attacks` round its loop again on the same child — a spin
/// with an unlimited budget. Within one search the node budget is the bound.
pub struct Memo<V> {
    entries: HashMap<u64, Entry<V>>,
    generation: u32,
    carry_capacity: usize,
}

struct Entry<V> {
    value: V,
    generation: u32,
}

impl<V> Memo<V> {
    pub fn new(carry_capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            generation: 0,
            carry_capacity,
        }
    }

    /// Forgets everything.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.generation = 0;
    }

    /// Opens a new generation, first dropping everything but the previous
    /// one if the memo has outgrown `carry_capacity`. Call it once per
    /// search, not per node.
    pub fn advance_generation(&mut self) {
        let previous = self.generation;
        self.generation = self.generation.wrapping_add(1);
        if self.entries.len() > self.carry_capacity {
            self.entries.retain(|_, e| e.generation == previous);
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, key: u64) -> Option<&V> {
        self.entries.get(&key).map(|e| &e.value)
    }

    pub fn contains(&self, key: u64) -> bool {
        self.entries.contains_key(&key)
    }

    pub fn insert(&mut self, key: u64, value: V) {
        self.entries.insert(
            key,
            Entry {
                value,
                generation: self.generation,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keeps_the_previous_generation_and_drops_the_rest() {
        let mut memo = Memo::new(2);
        memo.insert(1, "oldest");
        memo.advance_generation();
        memo.insert(2, "previous");
        memo.advance_generation();

        // Two entries is not over the carry capacity, so nothing was dropped.
        assert_eq!(memo.len(), 2);
        assert_eq!(memo.get(1), Some(&"oldest"));

        memo.insert(3, "current");
        // Still readable while they are there: a generation is not validity.
        assert_eq!(memo.get(1), Some(&"oldest"));
        assert_eq!(memo.len(), 3);

        // Over it now, so the next search drops all but generation 1.
        memo.advance_generation();
        assert_eq!(memo.len(), 1);
        assert_eq!(memo.get(3), Some(&"current"));
        assert_eq!(memo.get(2), None);
        assert_eq!(memo.get(1), None);
    }

    #[test]
    fn test_reinserting_moves_an_entry_to_the_current_generation() {
        let mut memo = Memo::new(1);
        memo.insert(1, "old");
        memo.advance_generation();
        memo.insert(1, "refreshed");
        memo.insert(2, "other");
        memo.advance_generation();
        memo.insert(3, "current");
        memo.advance_generation();

        // 1 was refreshed in generation 1 and 2 was made there, so both
        // survived into 2 and were dropped on the way into 3 with 3 kept.
        assert_eq!(memo.get(3), Some(&"current"));
        assert_eq!(memo.get(1), None);
        assert_eq!(memo.get(2), None);
    }

    #[test]
    fn test_clear() {
        let mut memo = Memo::new(10);
        memo.insert(1, ());
        assert!(memo.contains(1));
        memo.clear();
        assert!(!memo.contains(1));
        assert_eq!(memo.len(), 0);
    }
}
