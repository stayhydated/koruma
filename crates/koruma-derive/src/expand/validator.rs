use super::koruma_crate_path;
#[cfg(feature = "internal-showcase")]
use super::{ShowcaseInputType, ShowcaseModule};
use heck::{ToSnakeCase, ToUpperCamelCase};
#[cfg(feature = "internal-showcase")]
use koruma_derive_core::find_showcase_attr;
use koruma_derive_core::{
    CapturePolicy, SetterDefault, ValidatorFieldRole, option_inner_type, parse_validator_struct,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
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

    let validator_spec = parse_validator_struct(&input)?;
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

    let slots = builder_slots(&validator_spec, &input.generics)?;

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
    CapturedValue(CapturedValueSlot),
    SkippedValue(SkippedValueSlot),
    RequiredSetter(RequiredSetterSlot),
    OptionalSetter(OptionalSetterSlot),
    DefaultedSetter(DefaultedSetterSlot),
}

struct CapturedValueSlot {
    ident: Ident,
    ty: Type,
    state_ident: Ident,
}

struct SkippedValueSlot {
    ident: Ident,
    ty: Type,
}

struct RequiredSetterSlot {
    ident: Ident,
    ty: Type,
    method: Ident,
    into: bool,
    state_ident: Ident,
}

struct OptionalSetterSlot {
    ident: Ident,
    ty: Type,
    method: Ident,
    into: bool,
}

struct DefaultedSetterSlot {
    ident: Ident,
    ty: Type,
    method: Ident,
    into: bool,
    default: SetterDefaultValue,
}

enum SetterDefaultValue {
    Default,
    Expr(syn::Expr),
}

struct SetterRenderParts<'a> {
    ident: &'a Ident,
    ty: &'a Type,
    method: &'a Ident,
    into: bool,
    required: bool,
}

impl BuilderSlot {
    fn ident(&self) -> &Ident {
        match self {
            Self::CapturedValue(slot) => &slot.ident,
            Self::SkippedValue(slot) => &slot.ident,
            Self::RequiredSetter(slot) => &slot.ident,
            Self::OptionalSetter(slot) => &slot.ident,
            Self::DefaultedSetter(slot) => &slot.ident,
        }
    }

    fn ty(&self) -> &Type {
        match self {
            Self::CapturedValue(slot) => &slot.ty,
            Self::SkippedValue(slot) => &slot.ty,
            Self::RequiredSetter(slot) => &slot.ty,
            Self::OptionalSetter(slot) => &slot.ty,
            Self::DefaultedSetter(slot) => &slot.ty,
        }
    }

    fn is_required(&self) -> bool {
        matches!(self, Self::CapturedValue(_) | Self::RequiredSetter(_))
    }

    fn required_state_ident(&self) -> Option<Ident> {
        match self {
            Self::CapturedValue(slot) => Some(slot.state_ident.clone()),
            Self::RequiredSetter(slot) => Some(slot.state_ident.clone()),
            Self::SkippedValue(_) | Self::OptionalSetter(_) | Self::DefaultedSetter(_) => None,
        }
    }

    fn setter_method(&self) -> Option<&Ident> {
        match self {
            Self::RequiredSetter(slot) => Some(&slot.method),
            Self::OptionalSetter(slot) => Some(&slot.method),
            Self::DefaultedSetter(slot) => Some(&slot.method),
            Self::CapturedValue(_) | Self::SkippedValue(_) => None,
        }
    }

    fn setter_render_parts(&self) -> Option<SetterRenderParts<'_>> {
        match self {
            Self::RequiredSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                ty: &slot.ty,
                method: &slot.method,
                into: slot.into,
                required: true,
            }),
            Self::OptionalSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                ty: &slot.ty,
                method: &slot.method,
                into: slot.into,
                required: false,
            }),
            Self::DefaultedSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                ty: &slot.ty,
                method: &slot.method,
                into: slot.into,
                required: false,
            }),
            Self::CapturedValue(_) | Self::SkippedValue(_) => None,
        }
    }
}

fn state_ident_for(ident: &Ident) -> Ident {
    format_ident!("__Koruma{}State", ident.to_string().to_upper_camel_case())
}

