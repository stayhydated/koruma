//! Length validation for collections.
//!
//! This module provides:
//! - `HasLen` trait for types that have a measurable length
//! - `LenValidation` validator with min/max bounds
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::len::{HasLen, LenValidation};
//!
//! #[derive(Koruma)]
//! struct Order {
//!     #[koruma(LenValidation<_>(min = 1, max = 5))]
//!     items: Vec<String>,
//! }
//! ```

use koruma::{KorumaResult, Validate, validator};

/// Trait for types that have a measurable length.
///
/// Implemented for common std collections. Users can implement
/// this for custom collection types.
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
        self.len()
    }
}

impl HasLen for str {
    fn len(&self) -> usize {
        self.len()
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

/// Validates that a collection's length is within the specified bounds.
///
/// Works with any type that implements `HasLen + Clone`.
#[validator]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct LenValidation<T: HasLen> {
    /// Minimum allowed length (inclusive)
    pub min: usize,
    /// Maximum allowed length (inclusive)
    pub max: usize,
    /// The collection being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.len())))]
    pub actual: T,
}

impl<T: HasLen + Clone> Validate<T> for LenValidation<T> {
    fn validate(&self, value: &T) -> KorumaResult {
        let len = value.len();
        if len < self.min || len > self.max {
            Err(())
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: HasLen + Clone> std::fmt::Display for LenValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "length {} is not within bounds [{}, {}]",
            self.actual.len(),
            self.min,
            self.max
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_valid_length() {
        let items = vec![1, 2, 3];
        let validator = LenValidation::builder()
            .min(1)
            .max(5)
            .actual(items.clone())
            .build();
        assert!(validator.validate(&items).is_ok());
    }

    #[test]
    fn test_vec_too_short() {
        let items: Vec<i32> = vec![];
        let validator = LenValidation::builder()
            .min(1)
            .max(5)
            .actual(items.clone())
            .build();
        assert!(validator.validate(&items).is_err());
    }

    #[test]
    fn test_vec_too_long() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let validator = LenValidation::builder()
            .min(1)
            .max(5)
            .actual(items.clone())
            .build();
        assert!(validator.validate(&items).is_err());
    }

    #[test]
    fn test_vec_at_min_boundary() {
        let items = vec![1];
        let validator = LenValidation::builder()
            .min(1)
            .max(5)
            .actual(items.clone())
            .build();
        assert!(validator.validate(&items).is_ok());
    }

    #[test]
    fn test_vec_at_max_boundary() {
        let items = vec![1, 2, 3, 4, 5];
        let validator = LenValidation::builder()
            .min(1)
            .max(5)
            .actual(items.clone())
            .build();
        assert!(validator.validate(&items).is_ok());
    }

    #[test]
    fn test_string_valid_length() {
        let s = String::from("hello");
        let validator = LenValidation::builder()
            .min(1)
            .max(10)
            .actual(s.clone())
            .build();
        assert!(validator.validate(&s).is_ok());
    }

    #[test]
    fn test_string_empty_invalid() {
        let s = String::new();
        let validator = LenValidation::builder()
            .min(1)
            .max(10)
            .actual(s.clone())
            .build();
        assert!(validator.validate(&s).is_err());
    }

    #[test]
    fn test_string_too_long() {
        let s = String::from("this is a very long string");
        let validator = LenValidation::builder()
            .min(1)
            .max(10)
            .actual(s.clone())
            .build();
        assert!(validator.validate(&s).is_err());
    }

    #[test]
    fn test_hashset_valid_length() {
        use std::collections::HashSet;
        let set: HashSet<i32> = [1, 2, 3].into_iter().collect();
        let validator = LenValidation::builder()
            .min(2)
            .max(5)
            .actual(set.clone())
            .build();
        assert!(validator.validate(&set).is_ok());
    }

    #[test]
    fn test_hashmap_valid_length() {
        use std::collections::HashMap;
        let map: HashMap<&str, i32> = [("a", 1), ("b", 2)].into_iter().collect();
        let validator = LenValidation::builder()
            .min(1)
            .max(3)
            .actual(map.clone())
            .build();
        assert!(validator.validate(&map).is_ok());
    }

    #[test]
    fn test_with_value_method() {
        let items = vec![1, 2, 3];
        let validator = LenValidation::builder()
            .min(1)
            .max(5)
            .with_value(items.clone())
            .build();
        assert!(validator.validate(&items).is_ok());
    }

    #[cfg(feature = "fmt")]
    #[test]
    fn test_display_error_message() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let validator = LenValidation::builder().min(1).max(5).actual(items).build();
        let msg = format!("{}", validator);
        assert_eq!(msg, "length 6 is not within bounds [1, 5]");
    }

    #[test]
    fn test_clone() {
        let items = vec![1, 2, 3];
        let validator = LenValidation::builder()
            .min(1)
            .max(5)
            .actual(items.clone())
            .build();
        let cloned = validator.clone();
        assert_eq!(cloned.min, validator.min);
        assert_eq!(cloned.max, validator.max);
        assert_eq!(cloned.actual, validator.actual);
    }

    mod koruma_derive_tests {
        use super::*;
        use koruma::Koruma;

        #[derive(Koruma)]
        struct Order {
            // <_> infers the full field type: Vec<String>
            #[koruma(LenValidation<_>(min = 1, max = 5))]
            items: Vec<String>,
        }

        #[test]
        fn test_order_valid() {
            let order = Order {
                items: vec!["apple".to_string(), "banana".to_string()],
            };
            assert!(order.validate().is_ok());
        }

        #[test]
        fn test_order_empty_items_invalid() {
            let order = Order { items: vec![] };
            let result = order.validate();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.items.len_validation.is_some());
        }

        #[test]
        fn test_order_too_many_items_invalid() {
            let order = Order {
                items: vec![
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "d".to_string(),
                    "e".to_string(),
                    "f".to_string(),
                ],
            };
            let result = order.validate();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.items.len_validation.is_some());
        }

        #[derive(Koruma)]
        struct UserProfile {
            // <_> infers the full field type: String
            #[koruma(LenValidation<_>(min = 1, max = 50))]
            username: String,

            #[koruma(LenValidation<_>(min = 0, max = 500))]
            bio: String,
        }

        #[test]
        fn test_user_profile_valid() {
            let profile = UserProfile {
                username: "johndoe".to_string(),
                bio: "Hello, I'm John!".to_string(),
            };
            assert!(profile.validate().is_ok());
        }

        #[test]
        fn test_user_profile_empty_username_invalid() {
            let profile = UserProfile {
                username: String::new(),
                bio: "Some bio".to_string(),
            };
            let result = profile.validate();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.username.len_validation.is_some());
            assert!(err.bio.len_validation.is_none());
        }

        #[test]
        fn test_user_profile_username_too_long_invalid() {
            let profile = UserProfile {
                username: "a".repeat(51),
                bio: String::new(),
            };
            let result = profile.validate();
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(err.username.len_validation.is_some());
        }

        #[derive(Koruma)]
        struct TaggedPost {
            // <_> infers the full field type: HashSet<String>
            #[koruma(LenValidation<_>(min = 1, max = 10))]
            tags: std::collections::HashSet<String>,
        }

        #[test]
        fn test_tagged_post_valid() {
            let post = TaggedPost {
                tags: ["rust", "programming"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            };
            assert!(post.validate().is_ok());
        }

        #[test]
        fn test_tagged_post_no_tags_invalid() {
            let post = TaggedPost {
                tags: std::collections::HashSet::new(),
            };
            let result = post.validate();
            assert!(result.is_err());
        }
    }
}
