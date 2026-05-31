#![doc = include_str!("../README.md")]

mod expand;
#[cfg(feature = "internal-showcase")]
mod showcase_modules;
#[cfg(test)]
mod tests;

use proc_macro::TokenStream;
use proc_macro_error2::{abort, proc_macro_error};
use syn::spanned::Spanned;
use syn::{DeriveInput, Fields, Item, parse_macro_input};

#[cfg(feature = "fluent")]
use expand::expand_koruma_all_fluent;
use expand::{expand_koruma, expand_koruma_all_display, expand_validator};
#[cfg(feature = "internal-showcase")]
use showcase_modules::{expand_showcase_module_enum_macro, expand_showcase_modules_macro};

/// Attribute macro for validator structs.
///
/// This automatically:
/// - Generates Koruma-owned builder plumbing for the struct
/// - Generates direct builder entrypoints on the validator type for each
///   configurable field, such as `RangeValidation::min(value)`
/// - Generates `with_value` methods that delegate to the field marked with
///   `#[koruma(value)]`
/// - Generates a getter on the validator type with the same name as the
///   `#[koruma(value)]` field
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

    let item = parse_macro_input!(item as Item);
    let input = match item {
        Item::Struct(input) => input,
        other => {
            let err = syn::Error::new(
                other.span(),
                "koruma::validator can only be applied to structs",
            );
            return TokenStream::from(err.to_compile_error());
        },
    };

    match &input.fields {
        Fields::Named(fields) if !fields.named.is_empty() => {},
        Fields::Named(_) | Fields::Unit => {
            let err = syn::Error::new(
                input.ident.span(),
                "koruma::validator requires at least one field",
            );
            return TokenStream::from(err.to_compile_error());
        },
        Fields::Unnamed(_) => {
            let err = syn::Error::new(
                input.ident.span(),
                "koruma::validator only supports structs with named fields",
            );
            return TokenStream::from(err.to_compile_error());
        },
    }

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
///     #[koruma(NumberRangeValidation::min(0).max(100))]
///     age: i32,
///
///     #[koruma(StringLengthValidation::min(1).max(50))]
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
/// The macro captures validator values through a hidden borrowed builder hook.
/// Validators that keep the default `#[koruma(value)]` behavior still clone the
/// input into the error value; validators marked with
/// `#[koruma(value(capture = skip))]` on an `Option<T>` value field can opt out
/// when they do not need to store the validated value.
#[proc_macro_error]
#[proc_macro_derive(Koruma, attributes(koruma))]
pub fn derive_koruma(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_koruma(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}

/// Derive macro for implementing `Display` on the borrowed `all()` validator enums.
///
/// Place this alongside `#[derive(Koruma)]` to generate `Display` implementations
/// for the `{Struct}{Field}KorumaValidatorRef` enums returned by the `all()` method.
/// Each variant delegates to its inner validator's `Display` implementation.
///
/// # Example
///
/// ```ignore
/// use koruma::{Koruma, KorumaAllDisplay};
///
/// #[derive(Koruma, KorumaAllDisplay)]
/// pub struct Product {
///     #[koruma(LenValidation::<_>::min(5).max(20), PrefixValidation::<_>::prefix("SKU-".to_string()))]
///     pub sku: String,
/// }
///
/// Now you can use Display on all() results:
/// for err in errors.sku().all() {
///     println!("{}", err);  // Uses Display
/// }
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

/// Derive macro for implementing `FluentMessage` on the borrowed `all()` validator enums.
///
/// Place this alongside `#[derive(Koruma)]` to generate `FluentMessage` implementations
/// for the `{Struct}{Field}KorumaValidatorRef` enums returned by the `all()` method.
/// Each variant delegates to its inner validator's `FluentMessage` implementation.
///
/// Requires the `fluent` feature to be enabled.
///
/// # Example
///
/// ```ignore
/// use koruma::{Koruma, KorumaAllFluent};
///
/// #[derive(Koruma, KorumaAllFluent)]
/// pub struct Product {
///     #[koruma(LenValidation::<_>::min(5).max(20), PrefixValidation::<_>::prefix("SKU-".to_string()))]
///     pub sku: String,
/// }
///
/// // Now you can use `FluentMessage` on all() results:
/// for err in errors.sku().all() {
///     // Use your active i18n context/localizer to render this error:
///     // println!("{}", i18n_context.localize_message(err));
/// }
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

/// Internal helper macro for generating showcase module declarations and linker functions.
#[cfg(feature = "internal-showcase")]
#[proc_macro_error]
#[proc_macro]
pub fn showcase_modules(input: TokenStream) -> TokenStream {
    match expand_showcase_modules_macro(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}

/// Internal helper macro for generating `ValidatorModule`.
#[cfg(feature = "internal-showcase")]
#[proc_macro_error]
#[proc_macro]
pub fn showcase_module_enum(input: TokenStream) -> TokenStream {
    match expand_showcase_module_enum_macro(input) {
        Ok(tokens) => TokenStream::from(tokens),
        Err(e) => TokenStream::from(e.to_compile_error()),
    }
}
