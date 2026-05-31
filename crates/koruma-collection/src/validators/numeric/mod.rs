//! Numeric validation validators.
//!
//! This module contains validators for numeric validation scenarios.

use std::fmt::Display;

/// Trait for numeric types that can be validated for positivity, negativity, and ranges.
///
/// Primitive integers and floats implement this trait out of the box. Custom numeric-like
/// types can implement it directly when they have a well-defined additive zero.
pub trait Numeric: PartialOrd + Display {
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

#[cfg(feature = "rust_decimal")]
impl Numeric for rust_decimal::Decimal {
    fn zero() -> Self {
        rust_decimal::Decimal::ZERO
    }
}

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
    #[cfg(feature = "rust_decimal")]
    use rust_decimal::Decimal;

    #[derive(Clone, Debug, PartialEq, PartialOrd)]
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
        let positive = PositiveValidation::with_value(OffsetNumber(10)).build();
        let negative = NegativeValidation::with_value(OffsetNumber(10)).build();
        let non_negative = NonNegativeValidation::with_value(OffsetNumber(10)).build();

        assert_eq!(OffsetNumber(7).to_string(), "7");
        assert!(positive.validate(&OffsetNumber(11)));
        assert!(!positive.validate(&OffsetNumber(10)));
        assert!(negative.validate(&OffsetNumber(9)));
        assert!(non_negative.validate(&OffsetNumber(10)));
    }

    #[cfg(feature = "rust_decimal")]
    #[test]
    fn decimal_support_uses_decimal_zero() {
        let positive = PositiveValidation::with_value(Decimal::ZERO).build();
        let negative = NegativeValidation::with_value(Decimal::ZERO).build();
        let non_negative = NonNegativeValidation::with_value(Decimal::ZERO).build();

        assert!(positive.validate(&Decimal::new(1, 0)));
        assert!(!positive.validate(&Decimal::ZERO));
        assert!(negative.validate(&Decimal::new(-1, 0)));
        assert!(non_negative.validate(&Decimal::ZERO));
    }
}
