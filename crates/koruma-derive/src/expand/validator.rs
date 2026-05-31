use super::koruma_crate_path;
#[cfg(feature = "internal-showcase")]
use super::{ShowcaseInputType, ShowcaseModule};
use heck::{ToSnakeCase, ToUpperCamelCase};
#[cfg(feature = "internal-showcase")]
use koruma_derive_core::find_showcase_attr;
use koruma_derive_core::{
    CapturePolicy, SetterDefault, ValidatorFieldRole, option_inner_type,
    parse_validator_fields_strict,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Fields, GenericParam, Ident, ItemStruct, Type, Visibility, parse_quote};

/// Core expansion logic for the `#[validator]` attribute macro.
///
/// Takes a parsed struct and returns the expanded TokenStream.
pub fn expand_validator(mut input: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);
    let module_name = format_ident!("{}_builder", struct_name.to_string().to_snake_case());
    let koruma = koruma_crate_path();

    if !matches!(input.fields, Fields::Named(_)) {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "koruma::validator only supports structs with named fields",
        ));
    }

    #[cfg(feature = "internal-showcase")]
    let showcase_attr = find_showcase_attr(&input)?;

    let validator_spec = parse_validator_fields_strict(&input)?;
    let value_field = validator_spec.value_field();
    let value_field_name = value_field.name.clone();
    let value_field_type = value_field.ty.clone();
    let value_field_capture = validator_spec.value_spec().capture;
    let inner_type = option_inner_type(&value_field_type).unwrap_or(&value_field_type);

    if value_field_capture == CapturePolicy::Skip && option_inner_type(&value_field_type).is_none()
    {
        return Err(syn::Error::new_spanned(
            &value_field_type,
            "`#[koruma(value(capture = skip))]` currently requires an `Option<T>` field",
        ));
    }

    input.attrs.retain(|attr| !attr.path().is_ident("showcase"));

    let Fields::Named(ref fields) = input.fields else {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "koruma::validator only supports structs with named fields",
        ));
    };
    for field in &fields.named {
        reject_builder_attrs(field)?;
        if field.ident.as_ref() == Some(&value_field_name) {
            if !matches!(field.vis, Visibility::Inherited) {
                return Err(syn::Error::new_spanned(
                    &field.vis,
                    format!(
                        "`#[koruma(value)]` field `{}` must be private; use the generated getter instead",
                        value_field_name
                    ),
                ));
            }
        }
    }

    let slots = builder_slots(&validator_spec);

    let Fields::Named(ref mut fields) = input.fields else {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "koruma::validator only supports structs with named fields",
        ));
    };
    for field in &mut fields.named {
        field.attrs.retain(|attr| !attr.path().is_ident("koruma"));
    }
    let required_slots: Vec<_> = slots.iter().filter(|slot| slot.is_required()).collect();
    let builder_generic_args = generic_args(&input.generics);
    let initial_state_args: Vec<_> = required_slots
        .iter()
        .map(|_| quote! { #module_name::Empty })
        .collect();
    let set_state_args: Vec<_> = required_slots
        .iter()
        .map(|_| quote! { #module_name::Set })
        .collect();
    let initial_builder_ty =
        builder_type_path(&builder_name, &builder_generic_args, &initial_state_args);
    let build_ready_builder_ty =
        builder_type_path(&builder_name, &builder_generic_args, &set_state_args);
    let value_builder_ty = value_builder_type(
        &builder_name,
        &builder_generic_args,
        &required_slots,
        &module_name,
        &value_field_name,
    );
    let value_field_name_str = value_field_name.to_string();
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let builder_module = quote! {
        #[doc(hidden)]
        pub mod #module_name {
            pub struct Empty;
            pub struct Set;
        }
    };
    let builder_struct = render_builder_struct(&input, &builder_name, &slots)?;
    let builder_impl = render_builder_impl(&input, &builder_name, &module_name, &slots)?;
    let build_impl = render_build_impl(&input, &slots, &build_ready_builder_ty, &koruma)?;
    let capture_value_ref_impl = render_capture_value_ref_impl(
        &input,
        &builder_name,
        &module_name,
        &slots,
        &value_field_name,
        inner_type,
        value_field_capture,
        &koruma,
    )?;
    let direct_builder_methods = direct_builder_methods(
        &slots,
        &builder_name,
        &builder_generic_args,
        &required_slots,
        &module_name,
    );

    let value_getter_impl = quote! {
        impl #impl_generics #struct_name #type_generics #where_clause {
            #[doc(hidden)]
            pub fn __koruma_builder() -> #initial_builder_ty {
                #builder_name::new()
            }

            #[doc = concat!(
                "Starts building [`",
                stringify!(#struct_name),
                "`] with the `",
                #value_field_name_str,
                "` value set."
            )]
            pub fn with_value(value: #inner_type) -> #value_builder_ty {
                Self::__koruma_builder().with_value(value)
            }

            #[doc = concat!(
                "Returns the stored `",
                #value_field_name_str,
                "` value captured by `#[koruma(value)]`."
            )]
            pub fn #value_field_name(&self) -> &#value_field_type {
                &self.#value_field_name
            }

            #(#direct_builder_methods)*
        }
    };

    #[cfg(feature = "internal-showcase")]
    let showcase_registration = if let Some(showcase) = showcase_attr {
        let name = &showcase.name;
        let description = &showcase.description;
        let create_closure = &showcase.create;
        let anchor_fn = format_ident!(
            "__koruma_showcase_anchor_{}",
            struct_name.to_string().to_snake_case()
        );
        let input_type = &showcase.input_type;
        let input_type_tokens = match input_type {
            ShowcaseInputType::Text => quote! { #koruma::showcase::InputType::Text },
            ShowcaseInputType::Numeric => quote! { #koruma::showcase::InputType::Numeric },
        };
        let module_tokens = if let Some(module) = showcase.module {
            match module {
                ShowcaseModule::String => quote! { #koruma::showcase::ValidatorModule::String },
                ShowcaseModule::Format => quote! { #koruma::showcase::ValidatorModule::Format },
                ShowcaseModule::Numeric => quote! { #koruma::showcase::ValidatorModule::Numeric },
                ShowcaseModule::Collection => {
                    quote! { #koruma::showcase::ValidatorModule::Collection }
                },
                ShowcaseModule::General => quote! { #koruma::showcase::ValidatorModule::General },
            }
        } else {
            quote! { #koruma::showcase::ValidatorModule::General }
        };
        let showcase_validate_type = match value_field_capture {
            CapturePolicy::CloneInput => quote! { #value_field_type },
            CapturePolicy::Skip => quote! { #inner_type },
        };

        let mut showcase_generics = input.generics.clone();
        let showcase_where_clause = showcase_generics.make_where_clause();
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: ::std::marker::Send + ::std::marker::Sync));
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: #koruma::Validate<#showcase_validate_type>));
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: ::std::fmt::Display));
        #[cfg(feature = "fluent")]
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: ::es_fluent::FluentMessage));

        let (impl_generics, type_generics, where_clause) = showcase_generics.split_for_impl();

        let is_valid_body = match value_field_capture {
            CapturePolicy::CloneInput => quote! {
                #koruma::Validate::validate(self, &self.#value_field_name)
            },
            CapturePolicy::Skip => quote! {
                self.#value_field_name
                    .as_ref()
                    .is_some_and(|value| #koruma::Validate::validate(self, value))
            },
        };

        #[cfg(feature = "fluent")]
        let fluent_methods = quote! {
            fn fluent_string_with(
                &self,
                localize: &mut #koruma::showcase::FluentLocalizer<'_>,
            ) -> String {
                use ::es_fluent::FluentMessage;
                self.to_fluent_string_with(localize)
            }
        };

        #[cfg(not(feature = "fluent"))]
        let fluent_methods = quote! {
            fn fluent_string(&self) -> String {
                "(fluent feature required)".to_string()
            }
        };

        quote! {
            impl #impl_generics #koruma::showcase::DynValidator for #struct_name #type_generics #where_clause {
                fn is_valid(&self) -> bool {
                    #is_valid_body
                }

                fn display_string(&self) -> String {
                    ::std::string::ToString::to_string(self)
                }

                #fluent_methods
            }

            #koruma::inventory::submit! {
                #koruma::showcase::ValidatorShowcase {
                    name: #name,
                    description: #description,
                    input_type: #input_type_tokens,
                    module: #module_tokens,
                    create_validator: |input: &str| -> ::anyhow::Result<Box<dyn #koruma::showcase::DynValidator>> {
                        (#create_closure)(input).map(|v| Box::new(v) as Box<dyn #koruma::showcase::DynValidator>)
                    },
                }
            }

            #[doc(hidden)]
            pub fn #anchor_fn() {}
        }
    } else {
        quote! {}
    };

    #[cfg(not(feature = "internal-showcase"))]
    let showcase_registration = quote! {};

    Ok(quote! {
        #input

        #builder_module
        #builder_struct
        #builder_impl
        #build_impl
        #capture_value_ref_impl
        #value_getter_impl

        #showcase_registration
    })
}

enum BuilderSlot {
    Value(ValueSlot),
    Setter(SetterSlot),
}

struct ValueSlot {
    ident: Ident,
    ty: Type,
    capture: CapturePolicy,
}

struct SetterSlot {
    ident: Ident,
    ty: Type,
    method: Ident,
    requirement: SetterRequirement,
    default: SetterDefault,
    into: bool,
}

enum SetterRequirement {
    Required { state_ident: Ident },
    Optional,
}

impl BuilderSlot {
    fn ident(&self) -> &Ident {
        match self {
            Self::Value(slot) => &slot.ident,
            Self::Setter(slot) => &slot.ident,
        }
    }

    fn ty(&self) -> &Type {
        match self {
            Self::Value(slot) => &slot.ty,
            Self::Setter(slot) => &slot.ty,
        }
    }

    fn is_required(&self) -> bool {
        match self {
            Self::Value(slot) => slot.capture == CapturePolicy::CloneInput,
            Self::Setter(slot) => slot.is_required(),
        }
    }

    fn required_state_ident(&self) -> Option<Ident> {
        match self {
            Self::Value(slot) if slot.capture == CapturePolicy::CloneInput => {
                Some(state_ident_for(&slot.ident))
            },
            Self::Value(_) => None,
            Self::Setter(slot) => slot.required_state_ident().cloned(),
        }
    }
}

impl SetterSlot {
    fn is_required(&self) -> bool {
        matches!(self.requirement, SetterRequirement::Required { .. })
    }

    fn required_state_ident(&self) -> Option<&Ident> {
        match &self.requirement {
            SetterRequirement::Required { state_ident } => Some(state_ident),
            SetterRequirement::Optional => None,
        }
    }
}

fn state_ident_for(ident: &Ident) -> Ident {
    format_ident!("__Koruma{}State", ident.to_string().to_upper_camel_case())
}

fn builder_slots(validator_spec: &koruma_derive_core::ValidatorStructSpec) -> Vec<BuilderSlot> {
    validator_spec
        .fields
        .iter()
        .map(|field| {
            let ident = field.name.clone();
            match &field.role {
                ValidatorFieldRole::Value(value) => BuilderSlot::Value(ValueSlot {
                    ident,
                    ty: field.ty.clone(),
                    capture: value.capture,
                }),
                ValidatorFieldRole::Setter(setter) => {
                    let field_is_option = option_inner_type(&field.ty).is_some();
                    let required = setter.required
                        || (!field_is_option && matches!(setter.default, SetterDefault::None));
                    let requirement = if required {
                        SetterRequirement::Required {
                            state_ident: state_ident_for(&ident),
                        }
                    } else {
                        SetterRequirement::Optional
                    };

                    BuilderSlot::Setter(SetterSlot {
                        ident,
                        ty: field.ty.clone(),
                        method: setter.method.clone(),
                        requirement,
                        default: setter.default.clone(),
                        into: setter.into,
                    })
                },
            }
        })
        .collect()
}

fn reject_builder_attrs(field: &Field) -> Result<(), syn::Error> {
    if let Some(attr) = field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("builder"))
    {
        return Err(syn::Error::new_spanned(
            attr,
            "`#[builder(...)]` is not valid on koruma validator fields; use `#[koruma(setter(...))]` for generated setter options",
        ));
    }

    Ok(())
}

