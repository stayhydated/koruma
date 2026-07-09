//! String validation validators.
//!
//! This module contains validators for string-based validation scenarios.

/// Trait alias for types that can be treated as string references.
pub trait StringLike: AsRef<str> {}

impl<T: AsRef<str>> StringLike for T {}

mod alphanumeric;
mod ascii;
mod canonical_form;
mod contains;
mod matches;
#[cfg(feature = "regex")]
mod pattern;
mod prefix;
mod suffix;

pub use alphanumeric::AlphanumericValidation;
pub use ascii::AsciiValidation;
pub use canonical_form::CanonicalFormValidation;
pub use contains::ContainsValidation;
pub use matches::MatchesValidation;
#[cfg(feature = "regex")]
pub use pattern::PatternValidation;
pub use prefix::PrefixValidation;
pub use suffix::SuffixValidation;

#[cfg(feature = "internal-showcase")]
#[doc(hidden)]
#[inline(never)]
pub fn __link_showcase_validators() {
    alphanumeric::__koruma_showcase_anchor_alphanumeric_validation();
    ascii::__koruma_showcase_anchor_ascii_validation();
    canonical_form::__koruma_showcase_anchor_canonical_form_validation();
    contains::__koruma_showcase_anchor_contains_validation();
    matches::__koruma_showcase_anchor_matches_validation();

    #[cfg(feature = "regex")]
    pattern::__koruma_showcase_anchor_pattern_validation();

    prefix::__koruma_showcase_anchor_prefix_validation();
    suffix::__koruma_showcase_anchor_suffix_validation();
}
