#![doc = include_str!("../README.md")]

/// Trait for types that can validate a value of type `T`.
///
/// Implementors should return `true` if validation passes,
/// or `false` if validation fails. The error details are
/// captured in the validation struct itself.
pub trait Validate<T> {
    fn validate(&self, value: &T) -> bool;
}

/// Trait for validation error structs that have no errors.
///
/// This is auto-implemented by the derive macro for generated
/// error structs, allowing easy checking if any validation failed.
pub trait ValidationError {
    /// Returns `true` if there are no validation errors.
    fn is_empty(&self) -> bool;

    /// Returns `true` if there are any validation errors.
    fn has_errors(&self) -> bool {
        !self.is_empty()
    }
}

/// Hidden trait used by derived validation code to pass borrowed values into
/// validator builders.
///
/// Validators that capture the input clone from the borrowed value inside the
/// builder impl. Validators marked with `#[koruma(value, skip_capture)]` on an
/// `Option<T>` value field can ignore the borrowed input and keep their default
/// value instead.
#[doc(hidden)]
pub trait BuilderWithValueRef<T> {
    type Output;

    fn with_value_ref(self, value: &T) -> Self::Output;
}

/// Trait for structs that derive `Koruma` and have a `validate()` method.
///
/// This trait provides an associated type for the validation error struct,
/// which is used by nested validation to properly type the error fields.
///
/// This is auto-implemented by the `#[derive(Koruma)]` macro.
pub trait ValidateExt {
    /// The validation error type for this struct.
    type Error: ValidationError;

    /// Validates the struct and returns the error struct if validation fails.
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Marker trait for newtype structs (single-field wrappers) that derive `Koruma`.
///
/// This trait is auto-implemented by `#[derive(Koruma)]` when `#[koruma(newtype)]`
/// is used at the struct level. It signals that this type is a newtype wrapper
/// and its error type supports transparent `Deref` access.
///
/// When using a newtype as a field in another struct, use `#[koruma(newtype)]`
/// on the field (instead of `#[koruma(nested)]`) to get transparent error access.
pub trait NewtypeValidation: ValidateExt {}

/// Showcase module for validator discovery and registration.
///
/// When the `internal-showcase` feature is enabled, validators decorated with
/// `#[showcase(...)]` attributes are automatically registered for
/// programmatic discovery by showcase consumers (for example, UIs, examples, or tooling).
/// discovery for showcase purposes.
#[cfg(feature = "internal-showcase")]
pub mod showcase {
    /// Localizer callback used by showcased validators to render Fluent messages.
    #[cfg(feature = "fluent")]
    pub type FluentLocalizer<'localizer> = dyn for<'a> FnMut(
            &str,
            &str,
            Option<&std::collections::HashMap<&str, ::es_fluent::FluentValue<'a>>>,
        ) -> String
        + 'localizer;

    /// Trait for validators that can be presented by showcase consumers.
    ///
    /// This trait provides a type-erased interface for validators,
    /// allowing consumers to work with any validator regardless of its
    /// generic type parameters.
    ///
    pub trait DynValidator: Send + Sync {
        /// Check if the validation passed.
        fn is_valid(&self) -> bool;

        /// Get the display string via `Display::to_string()`.
        ///
        /// Showcased validators should implement `Display` when they want a
        /// user-facing message in showcase UIs.
        fn display_string(&self) -> String;

        /// Get the fluent i18n string via `FluentMessage::to_fluent_string_with(...)`
        /// `fluent` feature is enabled for the generated showcase impl.
        ///
        /// Returns the message identifier when no localizer callback is provided.
        #[cfg(feature = "fluent")]
        fn fluent_string_with(&self, localize: &mut FluentLocalizer<'_>) -> String;

        #[cfg(feature = "fluent")]
        fn fluent_string(&self) -> String {
            self.fluent_string_with(&mut |_, id, _| id.to_string())
        }

        #[cfg(not(feature = "fluent"))]
        fn fluent_string(&self) -> String;
    }

    /// The type of input expected by the validator.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum InputType {
        /// Any text input.
        Text,
        /// Numeric input only (integers)
        Numeric,
    }

    koruma_derive::showcase_module_enum!(string, format, numeric, collection, general);

    /// Information about a validator for showcase purposes.
    ///
    /// This struct is registered via `inventory` when a validator uses
    /// `#[showcase(...)]` attributes.
    pub struct ValidatorShowcase {
        /// Human-readable name of the validator
        pub name: &'static str,
        /// Description of what the validator checks
        pub description: &'static str,
        /// The type of input expected by the validator
        pub input_type: InputType,
        /// The module/category this validator belongs to.
        pub module: ValidatorModule,
        /// Factory function that creates a validator from string input.
        /// Returns Ok(validator) on success, or Err(error) if input cannot be parsed.
        pub create_validator: fn(&str) -> ::anyhow::Result<Box<dyn DynValidator>>,
    }

    inventory::collect!(ValidatorShowcase);

    /// Get all registered showcase validators.
    pub fn validators() -> Vec<&'static ValidatorShowcase> {
        inventory::iter::<ValidatorShowcase>().collect()
    }
}
