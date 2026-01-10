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

/// Trait for validator builders that can receive the value being validated.
///
/// This is auto-implemented by `#[koruma::validator]` to delegate to the
/// field marked with `#[koruma(value)]`.
pub trait BuilderWithValue<T> {
    fn with_value(self, value: T) -> Self;
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
/// When the `showcase` feature is enabled, validators decorated with
/// `#[showcase(...)]` attributes are automatically registered for
/// programmatic discovery by showcase consumers (for example, UIs, examples, or tooling).
/// discovery for showcase purposes.
#[cfg(feature = "showcase")]
pub mod showcase {
    /// Trait for validators that can be presented by showcase consumers.
    ///
    /// This trait provides a type-erased interface for validators,
    /// allowing consumers to work with any validator regardless of its
    /// generic type parameters.
    ///
    /// Methods are always present but may return placeholder values
    /// when the corresponding feature is not enabled.
    pub trait DynValidator: Send + Sync {
        /// Check if the validation passed.
        fn is_valid(&self) -> bool;

        /// Get the display string (via `to_string()` when `fmt` feature is enabled).
        /// Returns "(fmt feature required)" if fmt is not enabled.
        fn display_string(&self) -> String;

        /// Get the fluent i18n string (via `to_fluent_string()` when `fluent` feature is enabled).
        /// Returns "(fluent feature required)" if fluent is not enabled.
        fn fluent_string(&self) -> String;
    }

    /// The type of input expected by the validator.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub enum InputType {
        /// Any text input (default)
        #[default]
        Text,
        /// Numeric input only (integers)
        Numeric,
    }

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
        /// Factory function that creates a validator from string input.
        pub create_validator: fn(&str) -> Box<dyn DynValidator>,
    }

    inventory::collect!(ValidatorShowcase);

    /// Get all registered showcase validators.
    pub fn validators() -> Vec<&'static ValidatorShowcase> {
        inventory::iter::<ValidatorShowcase>().collect()
    }
}
