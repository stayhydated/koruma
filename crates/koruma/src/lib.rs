#![doc = include_str!("../README.md")]

pub use koruma_core::{NewtypeValidation, Validate, ValidateExt, ValidationError};

#[cfg(feature = "derive")]
pub use koruma_derive::{Koruma, KorumaAllDisplay, validator};

#[cfg(all(feature = "derive", feature = "fluent"))]
pub use koruma_derive::KorumaAllFluent;

#[doc(hidden)]
pub mod __private {
    pub use koruma_core::__private::{
        BuildValidator, CaptureValueRef, CapturedInputCanBeCloned, EachCollectionRef,
        KorumaAllDisplayRequiresKoruma, KorumaAllFluentRequiresKoruma, KorumaWasDerived,
        OptionalEachCollectionRef, assert_display, assert_each_collection_ref,
        assert_element_display, assert_element_validator_ready, assert_field_display,
        assert_field_validator_ready, assert_nested_validation_ready, assert_newtype_error_display,
        assert_newtype_field_ready, assert_newtype_validation, assert_optional_each_collection_ref,
        assert_validate_ext, assert_validator_ready,
    };

    #[cfg(feature = "fluent")]
    pub use koruma_core::__private::{
        assert_element_fluent_message, assert_error_fluent_message, assert_field_fluent_message,
        assert_fluent_message, assert_newtype_error_fluent_message,
    };
}

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use koruma_core::showcase;

#[doc(hidden)]
#[cfg(feature = "internal-showcase")]
pub use inventory;
