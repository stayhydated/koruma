//! Collection validation validators.
//!
//! This module contains validators for collection-based validation scenarios.

/// Trait for types that have a measurable length.
pub trait HasLen {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// Implementations for std collections
impl<T> HasLen for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for std::collections::VecDeque<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<K, V> HasLen for std::collections::HashMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<K, V> HasLen for std::collections::BTreeMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for std::collections::HashSet<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> HasLen for std::collections::BTreeSet<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl HasLen for String {
    fn len(&self) -> usize {
        self.chars().count()
    }
}

impl HasLen for str {
    fn len(&self) -> usize {
        self.chars().count()
    }
}

impl<T> HasLen for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T, const N: usize> HasLen for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

#[cfg(feature = "smallvec")]
impl<T, const N: usize> HasLen for smallvec::SmallVec<[T; N]> {
    fn len(&self) -> usize {
        self.len()
    }
}

mod len;
mod non_empty;

pub use len::LenValidation;
pub use non_empty::NonEmptyValidation;

#[cfg(feature = "internal-showcase")]
#[doc(hidden)]
#[inline(never)]
pub fn __link_showcase_validators() {
    len::__koruma_showcase_anchor_len_validation();
    non_empty::__koruma_showcase_anchor_non_empty_validation();
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

    use super::HasLen;

    #[test]
    fn std_collection_lengths_use_runtime_length() {
        let vec = vec![1_u8, 2, 3];
        assert_eq!(HasLen::len(&vec), 3);

        let mut deque = VecDeque::new();
        deque.push_back(1_u8);
        deque.push_back(2);
        assert_eq!(HasLen::len(&deque), 2);

        let hash_map = HashMap::from([("one", 1), ("two", 2)]);
        assert_eq!(HasLen::len(&hash_map), 2);

        let btree_map = BTreeMap::from([("one", 1), ("two", 2), ("three", 3)]);
        assert_eq!(HasLen::len(&btree_map), 3);

        let hash_set = HashSet::from([1_u8, 2, 3, 4]);
        assert_eq!(HasLen::len(&hash_set), 4);

        let btree_set = BTreeSet::from([1_u8, 2, 3, 4, 5]);
        assert_eq!(HasLen::len(&btree_set), 5);
    }

    #[test]
    fn array_len_matches_size() {
        let values = [1_u8, 2, 3];
        assert_eq!(HasLen::len(&values), 3);
    }

    #[test]
    fn slice_len_uses_runtime_length() {
        let values = [1_u8, 2, 3, 4];
        assert_eq!(HasLen::len(&values[..2]), 2);
        assert!(!HasLen::is_empty(&values[..2]));
        assert!(HasLen::is_empty(&values[..0]));
    }

    #[test]
    fn string_len_counts_unicode_scalar_values() {
        assert_eq!(HasLen::len("a💀é"), 3);
        assert_eq!(HasLen::len(&"a💀é".to_string()), 3);
    }

    #[cfg(feature = "smallvec")]
    #[test]
    fn smallvec_len_uses_runtime_length() {
        let mut values = smallvec::SmallVec::<[u8; 4]>::new();
        values.push(10);
        values.push(20);

        assert_eq!(HasLen::len(&values), 2);
    }
}