fn builder_slots(
    validator_spec: &koruma_derive_core::ValidatorStructSpec,
    generics: &syn::Generics,
) -> Result<Vec<BuilderSlot>, syn::Error> {
    let user_generic_names: HashSet<String> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(param) => Some(param.ident.to_string()),
            GenericParam::Const(param) => Some(param.ident.to_string()),
            GenericParam::Lifetime(_) => None,
        })
        .collect();
    let mut slots = Vec::new();

    for field in &validator_spec.fields {
        if field.name == "_state" {
            return Err(syn::Error::new(
                field.name.span(),
                "validator field name `_state` is reserved for the generated builder",
            ));
        }

        let ident = field.name.clone();
        let slot = match &field.role {
            ValidatorFieldRole::Value(value) => match value.capture {
                CapturePolicy::CloneInput => {
                    let state_ident = state_ident_for(&ident);
                    reject_state_ident_collision(&state_ident, &user_generic_names)?;
                    BuilderSlot::CapturedValue(CapturedValueSlot {
                        ident,
                        ty: field.ty.clone(),
                        state_ident,
                    })
                },
                CapturePolicy::Skip => BuilderSlot::SkippedValue(SkippedValueSlot {
                    ident,
                    ty: field.ty.clone(),
                }),
            },
            ValidatorFieldRole::Setter(setter) => {
                let field_is_option = option_inner_type(&field.ty).is_some();
                let required = setter.required
                    || (!field_is_option && matches!(setter.default, SetterDefault::None));

                if required {
                    let state_ident = state_ident_for(&ident);
                    reject_state_ident_collision(&state_ident, &user_generic_names)?;
                    BuilderSlot::RequiredSetter(RequiredSetterSlot {
                        ident,
                        ty: field.ty.clone(),
                        method: setter.method.clone(),
                        into: setter.into,
                        state_ident,
                    })
                } else if matches!(setter.default, SetterDefault::None) {
                    BuilderSlot::OptionalSetter(OptionalSetterSlot {
                        ident,
                        ty: field.ty.clone(),
                        method: setter.method.clone(),
                        into: setter.into,
                    })
                } else if let Some(default) = setter_default_value(&setter.default) {
                    BuilderSlot::DefaultedSetter(DefaultedSetterSlot {
                        ident,
                        ty: field.ty.clone(),
                        method: setter.method.clone(),
                        into: setter.into,
                        default,
                    })
                } else {
                    BuilderSlot::OptionalSetter(OptionalSetterSlot {
                        ident,
                        ty: field.ty.clone(),
                        method: setter.method.clone(),
                        into: setter.into,
                    })
                }
            },
        };
        slots.push(slot);
    }

    reject_generated_method_collisions(&slots)?;
    Ok(slots)
}

fn setter_default_value(default: &SetterDefault) -> Option<SetterDefaultValue> {
    match default {
        SetterDefault::None => None,
        SetterDefault::Default => Some(SetterDefaultValue::Default),
        SetterDefault::Expr(expr) => Some(SetterDefaultValue::Expr(expr.clone())),
    }
}

fn reject_state_ident_collision(
    state_ident: &Ident,
    user_generic_names: &HashSet<String>,
) -> Result<(), syn::Error> {
    if user_generic_names.contains(&state_ident.to_string()) {
        return Err(syn::Error::new(
            state_ident.span(),
            format!(
                "generated required-state generic `{state_ident}` collides with a user generic"
            ),
        ));
    }
    Ok(())
}

fn reject_generated_method_collisions(slots: &[BuilderSlot]) -> Result<(), syn::Error> {
    let reserved = ["with_value", "build", "__koruma_builder"];
    let mut direct_methods: HashMap<String, Ident> = HashMap::new();

    for slot in slots {
        let Some(method) = slot.setter_method() else {
            continue;
        };
        let method_name = method.to_string();
        if reserved.contains(&method_name.as_str()) {
            return Err(syn::Error::new(
                method.span(),
                format!("setter method name `{method_name}` is reserved by koruma"),
            ));
        }
        if let Some(first) = direct_methods.insert(method_name.clone(), method.clone()) {
            return Err(syn::Error::new(
                method.span(),
                format!("setter method `{method_name}` collides with another setter `{first}`"),
            ));
        }
    }

    for slot in slots {
        let BuilderSlot::OptionalSetter(slot) = slot else {
            continue;
        };
        if option_inner_type(&slot.ty).is_none() {
            continue;
        }
        let maybe_method = format_ident!("maybe_{}", slot.method);
        let maybe_name = maybe_method.to_string();
        if let Some(first) = direct_methods.get(&maybe_name) {
            return Err(syn::Error::new(
                slot.method.span(),
                format!(
                    "generated optional setter method `{maybe_name}` collides with setter `{first}`"
                ),
            ));
        }
    }

    Ok(())
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
        BuilderSlot::CapturedValue(slot) => render_value_setter(
            builder_name,
            module_name,
            slots,
            &slot.ident,
            &slot.ty,
            true,
            generic_args,
        ),
        BuilderSlot::SkippedValue(slot) => render_value_setter(
            builder_name,
            module_name,
            slots,
            &slot.ident,
            &slot.ty,
            false,
            generic_args,
        ),
        BuilderSlot::RequiredSetter(_)
        | BuilderSlot::OptionalSetter(_)
        | BuilderSlot::DefaultedSetter(_) => {
            if let Some(parts) = slot.setter_render_parts() {
                render_setter_slot(builder_name, module_name, slots, parts, generic_args)
            } else {
                quote! {}
            }
        },
    }
}

