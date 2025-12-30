/// Trait for types that can validate a value of type `T`.
///
/// Implementors should return `Ok(())` if validation passes,
/// or `Err(())` if validation fails. The error details are
/// captured in the validation struct itself (via `ToFluentString`).
pub trait Validate<T> {
    fn validate(&self, value: &T) -> Result<(), ()>;
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
