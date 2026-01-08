//! Core expansion logic for koruma derive macros.
//!
//! This module contains the actual TokenStream generation that can be tested.

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{DeriveInput, Field, Fields, Ident, ItemStruct, Token, parse::Parse, parse::ParseStream};

/// Represents a single parsed validator: `ValidatorName(arg = value, ...)` or
/// `ValidatorName<_>(arg = value, ...)` or `ValidatorName<SomeType>(arg = value, ...)`
pub(crate) struct ValidatorAttr {
    pub validator: Ident,
    /// Whether the validator uses `<_>` syntax for type inference from field type.
    /// When true, the full field type is used (unwrapping Option if present).
    pub infer_type: bool,
    /// Explicit type parameter if specified (e.g., `<f64>`, `<Vec<_>>`)
    /// If this contains `_`, it will be substituted with the inner type from the field.
    pub explicit_type: Option<syn::Type>,
    pub args: Vec<(Ident, syn::Expr)>,
}

impl Parse for ValidatorAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let validator: Ident = input.parse()?;

        // Check for generic type parameter syntax: <_> or <SomeType>
        // <_> means "use the full field type" (unwrapping Option if present)
        // <Vec<_>> means "substitute _ with the inner type from the field"
        let (infer_type, explicit_type) = if input.peek(Token![<]) {
            input.parse::<Token![<]>()?;
            // Check if it's <_> (full type inference) or an explicit/partial type
            if input.peek(Token![_]) {
                input.parse::<Token![_]>()?;
                input.parse::<Token![>]>()?;
                // <_> means use the full field type
                (true, None)
            } else {
                // Parse explicit type parameter and store it
                // This handles types like <f64>, <Vec<u8>>, <Vec<_>>, etc.
                let explicit_type: syn::Type = input.parse()?;
                input.parse::<Token![>]>()?;
                (false, Some(explicit_type))
            }
        } else {
            (false, None)
        };

        let args = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            let mut args = Vec::new();
            while !content.is_empty() {
                let name: Ident = content.parse()?;
                content.parse::<Token![=]>()?;
                let value: syn::Expr = content.parse()?;

                args.push((name, value));

                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
            args
        } else {
            Vec::new()
        };

        Ok(ValidatorAttr {
            validator,
            infer_type,
            explicit_type,
            args,
        })
    }
}

/// Represents a parsed `#[koruma(...)]` attribute which can contain multiple validators
/// separated by commas: `#[koruma(Validator1(a = 1), Validator2(b = 2))]`
/// Can also include `each` modifier for collection validation:
/// `#[koruma(VecValidator(min = 0), each(ElementValidator(max = 100)))]`
pub(crate) struct KorumaAttr {
    /// Validators applied to the field/collection itself
    pub field_validators: Vec<ValidatorAttr>,
    /// Validators applied to each element in a collection (from `each(...)`)
    pub element_validators: Vec<ValidatorAttr>,
    pub is_skip: bool,
}

impl Parse for KorumaAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for skip
        if input.peek(Ident) {
            let fork = input.fork();
            let ident: Ident = fork.parse()?;
            if ident == "skip" && fork.is_empty() {
                input.parse::<Ident>()?; // consume "skip"
                return Ok(KorumaAttr {
                    field_validators: Vec::new(),
                    element_validators: Vec::new(),
                    is_skip: true,
                });
            }
        }

        let mut field_validators = Vec::new();
        let mut element_validators = Vec::new();

        // Parse comma-separated items (validators or each(...))
        while !input.is_empty() {
            // Check if this is an `each(...)` block
            if input.peek(Ident) {
                let fork = input.fork();
                let ident: Ident = fork.parse()?;
                if ident == "each" && fork.peek(syn::token::Paren) {
                    input.parse::<Ident>()?; // consume "each"
                    let content;
                    syn::parenthesized!(content in input);

                    // Parse validators inside each(...)
                    while !content.is_empty() {
                        element_validators.push(content.parse::<ValidatorAttr>()?);
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        } else {
                            break;
                        }
                    }

                    // Continue parsing after each(...)
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    }
                    continue;
                }
            }

            // Regular validator
            field_validators.push(input.parse::<ValidatorAttr>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            } else {
                break;
            }
        }

        Ok(KorumaAttr {
            field_validators,
            element_validators,
            is_skip: false,
        })
    }
}