fn generic_args(generics: &syn::Generics) -> Vec<TokenStream2> {
    generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                quote! { #lifetime }
            },
            GenericParam::Type(param) => {
                let ident = &param.ident;
                quote! { #ident }
            },
            GenericParam::Const(param) => {
                let ident = &param.ident;
                quote! { #ident }
            },
        })
        .collect()
}

fn builder_type_path(
    builder_name: &Ident,
    generic_args: &[TokenStream2],
    state_args: &[TokenStream2],
) -> TokenStream2 {
    let args: Vec<_> = generic_args.iter().chain(state_args.iter()).collect();
    if args.is_empty() {
        quote! { #builder_name }
    } else {
        quote! { #builder_name<#(#args),*> }
    }
}

fn value_builder_type(
    builder_name: &Ident,
    generic_args: &[TokenStream2],
    required_slots: &[&BuilderSlot],
    module_name: &Ident,
    value_field_name: &Ident,
) -> TokenStream2 {
    let state_args: Vec<_> = required_slots
        .iter()
        .map(|slot| {
            if slot.ident() == value_field_name {
                quote! { #module_name::Set }
            } else {
                quote! { #module_name::Empty }
            }
        })
        .collect();

    builder_type_path(builder_name, generic_args, &state_args)
}

fn render_builder_struct(
    input: &ItemStruct,
    builder_name: &Ident,
    slots: &[BuilderSlot],
) -> Result<TokenStream2, syn::Error> {
    let mut builder_generics = input.generics.clone();
    for state_ident in slots.iter().filter_map(BuilderSlot::required_state_ident) {
        builder_generics.params.push(parse_quote!(#state_ident));
    }
    let field_defs: Vec<_> = slots
        .iter()
        .map(|slot| {
            let ident = slot.ident();
            let ty = slot.ty();
            quote! { #ident: ::std::option::Option<#ty> }
        })
        .collect();
    let state_idents: Vec<_> = slots
        .iter()
        .filter_map(BuilderSlot::required_state_ident)
        .collect();
    let state_marker = if state_idents.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#state_idents),*) }
    };

    Ok(quote! {
        pub struct #builder_name #builder_generics {
            #(#field_defs,)*
            _state: ::std::marker::PhantomData<#state_marker>,
        }
    })
}

fn render_builder_impl(
    input: &ItemStruct,
    builder_name: &Ident,
    module_name: &Ident,
    slots: &[BuilderSlot],
) -> Result<TokenStream2, syn::Error> {
    let mut builder_generics = input.generics.clone();
    for state_ident in slots.iter().filter_map(BuilderSlot::required_state_ident) {
        builder_generics.params.push(parse_quote!(#state_ident));
    }
    let (impl_generics, builder_ty_generics, where_clause) = builder_generics.split_for_impl();
    let generic_args = generic_args(&input.generics);
    let initial_fields: Vec<_> = slots
        .iter()
        .map(|slot| {
            let ident = slot.ident();
            quote! { #ident: ::std::option::Option::None }
        })
        .collect();
    let setter_methods: Vec<_> = slots
        .iter()
        .map(|slot| render_builder_setter(builder_name, module_name, slots, slot, &generic_args))
        .collect();

    Ok(quote! {
        impl #impl_generics #builder_name #builder_ty_generics #where_clause {
            fn new() -> Self {
                Self {
                    #(#initial_fields,)*
                    _state: ::std::marker::PhantomData,
                }
            }

            #(#setter_methods)*
        }
    })
}

fn render_builder_setter(
    builder_name: &Ident,
    module_name: &Ident,
    slots: &[BuilderSlot],
    slot: &BuilderSlot,
    generic_args: &[TokenStream2],
) -> TokenStream2 {
    match slot {
        BuilderSlot::Value(slot) => {
            render_value_setter(builder_name, module_name, slots, slot, generic_args)
        },
        BuilderSlot::Setter(slot) => {
            render_setter_slot(builder_name, module_name, slots, slot, generic_args)
        },
    }
}

fn render_setter_slot(
    builder_name: &Ident,
    module_name: &Ident,
    slots: &[BuilderSlot],
    slot: &SetterSlot,
    generic_args: &[TokenStream2],
) -> TokenStream2 {
    let method = &slot.method;
    let ty = &slot.ty;
    let arg_ty = if slot.into {
        quote! { impl ::std::convert::Into<#ty> }
    } else {
        quote! { #ty }
    };
    let value_expr = if slot.into {
        quote! { ::std::convert::Into::into(value) }
    } else {
        quote! { value }
    };
    let return_ty = if slot.is_required() {
        builder_type_with_replaced_state(
            builder_name,
            generic_args,
            module_name,
            slots,
            &slot.ident,
        )
    } else {
        quote! { Self }
    };
    let assignments = builder_assignments(slots, &slot.ident, quote! { #value_expr });
    let maybe_method = if !slot.is_required() {
        option_inner_type(&slot.ty).map(|inner_ty| {
            let maybe_method = format_ident!("maybe_{}", method);
            quote! {
                pub fn #maybe_method(self, value: ::std::option::Option<#inner_ty>) -> Self {
                    self.#method(value)
                }
            }
        })
    } else {
        None
    };

    quote! {
        pub fn #method(self, value: #arg_ty) -> #return_ty {
            #builder_name {
                #(#assignments,)*
                _state: ::std::marker::PhantomData,
            }
        }

        #maybe_method
    }
}

fn render_value_setter(
    builder_name: &Ident,
    module_name: &Ident,
    slots: &[BuilderSlot],
    slot: &ValueSlot,
    generic_args: &[TokenStream2],
) -> TokenStream2 {
    let method = format_ident!("with_value");
    let inner_ty = option_inner_type(&slot.ty).unwrap_or(&slot.ty);
    let value_expr = if option_inner_type(&slot.ty).is_some() {
        quote! { ::std::option::Option::Some(value) }
    } else {
        quote! { value }
    };
    let return_ty = match slot.capture {
        CapturePolicy::CloneInput => builder_type_with_replaced_state(
            builder_name,
            generic_args,
            module_name,
            slots,
            &slot.ident,
        ),
        CapturePolicy::Skip => quote! { Self },
    };
    let assignments = builder_assignments(slots, &slot.ident, value_expr);

    quote! {
        pub fn #method(self, value: #inner_ty) -> #return_ty {
            #builder_name {
                #(#assignments,)*
                _state: ::std::marker::PhantomData,
            }
        }
    }
}

fn builder_assignments(
    slots: &[BuilderSlot],
    target: &Ident,
    value_expr: TokenStream2,
) -> Vec<TokenStream2> {
    slots
        .iter()
        .map(|slot| {
            let ident = slot.ident();
            if ident == target {
                quote! { #ident: ::std::option::Option::Some(#value_expr) }
            } else {
                quote! { #ident: self.#ident }
            }
        })
        .collect()
}

fn builder_type_with_replaced_state(
    builder_name: &Ident,
    generic_args: &[TokenStream2],
    module_name: &Ident,
    slots: &[BuilderSlot],
    target: &Ident,
) -> TokenStream2 {
    let state_args: Vec<_> = slots
        .iter()
        .filter(|slot| slot.is_required())
        .map(|slot| {
            if slot.ident() == target {
                quote! { #module_name::Set }
            } else {
                let state_ident = slot.required_state_ident().expect("required slot");
                quote! { #state_ident }
            }
        })
        .collect();

    builder_type_path(builder_name, generic_args, &state_args)
}

fn render_build_impl(
    input: &ItemStruct,
    slots: &[BuilderSlot],
    build_ready_builder_ty: &TokenStream2,
    koruma: &TokenStream2,
) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let field_values: Vec<_> = slots
        .iter()
        .map(|slot| {
            let ident = slot.ident();
            let value = build_value_expr(slot);
            quote! { #ident: #value }
        })
        .collect();

    Ok(quote! {
        impl #impl_generics #build_ready_builder_ty #where_clause {
            pub fn build(self) -> #struct_name #type_generics {
                #struct_name {
                    #(#field_values,)*
                }
            }
        }

        impl #impl_generics #koruma::BuildValidator for #build_ready_builder_ty #where_clause {
            type Validator = #struct_name #type_generics;

            fn build_validator(self) -> Self::Validator {
                self.build()
            }
        }
    })
}

fn build_value_expr(slot: &BuilderSlot) -> TokenStream2 {
    match slot {
        BuilderSlot::Value(slot) => {
            let ident = &slot.ident;
            match slot.capture {
                CapturePolicy::CloneInput => quote! {
                    self.#ident.expect("required koruma validator builder field should be set")
                },
                CapturePolicy::Skip => quote! {
                    self.#ident.unwrap_or(::std::option::Option::None)
                },
            }
        },
        BuilderSlot::Setter(slot) => {
            let ident = &slot.ident;
            if slot.is_required() {
                return quote! {
                    self.#ident.expect("required koruma validator builder field should be set")
                };
            }

            match &slot.default {
                SetterDefault::Expr(expr) => quote! {
                    self.#ident.unwrap_or_else(|| #expr)
                },
                SetterDefault::Default => quote! {
                    self.#ident.unwrap_or_default()
                },
                SetterDefault::None => {
                    if option_inner_type(&slot.ty).is_some() {
                        quote! {
                            self.#ident.unwrap_or(::std::option::Option::None)
                        }
                    } else {
                        quote! {
                            self.#ident.expect("defaulted koruma validator builder field should be set")
                        }
                    }
                },
            }
        },
    }
}

fn render_capture_value_ref_impl(
    input: &ItemStruct,
    builder_name: &Ident,
    module_name: &Ident,
    slots: &[BuilderSlot],
    value_field_name: &Ident,
    inner_type: &Type,
    value_field_capture: CapturePolicy,
    koruma: &TokenStream2,
) -> Result<TokenStream2, syn::Error> {
    let mut builder_generics = input.generics.clone();
    let generic_args = generic_args(&input.generics);
    let builder_state_args: Vec<_> = slots
        .iter()
        .filter_map(BuilderSlot::required_state_ident)
        .map(|state_ident| quote! { #state_ident })
        .collect();
    for state_ident in slots.iter().filter_map(BuilderSlot::required_state_ident) {
        builder_generics.params.push(parse_quote!(#state_ident));
    }
    if value_field_capture == CapturePolicy::CloneInput {
        builder_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#inner_type: ::std::clone::Clone));
    }
    let builder_ty = builder_type_path(builder_name, &generic_args, &builder_state_args);

    match value_field_capture {
        CapturePolicy::CloneInput => {
            let output_ty = builder_type_with_replaced_state(
                builder_name,
                &generic_args,
                module_name,
                slots,
                value_field_name,
            );
            let mut capture_generics = builder_generics;
            capture_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#output_ty: #koruma::BuildValidator));
            let (impl_generics, _, where_clause) = capture_generics.split_for_impl();
            Ok(quote! {
                impl #impl_generics #koruma::CaptureValueRef<#inner_type>
                    for #builder_ty #where_clause
                {
                    type Output = #output_ty;

                    fn capture_value_ref(self, value: &#inner_type) -> Self::Output {
                        self.with_value(value.clone())
                    }
                }
            })
        },
        CapturePolicy::Skip => {
            let mut capture_generics = builder_generics;
            capture_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#builder_ty: #koruma::BuildValidator));
            let (impl_generics, _, where_clause) = capture_generics.split_for_impl();
            Ok(quote! {
                impl #impl_generics #koruma::CaptureValueRef<#inner_type>
                    for #builder_ty #where_clause
                {
                    type Output = Self;

                    fn capture_value_ref(self, _value: &#inner_type) -> Self::Output {
                        self
                    }
                }
            })
        },
    }
}

fn direct_builder_methods(
    slots: &[BuilderSlot],
    builder_name: &Ident,
    builder_generic_args: &[TokenStream2],
    required_slots: &[&BuilderSlot],
    module_name: &Ident,
) -> Vec<TokenStream2> {
    slots
        .iter()
        .filter_map(|slot| match slot {
            BuilderSlot::Value(_) => None,
            BuilderSlot::Setter(slot) => Some(slot),
        })
        .map(|slot| {
            render_direct_builder_method(
                slot,
                builder_name,
                builder_generic_args,
                required_slots,
                module_name,
            )
        })
        .collect()
}

fn render_direct_builder_method(
    slot: &SetterSlot,
    builder_name: &Ident,
    builder_generic_args: &[TokenStream2],
    required_slots: &[&BuilderSlot],
    module_name: &Ident,
) -> TokenStream2 {
    let method = &slot.method;
    let ty = &slot.ty;
    let arg_ty = if slot.into {
        quote! { impl ::std::convert::Into<#ty> }
    } else {
        quote! { #ty }
    };
    let state_args: Vec<_> = required_slots
        .iter()
        .map(|required| {
            if required.ident() == &slot.ident {
                quote! { #module_name::Set }
            } else {
                quote! { #module_name::Empty }
            }
        })
        .collect();
    let output_builder_ty = if slot.is_required() {
        builder_type_path(builder_name, builder_generic_args, &state_args)
    } else {
        let empty_state_args: Vec<_> = required_slots
            .iter()
            .map(|_| quote! { #module_name::Empty })
            .collect();
        builder_type_path(builder_name, builder_generic_args, &empty_state_args)
    };
    let method_name_str = method.to_string();

    let maybe_method = if !slot.is_required() {
        option_inner_type(&slot.ty).map(|inner_ty| {
            let maybe_method = format_ident!("maybe_{}", method);
            let maybe_method_name_str = maybe_method.to_string();
            quote! {
                #[doc = concat!(
                    "Starts building this validator with `",
                    #maybe_method_name_str,
                    "` set."
                )]
                pub fn #maybe_method(value: ::std::option::Option<#inner_ty>) -> #output_builder_ty {
                    Self::__koruma_builder().#maybe_method(value)
                }
            }
        })
    } else {
        None
    };

    quote! {
        #[doc = concat!(
            "Starts building this validator with `",
            #method_name_str,
            "` set."
        )]
        pub fn #method(value: #arg_ty) -> #output_builder_ty {
            Self::__koruma_builder().#method(value)
        }

        #maybe_method
    }
}
