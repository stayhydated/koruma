#![doc = include_str!("../README.md")]

pub use koruma_core::{NewtypeValidation, Validate, ValidateExt, ValidationError};

#[cfg(feature = "derive")]
pub use koruma_derive::{Koruma, KorumaAllDisplay, validator};

#[cfg(all(feature = "derive", feature = "fluent"))]
pub use koruma_derive::KorumaAllFluent;

#[doc(hidden)]
pub mod __private {
    pub use koruma_core::__private::{BuildValidator, CaptureValueRef};
}

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use koruma_core::showcase;

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use inventory;