fn render_setter_slot(
    builder_name: &Ident,
    module_name: &Ident,
    slots: &[BuilderSlot],
    slot: SetterRenderParts<'_>,
    generic_args: &[TokenStream2],
) -> TokenStream2 {
    let method = slot.method;
    let ty = slot.ty;
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
    let return_ty = if slot.required {
        builder_type_with_replaced_state(builder_name, generic_args, module_name, slots, slot.ident)
    } else {
        quote! { Self }
    };
    let assignments = builder_assignments(slots, slot.ident, quote! { #value_expr });
    let maybe_method = if !slot.required {
        option_inner_type(slot.ty).map(|inner_ty| {
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
    ident: &Ident,
    ty: &Type,
    capture_required: bool,
    generic_args: &[TokenStream2],
) -> TokenStream2 {
    let method = format_ident!("with_value");
    let inner_ty = option_inner_type(ty).unwrap_or(ty);
    let value_expr = if option_inner_type(ty).is_some() {
        quote! { ::std::option::Option::Some(value) }
    } else {
        quote! { value }
    };
    let return_ty = if capture_required {
        builder_type_with_replaced_state(builder_name, generic_args, module_name, slots, ident)
    } else {
        quote! { Self }
    };
    let assignments = builder_assignments(slots, ident, value_expr);

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
            } else if let Some(state_ident) = slot.required_state_ident() {
                quote! { #state_ident }
            } else {
                quote! {}
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
        BuilderSlot::CapturedValue(slot) => {
            let ident = &slot.ident;
            quote! {
                self.#ident.expect("required koruma validator builder field should be set")
            }
        },
        BuilderSlot::SkippedValue(slot) => {
            let ident = &slot.ident;
            quote! {
                self.#ident.unwrap_or(::std::option::Option::None)
            }
        },
        BuilderSlot::RequiredSetter(slot) => {
            let ident = &slot.ident;
            quote! {
                self.#ident.expect("required koruma validator builder field should be set")
            }
        },
        BuilderSlot::OptionalSetter(slot) => {
            let ident = &slot.ident;
            quote! {
                self.#ident.unwrap_or(::std::option::Option::None)
            }
        },
        BuilderSlot::DefaultedSetter(slot) => {
            let ident = &slot.ident;
            match &slot.default {
                SetterDefaultValue::Expr(expr) => quote! {
                    self.#ident.unwrap_or_else(|| #expr)
                },
                SetterDefaultValue::Default => quote! {
                    self.#ident.unwrap_or_default()
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
        .filter_map(BuilderSlot::setter_render_parts)
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
    slot: SetterRenderParts<'_>,
    builder_name: &Ident,
    builder_generic_args: &[TokenStream2],
    required_slots: &[&BuilderSlot],
    module_name: &Ident,
) -> TokenStream2 {
    let method = slot.method;
    let ty = slot.ty;
    let arg_ty = if slot.into {
        quote! { impl ::std::convert::Into<#ty> }
    } else {
        quote! { #ty }
    };
    let state_args: Vec<_> = required_slots
        .iter()
        .map(|required| {
            if required.ident() == slot.ident {
                quote! { #module_name::Set }
            } else {
                quote! { #module_name::Empty }
            }
        })
        .collect();
    let output_builder_ty = if slot.required {
        builder_type_path(builder_name, builder_generic_args, &state_args)
    } else {
        let empty_state_args: Vec<_> = required_slots
            .iter()
            .map(|_| quote! { #module_name::Empty })
            .collect();
        builder_type_path(builder_name, builder_generic_args, &empty_state_args)
    };
    let method_name_str = method.to_string();

    let maybe_method = if !slot.required {
        option_inner_type(slot.ty).map(|inner_ty| {
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