/// Field info extracted from the struct
pub(crate) struct FieldInfo {
    pub name: Ident,
    pub ty: syn::Type,
    /// Validators for the field/collection itself
    pub field_validators: Vec<ValidatorAttr>,
    /// Validators for each element in a collection
    pub element_validators: Vec<ValidatorAttr>,
}

impl FieldInfo {
    /// Returns true if this field has element validators (uses `each(...)`)
    pub fn has_element_validators(&self) -> bool {
        !self.element_validators.is_empty()
    }
}

pub(crate) fn parse_field(field: &Field) -> Option<FieldInfo> {
    let name = field.ident.clone()?;
    let ty = field.ty.clone();

    for attr in &field.attrs {
        if !attr.path().is_ident("koruma") {
            continue;
        }

        // Parse the attribute content
        let parsed: syn::Result<KorumaAttr> = attr.parse_args();

        match parsed {
            Ok(koruma_attr) => {
                // Check for skip
                if koruma_attr.is_skip {
                    return None;
                }
                // Must have at least one validator
                if koruma_attr.field_validators.is_empty()
                    && koruma_attr.element_validators.is_empty()
                {
                    return None;
                }
                return Some(FieldInfo {
                    name,
                    ty,
                    field_validators: koruma_attr.field_validators,
                    element_validators: koruma_attr.element_validators,
                });
            },
            Err(_) => {
                return None;
            },
        }
    }

    // Field without koruma attribute - skip it
    None
}

/// Find the field marked with #[koruma(value)] and return its name and type
pub(crate) fn find_value_field(input: &ItemStruct) -> Option<(Ident, syn::Type)> {
    if let Fields::Named(ref fields) = input.fields {
        for field in &fields.named {
            for attr in &field.attrs {
                if attr.path().is_ident("koruma") {
                    // Try to parse as just "value"
                    if let Ok(ident) = attr.parse_args::<Ident>()
                        && ident == "value"
                    {
                        return Some((field.ident.clone().unwrap(), field.ty.clone()));
                    }
                }
            }
        }
    }
    None
}

/// Extract the inner type T from Option<T>
pub(crate) fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// Extract the inner type T from Vec<T>
pub(crate) fn vec_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;

    if segment.ident != "Vec" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };

    match args.args.first()? {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

/// Parsed showcase attribute: `#[showcase(name = "...", description = "...", create = |input| { ... })]`
/// The `create` closure takes a `&str` and returns the validator instance.
#[cfg(feature = "showcase")]
struct ShowcaseAttr {
    name: syn::LitStr,
    description: syn::LitStr,
    create: syn::ExprClosure,
}

#[cfg(feature = "showcase")]
impl Parse for ShowcaseAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut name: Option<syn::LitStr> = None;
        let mut description: Option<syn::LitStr> = None;
        let mut create: Option<syn::ExprClosure> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                },
                "description" => {
                    description = Some(input.parse()?);
                },
                "create" => {
                    create = Some(input.parse()?);
                },
                other => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!("unknown showcase attribute: {}", other),
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ShowcaseAttr {
            name: name.ok_or_else(|| {
                syn::Error::new(input.span(), "showcase requires `name` attribute")
            })?,
            description: description.ok_or_else(|| {
                syn::Error::new(input.span(), "showcase requires `description` attribute")
            })?,
            create: create.ok_or_else(|| {
                syn::Error::new(input.span(), "showcase requires `create` attribute")
            })?,
        })
    }
}

/// Find and parse showcase attribute from struct
#[cfg(feature = "showcase")]
fn find_showcase_attr(input: &ItemStruct) -> Option<ShowcaseAttr> {
    for attr in &input.attrs {
        if attr.path().is_ident("showcase") {
            if let Ok(parsed) = attr.parse_args::<ShowcaseAttr>() {
                return Some(parsed);
            }
        }
    }
    None
}

