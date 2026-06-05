#![doc = include_str!("../README.md")]

mod expand;
#[cfg(feature = "internal-showcase")]
mod showcase_modules;
#[cfg(test)]
mod tests;

use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use proc_macro2::TokenStream as TokenStream2;
use syn::spanned::Spanned as _;
use syn::{DeriveInput, Fields, Item};

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
    TokenStream::from(validator_macro(attr.into(), item.into()))
}

fn validator_macro(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    if !attr.is_empty() {
        return syn::Error::new_spanned(attr, "koruma::validator does not accept arguments")
            .to_compile_error();
    }

    let item = match syn::parse2::<Item>(item) {
        Ok(item) => item,
        Err(err) => return err.to_compile_error(),
    };

    let input = match item {
        Item::Struct(input) => input,
        other => {
            let err = syn::Error::new(
                other.span(),
                "koruma::validator can only be applied to structs",
            );
            return err.to_compile_error();
        },
    };

    if let Err(err) = validate_validator_macro_input(&input) {
        return err.to_compile_error();
    }

    match expand_validator(input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
}

fn validate_validator_macro_input(input: &syn::ItemStruct) -> syn::Result<()> {
    match &input.fields {
        Fields::Named(fields) if !fields.named.is_empty() => Ok(()),
        Fields::Named(_) | Fields::Unit => Err(syn::Error::new(
            input.ident.span(),
            "koruma::validator requires at least one field",
        )),
        Fields::Unnamed(_) => Err(syn::Error::new(
            input.ident.span(),
            "koruma::validator only supports structs with named fields",
        )),
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
/// `#[koruma(skip_capture)]` on an `Option<T>` value field can opt out
/// when they do not need to store the validated value.
#[proc_macro_error]
#[proc_macro_derive(Koruma, attributes(koruma))]
pub fn derive_koruma(input: TokenStream) -> TokenStream {
    TokenStream::from(derive_koruma_macro(input.into()))
}

fn derive_koruma_macro(input: TokenStream2) -> TokenStream2 {
    expand_derive_macro(input, expand_koruma)
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
    TokenStream::from(derive_koruma_all_display_macro(input.into()))
}

fn derive_koruma_all_display_macro(input: TokenStream2) -> TokenStream2 {
    expand_derive_macro(input, expand_koruma_all_display)
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
    TokenStream::from(derive_koruma_all_fluent_macro(input.into()))
}

#[cfg(feature = "fluent")]
fn derive_koruma_all_fluent_macro(input: TokenStream2) -> TokenStream2 {
    expand_derive_macro(input, expand_koruma_all_fluent)
}

fn expand_derive_macro(
    input: TokenStream2,
    expand: impl FnOnce(DeriveInput) -> syn::Result<TokenStream2>,
) -> TokenStream2 {
    let input = match syn::parse2::<DeriveInput>(input) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    match expand(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

/// Internal helper macro for generating showcase module declarations and linker functions.
#[cfg(feature = "internal-showcase")]
#[proc_macro_error]
#[proc_macro]
pub fn showcase_modules(input: TokenStream) -> TokenStream {
    TokenStream::from(showcase_modules_macro(input.into()))
}

/// Internal helper macro for generating `ValidatorModule`.
#[cfg(feature = "internal-showcase")]
#[proc_macro_error]
#[proc_macro]
pub fn showcase_module_enum(input: TokenStream) -> TokenStream {
    TokenStream::from(showcase_module_enum_macro(input.into()))
}

#[cfg(feature = "internal-showcase")]
fn showcase_modules_macro(input: TokenStream2) -> TokenStream2 {
    match expand_showcase_modules_macro(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

#[cfg(feature = "internal-showcase")]
fn showcase_module_enum_macro(input: TokenStream2) -> TokenStream2 {
    match expand_showcase_module_enum_macro(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

#[cfg(test)]
mod macro_entrypoint_tests {
    use super::*;
    use quote::quote;

    fn compact(tokens: TokenStream2) -> String {
        tokens
            .to_string()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect()
    }

    #[test]
    fn validator_macro_helper_covers_success_and_error_paths() {
        let valid = validator_macro(
            TokenStream2::new(),
            quote! {
                pub struct DemoValidation {
                    #[koruma(value)]
                    actual: Option<String>,
                }
            },
        );
        assert!(!valid.to_string().contains("compile_error"));

        let with_args = validator_macro(
            quote!(unexpected),
            quote! {
                pub struct DemoValidation {
                    #[koruma(value)]
                    actual: Option<String>,
                }
            },
        );
        assert!(with_args.to_string().contains("does not accept arguments"));

        let non_struct = validator_macro(
            TokenStream2::new(),
            quote!(
                enum Demo {}
            ),
        );
        assert!(
            non_struct
                .to_string()
                .contains("can only be applied to structs")
        );

        let empty_named = validator_macro(
            TokenStream2::new(),
            quote!(
                struct Demo {}
            ),
        );
        assert!(
            empty_named
                .to_string()
                .contains("requires at least one field")
        );

        let unit_struct = validator_macro(
            TokenStream2::new(),
            quote!(
                struct Demo;
            ),
        );
        assert!(
            unit_struct
                .to_string()
                .contains("requires at least one field")
        );

        let tuple_struct = validator_macro(
            TokenStream2::new(),
            quote!(
                struct Demo(String);
            ),
        );
        assert!(
            tuple_struct
                .to_string()
                .contains("only supports structs with named fields")
        );

        let expansion_error = validator_macro(
            TokenStream2::new(),
            quote! {
                pub struct DemoValidation {
                    actual: Option<String>,
                }
            },
        );
        assert!(expansion_error.to_string().contains("koruma::validator"));
    }

    #[test]
    fn derive_macro_helpers_cover_success_and_parse_errors() {
        let valid = derive_koruma_macro(quote! {
            pub struct Demo {
                #[koruma(RequiredValidation)]
                value: String,
            }
        });
        assert!(!valid.to_string().contains("compile_error"));

        let parse_error = derive_koruma_macro(quote!(not rust tokens));
        assert!(parse_error.to_string().contains("compile_error"));

        let expansion_error = derive_koruma_macro(quote! {
            pub enum Demo {
                Value,
            }
        });
        assert!(expansion_error.to_string().contains("structs"));

        let display = derive_koruma_all_display_macro(quote! {
            pub struct Demo {
                #[koruma(RequiredValidation)]
                value: String,
            }
        });
        assert!(compact(display).contains("DisplayforDemoValueKorumaValidatorRef"));

        #[cfg(feature = "fluent")]
        {
            let fluent = derive_koruma_all_fluent_macro(quote! {
                pub struct Demo {
                    #[koruma(RequiredValidation)]
                    value: String,
                }
            });
            assert!(compact(fluent).contains("FluentMessageforDemoValueKorumaValidatorRef"));
        }
    }

    #[cfg(feature = "internal-showcase")]
    #[test]
    fn showcase_macro_helpers_cover_success_and_error_paths() {
        let modules = showcase_modules_macro(quote!(string, numeric));
        assert!(compact(modules).contains("__link_showcase_validators"));

        let module_enum = showcase_module_enum_macro(quote!(string, numeric));
        let compact_enum = compact(module_enum);
        assert!(compact_enum.contains("enumValidatorModule"));
        assert!(compact_enum.contains("String"));
        assert!(compact_enum.contains("Numeric"));

        let empty = showcase_modules_macro(TokenStream2::new());
        assert!(
            empty
                .to_string()
                .contains("showcase_modules requires at least one module")
        );
    }
}
