#![doc = include_str!("../README.md")]

pub use koruma_core::{NewtypeValidation, Validate, ValidateExt, ValidationError};

#[cfg(feature = "derive")]
pub use koruma_derive::{Koruma, KorumaAllDisplay, validator};

#[cfg(all(feature = "derive", feature = "fluent"))]
pub use koruma_derive::KorumaAllFluent;

#[doc(hidden)]
pub mod __private {
    pub use koruma_core::__private::{
        BuildValidator, CaptureValueRef, CapturedInputCanBeCloned, KorumaAllDisplayRequiresKoruma,
        KorumaAllFluentRequiresKoruma, KorumaWasDerived, assert_display, assert_newtype_validation,
        assert_validate_ext, assert_validator_ready,
    };

    #[cfg(feature = "fluent")]
    pub use koruma_core::__private::assert_fluent_message;
}

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use koruma_core::showcase;

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use inventory;