/// Core expansion logic for the `#[validator]` attribute macro.
///
/// Takes a parsed struct and returns the expanded TokenStream.
pub fn expand_validator(mut input: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    // Check if the struct has generics
    let has_generics = !input.generics.params.is_empty();

    // Parse showcase attribute if present (only when feature enabled)
    #[cfg(feature = "showcase")]
    let showcase_attr = find_showcase_attr(&input);

    // Find the field marked with #[koruma(value)]
    let (value_field_name, value_field_type) = find_value_field(&input).ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "koruma::validator requires a field marked with #[koruma(value)].\n\
             Example:\n\
             #[koruma(value)]\n\
             actual: Option<i32>",
        )
    })?;

    // Extract the inner type from Option<T>
    let inner_type = option_inner_type(&value_field_type).unwrap_or(&value_field_type);

    // Add #[derive(bon::Builder)] to the existing attributes
    let builder_attr: syn::Attribute = syn::parse_quote!(#[derive(koruma::bon::Builder)]);
    input.attrs.insert(0, builder_attr);

    // Remove #[koruma(value)] and #[showcase(...)] from attributes
    input.attrs.retain(|attr| !attr.path().is_ident("showcase"));

    // Remove #[koruma(value)] from the field so bon doesn't see it
    if let Fields::Named(ref mut fields) = input.fields {
        for field in &mut fields.named {
            field.attrs.retain(|attr| {
                if attr.path().is_ident("koruma")
                    && let Ok(ident) = attr.parse_args::<Ident>()
                {
                    return ident != "value";
                }

                true
            });
        }
    }

    // Generate the module name that bon creates (snake_case of struct name + _builder)
    let module_name = format_ident!("{}_builder", struct_name.to_string().to_snake_case());

    // Generate the associated type name (PascalCase of field name) and Set wrapper
    let value_pascal = value_field_name.to_string().to_upper_camel_case();
    let value_assoc_type = format_ident!("{}", value_pascal);
    let set_value_type = format_ident!("Set{}", value_pascal);

    let with_value_impl = if has_generics {
        // For generic validators, the builder is Builder<T, S> (type param first, then state)
        // Use the actual field type (inner_type) for the value parameter
        //
        // We need to propagate the bounds from the original struct's generics.
        // The builder has form: StructBuilder<T, S> where T has the original bounds and S is builder state.

        // Extract just the type parameter names (without bounds) for use in type position
        let type_param_names: Vec<_> = input
            .generics
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Type(t) => Some(&t.ident),
                _ => None,
            })
            .collect();

        // Extract bounds from the generic params to put in where clause
        let type_param_bounds: Vec<_> = input
            .generics
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Type(t) if !t.bounds.is_empty() => {
                    let ident = &t.ident;
                    let bounds = &t.bounds;
                    Some(quote! { #ident: #bounds })
                },
                _ => None,
            })
            .collect();

        let where_clause = &input.generics.where_clause;

        // Build where predicates: type param bounds + original where clause + S::Value: IsUnset
        let where_predicates = {
            let mut predicates = type_param_bounds;
            if let Some(wc) = where_clause {
                for pred in &wc.predicates {
                    predicates.push(quote! { #pred });
                }
            }
            predicates.push(quote! { S::#value_assoc_type: koruma::bon::IsUnset });
            predicates
        };

        quote! {
            impl<#(#type_param_names,)* S: #module_name::State> #builder_name<#(#type_param_names,)* S>
            where
                #(#where_predicates),*
            {
                /// Sets the value field. This is auto-generated by `#[koruma::validator]`.
                pub fn with_value(self, value: #inner_type) -> #builder_name<#(#type_param_names,)* #module_name::#set_value_type<S>> {
                    self.#value_field_name(value)
                }
            }
        }
    } else {
        quote! {
            impl<S: #module_name::State> #builder_name<S>
            where
                S::#value_assoc_type: koruma::bon::IsUnset,
            {
                /// Sets the value field. This is auto-generated by `#[koruma::validator]`.
                pub fn with_value(self, value: #inner_type) -> #builder_name<#module_name::#set_value_type<S>> {
                    self.#value_field_name(value)
                }
            }
        }
    };

    // Generate showcase registration if the attribute is present
    #[cfg(feature = "showcase")]
    let showcase_registration = if let Some(showcase) = showcase_attr {
        let name = &showcase.name;
        let description = &showcase.description;
        let create_closure = &showcase.create;

        // Extract generics from the struct
        let (impl_generics, type_generics, _where_clause) = input.generics.split_for_impl();

        // Extract bounds from the generic params for the where clause
        let mut where_predicates = Vec::new();
        for param in input.generics.params.iter() {
            if let syn::GenericParam::Type(t) = param {
                let ident = &t.ident;
                // Add all existing bounds plus Send + Sync + Clone + 'static
                // If no existing bounds, don't include the leading `+`
                if t.bounds.is_empty() {
                    where_predicates.push(quote! { #ident: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static });
                } else {
                    let bounds = &t.bounds;
                    where_predicates.push(quote! { #ident: #bounds + ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static });
                }
            }
        }
        // Add Self: Display bound
        where_predicates.push(quote! { Self: ::std::fmt::Display });

        let combined_where = if where_predicates.is_empty() {
            quote! {}
        } else {
            quote! { where #(#where_predicates),* }
        };

        quote! {
            // DynValidator is implemented by validators that have Validate + Display impls
            #[cfg(feature = "showcase")]
            impl #impl_generics ::koruma::showcase::DynValidator for #struct_name #type_generics
            #combined_where
            {
                fn is_valid(&self) -> bool {
                    ::koruma::Validate::validate(self, &self.#value_field_name)
                }

                fn display_string(&self) -> String {
                    #[cfg(feature = "fmt")]
                    { ::std::string::ToString::to_string(self) }
                    #[cfg(not(feature = "fmt"))]
                    { "(fmt feature required)".to_string() }
                }

                fn fluent_string(&self) -> String {
                    #[cfg(feature = "fluent")]
                    {
                        use ::es_fluent::ToFluentString as _;
                        self.to_fluent_string()
                    }
                    #[cfg(not(feature = "fluent"))]
                    { "(fluent feature required)".to_string() }
                }
            }

            ::koruma::inventory::submit! {
                ::koruma::showcase::ValidatorShowcase {
                    name: #name,
                    description: #description,
                    create_validator: |input: &str| -> Box<dyn ::koruma::showcase::DynValidator> {
                        Box::new((#create_closure)(input))
                    },
                }
            }
        }
    } else {
        quote! {}
    };

    #[cfg(not(feature = "showcase"))]
    let showcase_registration = quote! {};

    Ok(quote! {
        #input

        #with_value_impl

        #showcase_registration
    })
}

/// Substitute infer placeholders (`_`) in a type with the actual inferred type.
/// For example, `Vec<_>` with infer_ty=`String` becomes `Vec<String>`.
fn substitute_infer_type(ty: &syn::Type, infer_ty: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Infer(_) => infer_ty.clone(),
        syn::Type::Path(type_path) => {
            let mut new_path = type_path.clone();
            for segment in &mut new_path.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg {
                            *inner_ty = substitute_infer_type(inner_ty, infer_ty);
                        }
                    }
                }
            }
            syn::Type::Path(new_path)
        },
        _ => ty.clone(),
    }
}

/// Extract the first generic type argument from a type.
/// For example, `Vec<String>` → `String`, `HashSet<i32>` → `i32`.
fn first_generic_arg(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(type_path) = ty
        && let Some(segment) = type_path.path.segments.last()
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
    {
        for arg in &args.args {
            if let syn::GenericArgument::Type(inner_ty) = arg {
                return Some(inner_ty);
            }
        }
    }
    None
}

/// Check if a type contains any infer placeholders (`_`).
fn contains_infer_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Infer(_) => true,
        syn::Type::Path(type_path) => {
            for segment in &type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    for arg in &args.args {
                        if let syn::GenericArgument::Type(inner_ty) = arg
                            && contains_infer_type(inner_ty)
                        {
                            return true;
                        }
                    }
                }
            }
            false
        },
        _ => false,
    }
}

/// Helper to generate the type for a validator
///
/// Type inference behavior:
/// - `<_>`: uses the full field type (unwrapping Option if present)
/// - `<Vec<_>>`: substitutes `_` with the inner type from the field
/// - `<SomeType>`: uses the explicit type directly
/// - For `each` validation on `Vec<T>`: uses T
/// - For optional fields `Option<T>`: uses T (validation is skipped if None)
fn validator_type_for_field(
    v: &ValidatorAttr,
    field_ty: &syn::Type,
    validate_each: bool,
) -> TokenStream2 {
    let validator = &v.validator;

    // If explicit type is provided, check if it contains `_` for substitution
    if let Some(ref explicit_ty) = v.explicit_type {
        if contains_infer_type(explicit_ty) {
            // Substitute `_` with the inner type from the field
            // e.g., Vec<_> on field Vec<String> → Vec<String>
            // e.g., HashSet<_> on field HashSet<i32> → HashSet<i32>
            let inner_ty = first_generic_arg(field_ty).unwrap_or(field_ty);
            let substituted = substitute_infer_type(explicit_ty, inner_ty);
            return quote! { #validator<#substituted> };
        }
        return quote! { #validator<#explicit_ty> };
    }

    // For `each` validation, unwrap Vec<T> to get element type T
    let after_vec = if validate_each {
        vec_inner_type(field_ty).unwrap_or(field_ty)
    } else {
        field_ty
    };

    // Unwrap Option<T> for optional field validation
    let effective_ty = option_inner_type(after_vec).unwrap_or(after_vec);

    if v.infer_type {
        // <_> means use the full field type (after unwrapping Option)
        quote! { #validator<#effective_ty> }
    } else {
        quote! { #validator }
    }
}

/// Get the effective type for validation (unwrapping Option and Vec as needed)
fn effective_validation_type(field_ty: &syn::Type, validate_each: bool) -> &syn::Type {
    // Unwrap Vec<T> for each validation
    let after_vec = if validate_each {
        vec_inner_type(field_ty).unwrap_or(field_ty)
    } else {
        field_ty
    };

    // Unwrap Option<T> for optional field validation
    option_inner_type(after_vec).unwrap_or(after_vec)
}

/// Check if a field type is Option<T>
fn is_option_type(ty: &syn::Type) -> bool {
    option_inner_type(ty).is_some()
}

/// Core expansion logic for the `#[derive(Koruma)]` derive macro.
///
/// Takes a parsed DeriveInput and returns the expanded TokenStream.
pub fn expand_koruma(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let error_struct_name = format_ident!("{}KorumaValidationError", struct_name);

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Koruma only supports structs with named fields",
                ));
            },
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Koruma can only be derived for structs",
            ));
        },
    };

    // Parse all fields and extract validation info
    let field_infos: Vec<FieldInfo> = fields.iter().filter_map(parse_field).collect();

    if field_infos.is_empty() {
        return Err(syn::Error::new_spanned(
            &input,
            "Koruma requires at least one field with a #[koruma(...)] attribute",
        ));
    }

    // Generate per-field error structs and collect info for main error struct
    let field_error_structs: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let has_element_validators = f.has_element_validators();
            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            // Generate fields for field-level validators
            let field_validator_fields: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #validator_snake: Option<#vtype> }
                })
                .collect();

            // Generate getter methods for field-level validators
            let field_validator_getters: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! {
                        pub fn #validator_snake(&self) -> Option<&#vtype> {
                            self.#validator_snake.as_ref()
                        }
                    }
                })
                .collect();

            // Generate is_empty checks for field-level validators
            let field_is_empty_checks: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    quote! { self.#validator_snake.is_none() }
                })
                .collect();

            // Generate element error struct if we have element validators
            let element_error_struct = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );

                let element_validator_fields: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.validator.to_string().to_snake_case());
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! { #validator_snake: Option<#vtype> }
                    })
                    .collect();

                let element_validator_getters: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.validator.to_string().to_snake_case());
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! {
                            pub fn #validator_snake(&self) -> Option<&#vtype> {
                                self.#validator_snake.as_ref()
                            }
                        }
                    })
                    .collect();

                let element_is_empty_checks: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.validator.to_string().to_snake_case());
                        quote! { self.#validator_snake.is_none() }
                    })
                    .collect();

                // Generate enum variants for the element all() method
                let element_enum_name = format_ident!(
                    "{}{}ElementKorumaValidator",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );

                let element_enum_variants: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let variant_name =
                            format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                        let vtype = validator_type_for_field(v, field_ty, true);
                        quote! { #variant_name(#vtype) }
                    })
                    .collect();

                let element_all_pushes: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.validator.to_string().to_snake_case());
                        let variant_name =
                            format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                        quote! {
                            if let Some(v) = &self.#validator_snake {
                                result.push(#element_enum_name::#variant_name(v.clone()));
                            }
                        }
                    })
                    .collect();

                quote! {
                    /// Enum of all possible element validators for this field.
                    #[derive(Clone, Debug)]
                    #[allow(dead_code)]
                    pub enum #element_enum_name {
                        #(#element_enum_variants),*
                    }

                    /// Per-element validation error struct.
                    #[derive(Clone, Debug)]
                    pub struct #element_error_struct_name {
                        #(#element_validator_fields),*
                    }

                    impl #element_error_struct_name {
                        #(#element_validator_getters)*

                        /// Returns all failed element validators.
                        pub fn all(&self) -> Vec<#element_enum_name> {
                            let mut result = Vec::new();
                            #(#element_all_pushes)*
                            result
                        }

                        pub fn is_empty(&self) -> bool {
                            #(#element_is_empty_checks)&&*
                        }

                        pub fn has_errors(&self) -> bool {
                            !self.is_empty()
                        }
                    }
                }
            } else {
                quote! {}
            };

            // Field for storing element errors (if we have element validators)
            let _element_errors_field = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! { element_errors: Vec<(usize, #element_error_struct_name)> }
            } else {
                quote! {}
            };

            // Getter for element errors
            let element_errors_getter = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! {
                    /// Returns all element validation errors with their indices.
                    pub fn element_errors(&self) -> &[(usize, #element_error_struct_name)] {
                        &self.element_errors
                    }
                }
            } else {
                quote! {}
            };

            // is_empty check for element errors
            let element_is_empty_check = if has_element_validators {
                quote! { && self.element_errors.is_empty() }
            } else {
                quote! {}
            };

            // Generate enum variants for the field all() method
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let enum_variants: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    let vtype = validator_type_for_field(v, field_ty, false);
                    quote! { #variant_name(#vtype) }
                })
                .collect();

            // Generate the all() method body
            let all_pushes: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    quote! {
                        if let Some(v) = &self.#validator_snake {
                            result.push(#enum_name::#variant_name(v.clone()));
                        }
                    }
                })
                .collect();

            // Handle case where there are no field validators (only element validators)
            let enum_and_all = if f.field_validators.is_empty() {
                quote! {}
            } else {
                quote! {
                    /// Enum of all possible validators for this field.
                    #[derive(Clone, Debug)]
                    #[allow(dead_code)]
                    pub enum #enum_name {
                        #(#enum_variants),*
                    }
                }
            };

            let all_method = if f.field_validators.is_empty() {
                quote! {}
            } else {
                quote! {
                    /// Returns all failed field-level validators.
                    pub fn all(&self) -> Vec<#enum_name> {
                        let mut result = Vec::new();
                        #(#all_pushes)*
                        result
                    }
                }
            };

            let is_empty_body = if f.field_validators.is_empty() {
                // Only element validators
                quote! { self.element_errors.is_empty() }
            } else {
                quote! { #(#field_is_empty_checks)&&* #element_is_empty_check }
            };

            // Generate struct fields - need proper comma handling
            let struct_fields = if !f.field_validators.is_empty() && f.has_element_validators() {
                // Both field validators and element errors
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! {
                    #(#field_validator_fields,)*
                    element_errors: Vec<(usize, #element_error_struct_name)>
                }
            } else if f.has_element_validators() {
                // Only element errors
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );
                quote! {
                    element_errors: Vec<(usize, #element_error_struct_name)>
                }
            } else {
                // Only field validators
                quote! {
                    #(#field_validator_fields),*
                }
            };

            quote! {
                #element_error_struct

                #enum_and_all

                #[derive(Clone, Debug)]
                pub struct #field_error_struct_name {
                    #struct_fields
                }

                impl #field_error_struct_name {
                    #(#field_validator_getters)*

                    #element_errors_getter

                    #all_method

                    pub fn is_empty(&self) -> bool {
                        #is_empty_body
                    }

                    pub fn has_errors(&self) -> bool {
                        !self.is_empty()
                    }
                }
            }
        })
        .collect();

    // Generate main error struct fields (one per validated field)
    // Now all fields just have their field error struct (element errors are nested inside)
    let error_fields: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            quote! { #field_name: #field_error_struct_name }
        })
        .collect();

    // Generate getter methods for main error struct
    let getter_methods: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            quote! {
                pub fn #field_name(&self) -> &#field_error_struct_name {
                    &self.#field_name
                }
            }
        })
        .collect();

    // Generate is_empty check (all field error structs are empty)
    let is_empty_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            quote! { self.#field_name.is_empty() }
        })
        .collect();

    // Generate default values for main error struct initialization
    let error_defaults: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            // Generate defaults for field-level validators
            let field_validator_defaults: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        format_ident!("{}", v.validator.to_string().to_snake_case());
                    quote! { #validator_snake: None }
                })
                .collect();

            // Handle different combinations of field/element validators
            if f.has_element_validators() && f.field_validators.is_empty() {
                // Only element validators
                quote! {
                    #field_name: #field_error_struct_name {
                        element_errors: Vec::new()
                    }
                }
            } else if f.has_element_validators() {
                // Both field and element validators
                quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*,
                        element_errors: Vec::new()
                    }
                }
            } else {
                // Only field validators
                quote! {
                    #field_name: #field_error_struct_name {
                        #(#field_validator_defaults),*
                    }
                }
            }
        })
        .collect();

    // Generate validation logic - supports both field validators and element validators
    let validation_checks: Vec<TokenStream2> = field_infos
        .iter()
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let has_element_validators = f.has_element_validators();

            // Generate field-level validation checks
            let field_validator_checks: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let validator = &v.validator;
                    let validator_snake =
                        format_ident!("{}", validator.to_string().to_snake_case());
                    // For field validators, we never unwrap Vec - only Option.
                    // <_> uses the full field type (Vec<T> stays Vec<T>)
                    let effective_ty = effective_validation_type(field_ty, false);

                    let builder_calls: Vec<TokenStream2> = v
                        .args
                        .iter()
                        .map(|(arg_name, arg_value)| {
                            quote! { .#arg_name(#arg_value) }
                        })
                        .collect();

                    if v.infer_type {
                        let assert_fn = format_ident!(
                            "__koruma_assert_validate_{}_{}_field",
                            field_name,
                            validator_snake
                        );
                        quote! {
                            fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> bool {
                                v.validate(t)
                            }
                            let validator = #validator::<#effective_ty>::builder()
                                #(#builder_calls)*
                                .with_value(__field_value.clone())
                                .build();
                            if !#assert_fn(&validator, __field_value) {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    } else {
                        quote! {
                            let validator = #validator::builder()
                                #(#builder_calls)*
                                .with_value(__field_value.clone())
                                .build();
                            if !validator.validate(__field_value) {
                                error.#field_name.#validator_snake = Some(validator);
                                has_error = true;
                            }
                        }
                    }
                })
                .collect();

            // Generate element-level validation checks if we have element validators
            let element_validation = if has_element_validators {
                let element_error_struct_name = format_ident!(
                    "{}{}ElementKorumaValidationError",
                    struct_name,
                    field_name.to_string().to_upper_camel_case()
                );

                let element_ty = vec_inner_type(field_ty).unwrap_or(field_ty);
                let element_is_optional = is_option_type(element_ty);
                let effective_element_ty = effective_validation_type(field_ty, true);

                let element_validator_checks: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator = &v.validator;
                        let validator_snake =
                            format_ident!("{}", validator.to_string().to_snake_case());

                        let builder_calls: Vec<TokenStream2> = v
                            .args
                            .iter()
                            .map(|(arg_name, arg_value)| {
                                quote! { .#arg_name(#arg_value) }
                            })
                            .collect();

                        if v.infer_type {
                            let assert_fn = format_ident!(
                                "__koruma_assert_validate_{}_{}_element",
                                field_name,
                                validator_snake
                            );
                            quote! {
                                fn #assert_fn<V: koruma::Validate<T>, T>(v: &V, t: &T) -> bool {
                                    v.validate(t)
                                }
                                let validator = #validator::<#effective_element_ty>::builder()
                                    #(#builder_calls)*
                                    .with_value(__item_value.clone())
                                    .build();
                                if !#assert_fn(&validator, __item_value) {
                                    element_error.#validator_snake = Some(validator);
                                    element_has_error = true;
                                }
                            }
                        } else {
                            quote! {
                                let validator = #validator::builder()
                                    #(#builder_calls)*
                                    .with_value(__item_value.clone())
                                    .build();
                                if !validator.validate(__item_value) {
                                    element_error.#validator_snake = Some(validator);
                                    element_has_error = true;
                                }
                            }
                        }
                    })
                    .collect();

                let element_validator_defaults: Vec<TokenStream2> = f
                    .element_validators
                    .iter()
                    .map(|v| {
                        let validator_snake =
                            format_ident!("{}", v.validator.to_string().to_snake_case());
                        quote! { #validator_snake: None }
                    })
                    .collect();

                let inner_element_validation = quote! {
                    let mut element_error = #element_error_struct_name {
                        #(#element_validator_defaults),*
                    };
                    let mut element_has_error = false;

                    #(#element_validator_checks)*

                    if element_has_error {
                        error.#field_name.element_errors.push((idx, element_error));
                        has_error = true;
                    }
                };

                if element_is_optional {
                    // For Vec<Option<T>>, skip None items
                    quote! {
                        for (idx, item) in self.#field_name.iter().enumerate() {
                            if let Some(ref __item_value) = item {
                                #inner_element_validation
                            }
                        }
                    }
                } else {
                    // For Vec<T>, validate each item directly
                    quote! {
                        for (idx, __item_value) in self.#field_name.iter().enumerate() {
                            #inner_element_validation
                        }
                    }
                }
            } else {
                quote! {}
            };

            // Combine field validation and element validation
            let field_is_optional = is_option_type(field_ty);

            // For fields with element validators, the field value is the Vec itself
            // For regular fields, it's just the field value
            if !f.field_validators.is_empty() && field_is_optional {
                // Optional field with field validators
                quote! {
                    if let Some(ref __field_value) = self.#field_name {
                        #(#field_validator_checks)*
                    }
                    #element_validation
                }
            } else if !f.field_validators.is_empty() {
                // Non-optional field with field validators
                quote! {
                    let __field_value = &self.#field_name;
                    #(#field_validator_checks)*
                    #element_validation
                }
            } else {
                // No field validators, only element validators
                element_validation
            }
        })
        .collect();

    Ok(quote! {
        // Per-field error structs
        #(#field_error_structs)*

        /// Auto-generated validation error struct for [`#struct_name`].
        ///
        /// Each field contains a nested error struct with `Option<Validator>` for each
        /// validator. Access errors via chained calls like `error.field().validator()`.
        #[derive(Clone, Debug)]
        pub struct #error_struct_name {
            #(#error_fields),*
        }

        impl #error_struct_name {
            #(#getter_methods)*
        }

        impl koruma::ValidationError for #error_struct_name {
            fn is_empty(&self) -> bool {
                #(#is_empty_checks)&&*
            }
        }

        impl #struct_name {
            /// Validates all fields and returns an error struct containing
            /// all validation failures.
            ///
            /// Returns `Ok(())` if all validations pass, or `Err(error)` where
            /// `error` contains the validation failures for each field.
            pub fn validate(&self) -> Result<(), #error_struct_name> {
                let mut error = #error_struct_name {
                    #(#error_defaults),*
                };
                let mut has_error = false;

                #(#validation_checks)*

                if has_error {
                    Err(error)
                } else {
                    Ok(())
                }
            }
        }
    })
}

