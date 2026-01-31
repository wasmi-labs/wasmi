use super::arena::*;

impl ArenaKey for usize {
    fn into_usize(self) -> usize {
        self
    }

    fn from_usize(value: usize) -> Option<Self> {
        Some(value)
    }
}

const TEST_ENTITIES: &[&str] = &["a", "b", "c", "d"];

mod arena {
    use super::*;

    fn alloc_arena(entities: &[&'static str]) -> Arena<usize, &'static str> {
        let mut arena = <Arena<usize, &'static str>>::new();
        // Check that the given arena is actually empty.
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        // Fill arena and check invariants while doing so.
        for idx in 0..entities.len() {
            assert!(arena.get(idx).is_err());
        }
        for (n, str) in entities.iter().enumerate() {
            assert_eq!(arena.alloc(str).ok(), Some(n));
        }
        // Check state of filled arena.
        assert_eq!(arena.len(), entities.len());
        assert!(!arena.is_empty());
        for (n, str) in entities.iter().enumerate() {
            assert_eq!(arena.get(n).ok(), Some(str));
            assert_eq!(&arena[n], str);
        }
        assert!(arena.get(arena.len()).is_err());
        // Return filled arena.
        arena
    }

    #[test]
    fn alloc_works() {
        alloc_arena(TEST_ENTITIES);
    }

    #[test]
    fn clear_works() {
        let mut arena = alloc_arena(TEST_ENTITIES);
        // Clear the arena and check if all elements are removed.
        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        for idx in 0..arena.len() {
            assert!(arena.get(idx).is_err());
        }
        assert!(arena.get(arena.len()).is_err());
    }

    #[test]
    fn iter_works() {
        let arena = alloc_arena(TEST_ENTITIES);
        assert!(arena.iter().eq(TEST_ENTITIES.iter().enumerate()));
    }

    #[test]
    fn from_iter_works() {
        let expected = alloc_arena(TEST_ENTITIES);
        let actual = TEST_ENTITIES.iter().copied().collect::<Arena<_, _>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn duplicates_work() {
        let mut arena = alloc_arena(TEST_ENTITIES);
        // Re-inserting the same entities into the filled arena will
        // result in new and unique indices since the standard arena
        // type does not deduplicate its entities.
        let previous_len = arena.len();
        for (idx, str) in TEST_ENTITIES.iter().enumerate() {
            let offset = previous_len + idx;
            assert_eq!(arena.alloc(str).ok(), Some(offset));
            assert_eq!(arena.get(offset).ok(), Some(str));
        }
        // Assert that the arena actually did increase in size since
        // there is no deduplication of equal entities.
        assert_eq!(arena.len(), previous_len + TEST_ENTITIES.len());
    }
}

mod dedup_arena {
    use super::*;

    fn alloc_dedup_arena(entities: &[&'static str]) -> DedupArena<usize, &'static str> {
        let mut arena = <DedupArena<usize, &'static str>>::new();
        // Check that the given arena is actually empty.
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        // Fill arena and check invariants while doing so.
        for idx in 0..entities.len() {
            assert!(arena.get(idx).is_none());
        }
        for (n, str) in entities.iter().enumerate() {
            assert_eq!(arena.alloc(str), n);
        }
        // Check state of filled arena.
        assert_eq!(arena.len(), entities.len());
        assert!(!arena.is_empty());
        for (n, str) in entities.iter().enumerate() {
            assert_eq!(arena.get(n), Some(str));
            assert_eq!(&arena[n], str);
        }
        assert_eq!(arena.get(arena.len()), None);
        // Return filled arena.
        arena
    }

    #[test]
    fn alloc_works() {
        alloc_dedup_arena(TEST_ENTITIES);
    }

    #[test]
    fn clear_works() {
        let mut arena = alloc_dedup_arena(TEST_ENTITIES);
        // Clear the arena and check if all elements are removed.
        arena.clear();
        assert_eq!(arena.len(), 0);
        assert!(arena.is_empty());
        for idx in 0..arena.len() {
            assert_eq!(arena.get(idx), None);
        }
        assert_eq!(arena.get(arena.len()), None);
    }

    #[test]
    fn iter_works() {
        let arena = alloc_dedup_arena(TEST_ENTITIES);
        assert!(arena.iter().eq(TEST_ENTITIES.iter().enumerate()));
    }

    #[test]
    fn from_iter_works() {
        let expected = alloc_dedup_arena(TEST_ENTITIES);
        let actual = TEST_ENTITIES.iter().copied().collect::<DedupArena<_, _>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn duplicates_work() {
        let mut arena = alloc_dedup_arena(TEST_ENTITIES);
        // Re-inserting the same entities into the filled arena will
        // yield back the same indices as their already allocated entities.
        for (idx, str) in TEST_ENTITIES.iter().enumerate() {
            assert_eq!(arena.alloc(str), idx);
            assert_eq!(arena.get(idx), Some(str));
        }
        // Assert that the deduplicating arena did not increase in size.
        assert_eq!(arena.len(), TEST_ENTITIES.len());
    }
}
