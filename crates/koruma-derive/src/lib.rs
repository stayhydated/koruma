mod expand;
#[cfg(test)]
mod tests;

use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use syn::{DeriveInput, ItemStruct, parse_macro_input};

#[cfg(feature = "fluent")]
use expand::expand_koruma_all_fluent;
use expand::{expand_koruma, expand_koruma_all_display, expand_validator};

/// Attribute macro for validator structs.
///
/// This automatically:
/// - Adds `#[derive(bon::Builder)]` to the struct
/// - Generates a `with_value` method on the builder that delegates to the field
///   marked with `#[koruma(value)]`
///
/// # Example (non-generic)
///
/// ```ignore
/// #[koruma::validator]
/// #[derive(Clone, Debug, EsFluent)]
/// pub struct NumberRangeValidation {
///     min: i32,
///     max: i32,
///     #[koruma(value)]
///     actual: i32,
/// }
///
/// impl Validate<i32> for NumberRangeValidation {
///     fn validate(&self, value: &i32) -> bool {
///         *value >= self.min && *value <= self.max
///     }
/// }
/// ```
///
/// # Example (generic)
///
/// For generic validators, use a blanket impl with trait bounds:
///
/// ```ignore
/// #[koruma::validator]
/// #[derive(Clone, Debug, EsFluent)]
/// pub struct RangeValidation<T> {
///     min: T,
///     max: T,
///     #[koruma(value)]
///     actual: T,
/// }
///
/// impl<T: PartialOrd + Clone> Validate<T> for RangeValidation<T> {
///     fn validate(&self, value: &T) -> bool {
///         *value >= self.min && *value <= self.max
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn validator(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Ensure no arguments
    if !attr.is_empty() {
        let attr2 = proc_macro2::TokenStream::from(attr);
        abort!(attr2, "koruma::validator does not accept arguments");
    }

    let input = parse_macro_input!(item as ItemStruct);

    match expand_validator(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}

/// Derive macro for generating validation error structs and validate methods.
///
/// # Example
///
/// ```ignore
/// #[derive(Koruma)]
/// struct Item {
///     #[koruma(NumberRangeValidation(min = 0, max = 100))]
///     age: i32,
///
///     #[koruma(StringLengthValidation(min = 1, max = 50))]
///     name: String,
///
///     // No #[koruma(...)] attribute means field is not validated
///     internal_id: u64,
/// }
/// ```
///
/// This generates:
/// - `ItemValidationError` struct with `Option<ValidatorType>` for each validated field
/// - Getter methods returning `Option<&ValidatorType>` for each field
/// - `validate(&self) -> Result<(), ItemValidationError>` method on `Item`
///
/// The macro always generates `.with_value(self.field.clone())` for validators.
#[proc_macro_error]
#[proc_macro_derive(Koruma, attributes(koruma))]
pub fn derive_koruma(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_koruma(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}

/// Derive macro for implementing `Display` on the `all()` validator enums.
///
/// Place this alongside `#[derive(Koruma)]` to generate `Display` implementations
/// for the `{Struct}{Field}KorumaValidator` enums returned by the `all()` method.
/// Each variant delegates to its inner validator's `Display` implementation.
///
/// # Example
///
/// ```ignore
/// use koruma::{Koruma, KorumaAllDisplay};
///
/// #[derive(Koruma, KorumaAllDisplay)]
/// pub struct Product {
///     #[koruma(LenValidation::<_>(min = 5, max = 20), PrefixValidation<_>(prefix = "SKU-".to_string()))]
///     pub sku: String,
/// }
///
/// // Now you can use Display on all() results:
/// // for err in errors.sku().all() {
/// //     println!("{}", err);  // Uses Display
/// // }
/// ```
#[proc_macro_error]
#[proc_macro_derive(KorumaAllDisplay, attributes(koruma))]
pub fn derive_koruma_all_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_koruma_all_display(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}

/// Derive macro for implementing `ToFluentString` on the `all()` validator enums.
///
/// Place this alongside `#[derive(Koruma)]` to generate `ToFluentString` implementations
/// for the `{Struct}{Field}KorumaValidator` enums returned by the `all()` method.
/// Each variant delegates to its inner validator's `ToFluentString` implementation.
///
/// Requires the `fluent` feature to be enabled.
///
/// # Example
///
/// ```ignore
/// use koruma::{Koruma, KorumaAllDisplay, KorumaAllFluent};
///
/// #[derive(Koruma, KorumaAllDisplay, KorumaAllFluent)]
/// pub struct Product {
///     #[koruma(LenValidation::<_>(min = 5, max = 20), PrefixValidation<_>(prefix = "SKU-".to_string()))]
///     pub sku: String,
/// }
///
/// // Now you can use ToFluentString on all() results:
/// // for err in errors.sku().all() {
/// //     println!("{}", err.to_fluent_string());  // Uses i18n
/// // }
/// ```
#[cfg(feature = "fluent")]
#[proc_macro_error]
#[proc_macro_derive(KorumaAllFluent, attributes(koruma))]
pub fn derive_koruma_all_fluent(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_koruma_all_fluent(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}