/// Core expansion logic for the `#[derive(KorumaAllDisplay)]` derive macro.
///
/// Generates `Display` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's Display.
pub fn expand_koruma_all_display(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "KorumaAllDisplay only supports structs with named fields",
                ));
            },
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "KorumaAllDisplay can only be derived for structs",
            ));
        },
    };

    // Parse all fields and extract validation info
    let field_infos: Vec<FieldInfo> = fields.iter().filter_map(parse_field).collect();

    // Generate Display impls for each field's validator enum
    let display_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.field_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(v, f)
                    }
                })
                .collect();

            quote! {
                impl ::std::fmt::Display for #enum_name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    // Generate Display impls for element validator enums (if any)
    let element_display_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.element_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}ElementKorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .element_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => ::std::fmt::Display::fmt(v, f)
                    }
                })
                .collect();

            quote! {
                impl ::std::fmt::Display for #enum_name {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#display_impls)*
        #(#element_display_impls)*
    })
}

/// Core expansion logic for the `#[derive(KorumaAllFluent)]` derive macro.
///
/// Generates `ToFluentString` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's ToFluentString.
#[cfg(feature = "fluent")]
pub fn expand_koruma_all_fluent(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "KorumaAllFluent only supports structs with named fields",
                ));
            },
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "KorumaAllFluent can only be derived for structs",
            ));
        },
    };

    // Parse all fields and extract validation info
    let field_infos: Vec<FieldInfo> = fields.iter().filter_map(parse_field).collect();

    // Generate ToFluentString impls for each field's validator enum
    let fluent_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.field_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .field_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string()
                    }
                })
                .collect();

            quote! {
                impl koruma::es_fluent::ToFluentString for #enum_name {
                    fn to_fluent_string(&self) -> String {
                        use koruma::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    // Generate ToFluentString impls for element validator enums (if any)
    let element_fluent_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.element_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let enum_name = format_ident!(
                "{}{}ElementKorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );

            let match_arms: Vec<TokenStream2> = f
                .element_validators
                .iter()
                .map(|v| {
                    let variant_name =
                        format_ident!("{}", v.validator.to_string().to_upper_camel_case());
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string()
                    }
                })
                .collect();

            quote! {
                impl koruma::es_fluent::ToFluentString for #enum_name {
                    fn to_fluent_string(&self) -> String {
                        use koruma::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#fluent_impls)*
        #(#element_fluent_impls)*
    })
}

#[cfg(test)]
mod attr_parsing_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod helper_tests;
#[cfg(test)]
mod snapshot_tests;
