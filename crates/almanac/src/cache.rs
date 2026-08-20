//! A bounded least-recently-used cache.
//!
//! Small and purpose-built rather than a dependency: the only requirements are a
//! size bound and whole-cache invalidation when settings change, and a
//! generation counter gives the latter in constant time instead of walking every
//! entry to decide what is still valid.

use std::collections::HashMap;
use std::hash::Hash;

pub struct Lru<K, V> {
    capacity: usize,
    generation: u64,
    entries: HashMap<K, (u64, u64, V)>,
    clock: u64,
}

impl<K: Eq + Hash + Clone, V: Clone> Lru<K, V> {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "a zero capacity cache would never hit");
        Self {
            capacity,
            generation: 0,
            entries: HashMap::with_capacity(capacity),
            clock: 0,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        let generation = self.generation;
        self.clock += 1;
        let clock = self.clock;

        let entry = self.entries.get_mut(key)?;
        if entry.0 != generation {
            // Stale by generation. Removing it here keeps the map from holding
            // dead entries until eviction reaches them.
            self.entries.remove(key);
            return None;
        }
        entry.1 = clock;
        Some(entry.2.clone())
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.clock += 1;

        if self.entries.len() >= self.capacity && !self.entries.contains_key(&key) {
            self.evict_oldest();
        }
        self.entries
            .insert(key, (self.generation, self.clock, value));
    }

    /// Invalidates everything. Called when a setting that every cached value
    /// depends on changes: ayanamsa, node type, or location.
    pub fn invalidate_all(&mut self) {
        self.generation += 1;
    }

    /// Live entry count. Only the tests need this; the running app never asks
    /// how full the cache is.
    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries
            .values()
            .filter(|(generation, _, _)| *generation == self.generation)
            .count()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn evict_oldest(&mut self) {
        // Prefer evicting an entry from a superseded generation: it can never be
        // read again, so it is free to drop.
        let generation = self.generation;
        let victim = self
            .entries
            .iter()
            .min_by_key(|(_, (entry_generation, used, _))| (*entry_generation == generation, *used))
            .map(|(key, _)| key.clone());

        if let Some(key) = victim {
            self.entries.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_what_was_stored() {
        let mut cache: Lru<u32, &str> = Lru::new(4);
        cache.insert(1, "one");
        assert_eq!(cache.get(&1), Some("one"));
        assert_eq!(cache.get(&2), None);
    }

    #[test]
    fn evicts_the_least_recently_used() {
        let mut cache: Lru<u32, u32> = Lru::new(3);
        for key in 1..=3 {
            cache.insert(key, key);
        }
        // Touch 1 so 2 becomes the coldest.
        assert_eq!(cache.get(&1), Some(1));
        cache.insert(4, 4);

        assert_eq!(
            cache.get(&2),
            None,
            "coldest entry should have been evicted"
        );
        assert_eq!(cache.get(&1), Some(1));
        assert_eq!(cache.get(&3), Some(3));
        assert_eq!(cache.get(&4), Some(4));
    }

    #[test]
    fn never_exceeds_capacity() {
        let mut cache: Lru<u32, u32> = Lru::new(8);
        for key in 0..1000 {
            cache.insert(key, key);
        }
        assert!(cache.entries.len() <= 8, "held {}", cache.entries.len());
    }

    #[test]
    fn invalidation_hides_every_previous_entry() {
        let mut cache: Lru<u32, u32> = Lru::new(4);
        cache.insert(1, 10);
        cache.insert(2, 20);
        cache.invalidate_all();

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
        assert!(cache.is_empty());

        cache.insert(1, 11);
        assert_eq!(cache.get(&1), Some(11));
    }

    #[test]
    fn stale_entries_are_evicted_before_live_ones() {
        let mut cache: Lru<u32, u32> = Lru::new(2);
        cache.insert(1, 1);
        cache.invalidate_all();
        cache.insert(2, 2);
        // Inserting a third entry must drop the stale 1, not the live 2.
        cache.insert(3, 3);

        assert_eq!(cache.get(&2), Some(2));
        assert_eq!(cache.get(&3), Some(3));
    }

    #[test]
    fn reinsert_updates_without_growing() {
        let mut cache: Lru<u32, u32> = Lru::new(2);
        cache.insert(1, 1);
        cache.insert(1, 99);
        assert_eq!(cache.get(&1), Some(99));
        assert_eq!(cache.len(), 1);
    }
}
