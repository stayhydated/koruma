//! Numeric validation validators.
//!
//! This module contains validators for numeric validation scenarios.

use std::fmt::Display;

/// Trait for numeric types that can be validated for positivity, negativity, and ranges.
///
/// Primitive integers and floats implement this trait out of the box. Custom numeric-like
/// types can implement it directly when they have a well-defined additive zero.
pub trait Numeric: PartialOrd + Copy + Display {
    fn zero() -> Self;
}

macro_rules! impl_numeric {
    ($($ty:ty),* $(,)?) => {
        $(
            impl Numeric for $ty {
                fn zero() -> Self {
                    0 as $ty
                }
            }
        )*
    };
}

impl_numeric!(i8, i16, i32, i64, i128, isize);
impl_numeric!(u8, u16, u32, u64, u128, usize);
impl_numeric!(f32, f64);

mod negative;
mod non_negative;
mod non_positive;
mod positive;
mod range;

pub use negative::NegativeValidation;
pub use non_negative::NonNegativeValidation;
pub use non_positive::NonPositiveValidation;
pub use positive::PositiveValidation;
pub use range::RangeValidation;

#[cfg(feature = "internal-showcase")]
#[doc(hidden)]
#[inline(never)]
pub fn __link_showcase_validators() {
    negative::__koruma_showcase_anchor_negative_validation();
    non_negative::__koruma_showcase_anchor_non_negative_validation();
    non_positive::__koruma_showcase_anchor_non_positive_validation();
    positive::__koruma_showcase_anchor_positive_validation();
    range::__koruma_showcase_anchor_range_validation();
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Formatter};

    use koruma::Validate as _;

    use super::{NegativeValidation, NonNegativeValidation, Numeric, PositiveValidation};

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
    struct OffsetNumber(i32);

    impl Display for OffsetNumber {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Numeric for OffsetNumber {
        fn zero() -> Self {
            Self(10)
        }
    }

    #[test]
    fn numeric_uses_explicit_zero_value() {
        let positive = PositiveValidation::builder()
            .with_value(OffsetNumber(10))
            .build();
        let negative = NegativeValidation::builder()
            .with_value(OffsetNumber(10))
            .build();
        let non_negative = NonNegativeValidation::builder()
            .with_value(OffsetNumber(10))
            .build();

        assert!(positive.validate(&OffsetNumber(11)));
        assert!(!positive.validate(&OffsetNumber(10)));
        assert!(negative.validate(&OffsetNumber(9)));
        assert!(non_negative.validate(&OffsetNumber(10)));
    }
}
