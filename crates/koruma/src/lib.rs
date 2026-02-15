#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod bon {
    pub use bon::__::IsUnset;
    pub use bon::*;
}

pub use koruma_core::{
    BuilderWithValue, NewtypeValidation, Validate, ValidateExt, ValidationError,
};

#[cfg(feature = "derive")]
pub use koruma_derive::{Koruma, KorumaAllDisplay, validator};

#[cfg(all(feature = "derive", feature = "fluent"))]
pub use koruma_derive::KorumaAllFluent;

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use koruma_core::showcase;

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use inventory;
