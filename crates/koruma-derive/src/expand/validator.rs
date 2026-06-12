use super::koruma_crate_path;
#[cfg(feature = "internal-showcase")]
use super::{ShowcaseInputType, ShowcaseModule};
use heck::ToSnakeCase as _;
#[cfg(feature = "internal-showcase")]
use koruma_derive_core::find_showcase_attr;
use koruma_derive_core::{
    CapturePolicy, SetterDefault, SetterInputPolicy, SetterPresence, ValidatorFieldRole,
    ValidatorStructSpec, option_inner_type, parse_validator_struct,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Fields, GenericParam, Ident, ItemStruct, Type, Visibility, parse_quote};

use super::generated_api::{
    GeneratedApiNameKind, GeneratedApiNamespace, RegisteredApiName, builder_method_namespace,
    reserved_builder_method_name, state_ident_for, user_generic_namespace,
};

/// Core expansion logic for the `#[validator]` attribute macro.
///
/// Takes a parsed struct and returns the expanded TokenStream.
pub fn expand_validator(mut input: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
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
    let value_field_name = validator_spec.value_field().name().clone();

    input.attrs.retain(|attr| !attr.path().is_ident("showcase"));

    let Fields::Named(ref fields) = input.fields else {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "koruma::validator only supports structs with named fields",
        ));
    };
    for field in &fields.named {
        if field.ident.as_ref() == Some(&value_field_name)
            && !matches!(field.vis, Visibility::Inherited)
        {
            return Err(syn::Error::new_spanned(
                &field.vis,
                format!(
                    "validator value field `{}` must be private; use the generated getter instead",
                    value_field_name
                ),
            ));
        }
    }

    let builder_plan = ValidatorBuilderPlan::build(&input, &validator_spec)?;
    let value_field_name = builder_plan.value_slot().ident().clone();
    let value_field_type = builder_plan.value_slot().ty().clone();
    let inner_type = builder_plan.value_inner_type().clone();
    #[cfg(feature = "internal-showcase")]
    let value_field_capture = builder_plan.capture_policy();

    let Fields::Named(ref mut fields) = input.fields else {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "koruma::validator only supports structs with named fields",
        ));
    };
    for field in &mut fields.named {
        field.attrs.retain(|attr| !attr.path().is_ident("koruma"));
    }
    let initial_state_args = builder_plan.initial_state_args();
    let set_state_args = builder_plan.set_state_args();
    let initial_builder_ty = builder_plan.builder_type_path_with_states(&initial_state_args);
    let build_ready_builder_ty = builder_plan.builder_type_path_with_states(&set_state_args);
    let value_builder_ty = builder_plan.value_builder_type();
    let value_field_name_str = value_field_name.to_string();
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let module_name = &builder_plan.module_name;
    let builder_module = quote! {
        #[doc(hidden)]
        pub mod #module_name {
            pub struct Empty;
            pub struct Set;
        }
    };
    let builder_struct = render_builder_struct(&builder_plan)?;
    let builder_impl = render_builder_impl(&builder_plan)?;
    let build_impl = render_build_impl(&builder_plan, &build_ready_builder_ty, &koruma)?;
    let capture_value_ref_impl = render_capture_value_ref_impl(&builder_plan, &koruma)?;
    let direct_builder_methods = direct_builder_methods(&builder_plan);
    let builder_name = &builder_plan.builder_name;

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
                "` value captured during validation."
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

pub(crate) struct ValidatorBuilderPlan {
    struct_name: Ident,
    input: ItemStruct,
    builder_name: Ident,
    module_name: Ident,
    slots: Vec<BuilderSlot>,
    direct_methods: Vec<DirectBuilderMethodPlan>,
    generic_args: Vec<TokenStream2>,
    capture_policy: CapturePolicy,
    value_slot_index: usize,
}

impl ValidatorBuilderPlan {
    fn build(input: &ItemStruct, validator_spec: &ValidatorStructSpec) -> Result<Self, syn::Error> {
        let struct_name = input.ident.clone();
        let builder_name = format_ident!("{}Builder", struct_name);
        let module_name = format_ident!("{}_builder", struct_name.to_string().to_snake_case());
        reject_builder_helper_name_collisions(&builder_name, &module_name)?;
        let slots = builder_slots(validator_spec, &input.generics)?;
        let direct_methods = direct_method_plans(&slots);
        let capture_policy = validator_spec.value_spec().capture();
        let value_slot_index = validator_spec.value_index();
        let Some(value_slot) = slots.get(value_slot_index) else {
            return Err(syn::Error::new_spanned(
                input,
                "internal error: validator value field did not produce a builder slot",
            ));
        };

        if capture_policy == CapturePolicy::Skip && option_inner_type(value_slot.ty()).is_none() {
            return Err(syn::Error::new_spanned(
                value_slot.ty(),
                "`#[koruma(skip_capture)]` requires an `Option<T>` field",
            ));
        }

        Ok(Self {
            struct_name,
            input: input.clone(),
            builder_name,
            module_name,
            slots,
            direct_methods,
            generic_args: generic_args(&input.generics),
            capture_policy,
            value_slot_index,
        })
    }

    fn slots(&self) -> &[BuilderSlot] {
        &self.slots
    }

    fn direct_methods(&self) -> &[DirectBuilderMethodPlan] {
        &self.direct_methods
    }

    fn value_slot(&self) -> &BuilderSlot {
        &self.slots[self.value_slot_index]
    }

    fn value_inner_type(&self) -> &Type {
        option_inner_type(self.value_slot().ty()).unwrap_or(self.value_slot().ty())
    }

    fn required_slots(&self) -> Vec<&BuilderSlot> {
        self.slots
            .iter()
            .filter(|slot| slot.is_required())
            .collect()
    }

    fn initial_state_args(&self) -> Vec<TokenStream2> {
        self.required_slots()
            .iter()
            .map(|_| {
                let module_name = &self.module_name;
                quote! { #module_name::Empty }
            })
            .collect()
    }

    fn set_state_args(&self) -> Vec<TokenStream2> {
        self.required_slots()
            .iter()
            .map(|_| {
                let module_name = &self.module_name;
                quote! { #module_name::Set }
            })
            .collect()
    }

    fn builder_type_path_with_states(&self, state_args: &[TokenStream2]) -> TokenStream2 {
        builder_type_path(&self.builder_name, &self.generic_args, state_args)
    }

    fn value_builder_type(&self) -> TokenStream2 {
        let value_field_name = self.value_slot().ident();
        let state_args: Vec<_> = self
            .required_slots()
            .iter()
            .map(|slot| {
                let module_name = &self.module_name;
                if slot.ident() == value_field_name {
                    quote! { #module_name::Set }
                } else {
                    quote! { #module_name::Empty }
                }
            })
            .collect();

        self.builder_type_path_with_states(&state_args)
    }

    fn builder_type_with_replaced_state(&self, target: &Ident) -> TokenStream2 {
        builder_type_with_replaced_state(
            &self.builder_name,
            &self.generic_args,
            &self.module_name,
            &self.slots,
            target,
        )
    }

    #[cfg(test)]
    pub(crate) fn test_build(input: &ItemStruct) -> Result<Self, syn::Error> {
        let validator_spec = parse_validator_struct(input)?;
        Self::build(input, &validator_spec)
    }

    #[cfg(test)]
    pub(crate) fn slot_summaries(&self) -> Vec<BuilderSlotSummary> {
        self.slots
            .iter()
            .map(BuilderSlotSummary::for_slot)
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn direct_method_summaries(&self) -> Vec<(String, Option<String>)> {
        self.direct_methods
            .iter()
            .map(|method| {
                (
                    method.setter.method.to_string(),
                    method.maybe_method.as_ref().map(ToString::to_string),
                )
            })
            .collect()
    }

    #[cfg(any(test, feature = "internal-showcase"))]
    pub(crate) fn capture_policy(&self) -> CapturePolicy {
        self.capture_policy
    }
}

struct DirectBuilderMethodPlan {
    setter: OwnedSetterRenderPlan,
    maybe_method: Option<Ident>,
}

fn direct_method_plans(slots: &[BuilderSlot]) -> Vec<DirectBuilderMethodPlan> {
    slots
        .iter()
        .filter_map(|slot| {
            let parts = slot.setter_render_parts()?;
            let maybe_method = parts
                .maybe_inner_ty
                .map(|_| format_ident!("maybe_{}", parts.method));
            let setter = OwnedSetterRenderPlan::from(parts);
            Some(DirectBuilderMethodPlan {
                setter,
                maybe_method,
            })
        })
        .collect()
}

#[cfg(test)]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct BuilderSlotSummary {
    pub name: String,
    pub kind: &'static str,
    pub required: bool,
    pub state_ident: Option<String>,
    pub method: Option<String>,
    pub signature: Option<String>,
    pub maybe_method: Option<String>,
}

#[cfg(test)]
impl BuilderSlotSummary {
    fn for_slot(slot: &BuilderSlot) -> Self {
        let method = slot.setter_method().map(ToString::to_string);
        let maybe_method = match slot {
            BuilderSlot::OptionalSetter(slot) if slot.maybe_inner_ty.is_some() => {
                Some(format!("maybe_{}", slot.method))
            },
            _ => None,
        };

        Self {
            name: slot.ident().to_string(),
            kind: slot.kind_name(),
            required: slot.is_required(),
            state_ident: slot.required_state_ident().map(|ident| ident.to_string()),
            method,
            signature: slot.setter_signature().map(signature_summary),
            maybe_method,
        }
    }
}

#[cfg(test)]
fn signature_summary(signature: &SetterSignature) -> String {
    match signature {
        SetterSignature::Exact(ty) => format!("exact({})", quote!(#ty)),
        SetterSignature::Into(ty) => format!("into({})", quote!(#ty)),
        SetterSignature::OptionalInner { inner, into } => {
            format!("optional_inner({}, into={into})", quote!(#inner))
        },
        SetterSignature::OptionalExact { option_ty } => {
            format!("optional_exact({})", quote!(#option_ty))
        },
    }
}

enum BuilderSlot {
    CapturedValue(CapturedValueSlot),
    SkippedValue(SkippedValueSlot),
    RequiredSetter(RequiredSetterSlot),
    OptionalSetter(Box<OptionalSetterSlot>),
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
    signature: SetterSignature,
    state_ident: Ident,
}

struct OptionalSetterSlot {
    ident: Ident,
    ty: Type,
    method: Ident,
    signature: SetterSignature,
    maybe_inner_ty: Option<Type>,
}

struct DefaultedSetterSlot {
    ident: Ident,
    ty: Type,
    method: Ident,
    signature: SetterSignature,
    default: SetterDefaultValue,
}

enum SetterDefaultValue {
    Default,
    Expr(Box<syn::Expr>),
}

struct SetterRenderParts<'a> {
    ident: &'a Ident,
    method: &'a Ident,
    signature: &'a SetterSignature,
    required: bool,
    maybe_inner_ty: Option<&'a Type>,
}

struct OwnedSetterRenderPlan {
    ident: Ident,
    method: Ident,
    signature: SetterSignature,
    required: bool,
    maybe_inner_ty: Option<Type>,
}

impl From<SetterRenderParts<'_>> for OwnedSetterRenderPlan {
    fn from(parts: SetterRenderParts<'_>) -> Self {
        Self {
            ident: parts.ident.clone(),
            method: parts.method.clone(),
            signature: parts.signature.clone(),
            required: parts.required,
            maybe_inner_ty: parts.maybe_inner_ty.cloned(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SetterSignature {
    Exact(Type),
    Into(Type),
    OptionalInner { inner: Type, into: bool },
    OptionalExact { option_ty: Type },
}

enum ValidatorSlotSpec {
    CapturedValue {
        ident: Ident,
        ty: Type,
    },
    SkippedValue {
        ident: Ident,
        ty: Type,
    },
    RequiredSetter {
        ident: Ident,
        ty: Type,
        method: Ident,
        input: SetterInputPolicy,
    },
    OptionalSetter {
        ident: Ident,
        ty: Type,
        method: Ident,
        input: SetterInputPolicy,
    },
    DefaultedSetter {
        ident: Ident,
        ty: Type,
        method: Ident,
        input: SetterInputPolicy,
        default: SetterDefault,
    },
}

impl ValidatorSlotSpec {
    fn for_field(field: &koruma_derive_core::ValidatorFieldSpec) -> Self {
        let ident = field.name().clone();
        let ty = field.ty().clone();

        match field.role() {
            ValidatorFieldRole::Value(value) => match value.capture() {
                CapturePolicy::CloneInput => Self::CapturedValue { ident, ty },
                CapturePolicy::Skip => Self::SkippedValue { ident, ty },
            },
            ValidatorFieldRole::Setter(setter) => match setter.presence() {
                SetterPresence::Required => Self::RequiredSetter {
                    ident,
                    ty,
                    method: setter.method().clone(),
                    input: setter.input(),
                },
                SetterPresence::Defaulted(default) => Self::DefaultedSetter {
                    ident,
                    ty,
                    method: setter.method().clone(),
                    input: setter.input(),
                    default: default.clone(),
                },
                SetterPresence::Optional if option_inner_type(field.ty()).is_some() => {
                    Self::OptionalSetter {
                        ident,
                        ty,
                        method: setter.method().clone(),
                        input: setter.input(),
                    }
                },
                SetterPresence::Optional => Self::RequiredSetter {
                    ident,
                    ty,
                    method: setter.method().clone(),
                    input: setter.input(),
                },
            },
        }
    }
}

impl BuilderSlot {
    #[cfg(test)]
    fn kind_name(&self) -> &'static str {
        match self {
            Self::CapturedValue(_) => "captured_value",
            Self::SkippedValue(_) => "skipped_value",
            Self::RequiredSetter(_) => "required_setter",
            Self::OptionalSetter(_) => "optional_setter",
            Self::DefaultedSetter(_) => "defaulted_setter",
        }
    }

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

    #[cfg(test)]
    fn setter_signature(&self) -> Option<&SetterSignature> {
        match self {
            Self::RequiredSetter(slot) => Some(&slot.signature),
            Self::OptionalSetter(slot) => Some(&slot.signature),
            Self::DefaultedSetter(slot) => Some(&slot.signature),
            Self::CapturedValue(_) | Self::SkippedValue(_) => None,
        }
    }

    fn setter_render_parts(&self) -> Option<SetterRenderParts<'_>> {
        match self {
            Self::RequiredSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                method: &slot.method,
                signature: &slot.signature,
                required: true,
                maybe_inner_ty: None,
            }),
            Self::OptionalSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                method: &slot.method,
                signature: &slot.signature,
                required: false,
                maybe_inner_ty: slot.maybe_inner_ty.as_ref(),
            }),
            Self::DefaultedSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                method: &slot.method,
                signature: &slot.signature,
                required: false,
                maybe_inner_ty: None,
            }),
            Self::CapturedValue(_) | Self::SkippedValue(_) => None,
        }
    }
}

fn builder_slots(
    validator_spec: &koruma_derive_core::ValidatorStructSpec,
    generics: &syn::Generics,
) -> Result<Vec<BuilderSlot>, syn::Error> {
    let mut generated_names = user_generic_namespace(generics);
    let mut slots = Vec::new();

    for field in validator_spec.fields() {
        if field.name() == "_state" {
            return Err(syn::Error::new(
                field.name().span(),
                "validator field name `_state` is reserved for the generated builder",
            ));
        }

        let slot = match ValidatorSlotSpec::for_field(field) {
            ValidatorSlotSpec::CapturedValue { ident, ty } => {
                let state_ident = state_ident_for(&ident);
                reject_state_ident_collision(&mut generated_names, &state_ident)?;
                BuilderSlot::CapturedValue(CapturedValueSlot {
                    ident,
                    ty,
                    state_ident,
                })
            },
            ValidatorSlotSpec::SkippedValue { ident, ty } => {
                BuilderSlot::SkippedValue(SkippedValueSlot { ident, ty })
            },
            ValidatorSlotSpec::RequiredSetter {
                ident,
                ty,
                method,
                input,
            } => {
                let state_ident = state_ident_for(&ident);
                reject_state_ident_collision(&mut generated_names, &state_ident)?;
                let signature = required_setter_signature(&ty, input);
                BuilderSlot::RequiredSetter(RequiredSetterSlot {
                    ident,
                    ty,
                    method,
                    signature,
                    state_ident,
                })
            },
            ValidatorSlotSpec::OptionalSetter {
                ident,
                ty,
                method,
                input,
            } => {
                let (signature, maybe_inner_ty) = optional_setter_signature(&ty, input);
                BuilderSlot::OptionalSetter(Box::new(OptionalSetterSlot {
                    ident,
                    ty,
                    method,
                    signature,
                    maybe_inner_ty,
                }))
            },
            ValidatorSlotSpec::DefaultedSetter {
                ident,
                ty,
                method,
                input,
                default,
            } => {
                let signature = defaulted_setter_signature(&ty, input);
                BuilderSlot::DefaultedSetter(DefaultedSetterSlot {
                    ident,
                    ty,
                    method,
                    signature,
                    default: setter_default_value(&default),
                })
            },
        };
        slots.push(slot);
    }

    reject_generated_method_collisions(&slots)?;
    Ok(slots)
}

fn required_setter_signature(ty: &Type, input: SetterInputPolicy) -> SetterSignature {
    if option_inner_type(ty).is_some() {
        SetterSignature::OptionalExact {
            option_ty: ty.clone(),
        }
    } else if input.accepts_into() {
        SetterSignature::Into(ty.clone())
    } else {
        SetterSignature::Exact(ty.clone())
    }
}

fn optional_setter_signature(
    ty: &Type,
    input: SetterInputPolicy,
) -> (SetterSignature, Option<Type>) {
    if let Some(inner) = option_inner_type(ty) {
        (
            SetterSignature::OptionalInner {
                inner: inner.clone(),
                into: input.accepts_into(),
            },
            Some(inner.clone()),
        )
    } else if input.accepts_into() {
        (SetterSignature::Into(ty.clone()), None)
    } else {
        (SetterSignature::Exact(ty.clone()), None)
    }
}

fn defaulted_setter_signature(ty: &Type, input: SetterInputPolicy) -> SetterSignature {
    if input.accepts_into() {
        SetterSignature::Into(ty.clone())
    } else {
        SetterSignature::Exact(ty.clone())
    }
}

fn setter_default_value(default: &SetterDefault) -> SetterDefaultValue {
    match default {
        SetterDefault::Default => SetterDefaultValue::Default,
        SetterDefault::Expr(expr) => SetterDefaultValue::Expr(expr.clone()),
    }
}

fn reject_state_ident_collision(
    generated_names: &mut GeneratedApiNamespace,
    state_ident: &Ident,
) -> Result<(), syn::Error> {
    generated_names.register_ident(
        state_ident,
        GeneratedApiNameKind::RequiredStateGeneric,
        |existing| state_ident_collision_message(state_ident, existing),
    )
}

fn reject_builder_helper_name_collisions(
    builder_name: &Ident,
    module_name: &Ident,
) -> Result<(), syn::Error> {
    let mut generated_names = GeneratedApiNamespace::new();
    generated_names.register_ident(
        builder_name,
        GeneratedApiNameKind::BuilderType,
        |existing| builder_helper_collision_message(builder_name, existing),
    )?;
    generated_names.register_ident(
        module_name,
        GeneratedApiNameKind::BuilderModule,
        |existing| builder_helper_collision_message(module_name, existing),
    )
}

fn reject_generated_method_collisions(slots: &[BuilderSlot]) -> Result<(), syn::Error> {
    let mut generated_names = builder_method_namespace();

    for slot in slots {
        let Some(method) = slot.setter_method() else {
            continue;
        };
        let method_name = method.to_string();
        if reserved_builder_method_name(&method_name) {
            return Err(syn::Error::new(
                method.span(),
                format!("setter method name `{method_name}` is reserved by koruma"),
            ));
        }
        generated_names.register_ident(
            method,
            GeneratedApiNameKind::BuilderMethod,
            |existing| builder_method_collision_message(&method_name, existing),
        )?;
    }

    for slot in slots {
        let BuilderSlot::OptionalSetter(slot) = slot else {
            continue;
        };
        if slot.maybe_inner_ty.is_none() {
            continue;
        }
        let maybe_method = format_ident!("maybe_{}", slot.method);
        let maybe_name = maybe_method.to_string();
        generated_names.register_ident(
            &maybe_method,
            GeneratedApiNameKind::OptionalBuilderMethod,
            |existing| optional_builder_method_collision_message(&maybe_name, existing),
        )?;
    }

    Ok(())
}

fn state_ident_collision_message(state_ident: &Ident, existing: &RegisteredApiName) -> String {
    match existing.kind {
        GeneratedApiNameKind::UserGeneric => {
            format!("generated required-state generic `{state_ident}` collides with a user generic")
        },
        _ => format!(
            "generated required-state generic `{state_ident}` collides with generated name `{}`",
            existing.ident
        ),
    }
}

fn builder_method_collision_message(method_name: &str, existing: &RegisteredApiName) -> String {
    if reserved_builder_method_name(method_name)
        || existing.kind == GeneratedApiNameKind::ReservedBuilderMethod
    {
        return format!("setter method name `{method_name}` is reserved by koruma");
    }

    format!(
        "setter method `{method_name}` collides with another setter `{}`",
        existing.ident
    )
}

fn optional_builder_method_collision_message(
    maybe_name: &str,
    existing: &RegisteredApiName,
) -> String {
    match existing.kind {
        GeneratedApiNameKind::BuilderMethod => format!(
            "generated optional setter method `{maybe_name}` collides with setter `{}`",
            existing.ident
        ),
        _ => format!(
            "generated optional setter method `{maybe_name}` collides with generated method `{}`",
            existing.ident
        ),
    }
}

fn builder_helper_collision_message(requested: &Ident, existing: &RegisteredApiName) -> String {
    format!(
        "generated builder helper `{requested}` collides with generated {} `{}`",
        builder_api_kind_label(existing.kind),
        existing.ident
    )
}

fn builder_api_kind_label(kind: GeneratedApiNameKind) -> &'static str {
    match kind {
        GeneratedApiNameKind::BuilderType => "builder type",
        GeneratedApiNameKind::BuilderModule => "builder module",
        GeneratedApiNameKind::BuilderMethod => "builder method",
        GeneratedApiNameKind::OptionalBuilderMethod => "optional builder method",
        GeneratedApiNameKind::ReservedBuilderMethod => "reserved builder method",
        GeneratedApiNameKind::UserGeneric => "user generic",
        GeneratedApiNameKind::RequiredStateGeneric => "required state generic",
        GeneratedApiNameKind::ExistingField => "input field",
        GeneratedApiNameKind::MainErrorStruct => "main error struct",
        GeneratedApiNameKind::FieldErrorStruct => "field error struct",
        GeneratedApiNameKind::FieldValidatorRefEnum => "field validator enum",
        GeneratedApiNameKind::ElementErrorStruct => "element error struct",
        GeneratedApiNameKind::ElementValidatorRefEnum => "element validator enum",
        GeneratedApiNameKind::ValidatorGetter => "validator getter",
        GeneratedApiNameKind::ValidatorVariant => "validator enum variant",
    }
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

fn render_builder_struct(plan: &ValidatorBuilderPlan) -> Result<TokenStream2, syn::Error> {
    let builder_name = &plan.builder_name;
    let mut builder_generics = plan.input.generics.clone();
    for state_ident in plan
        .slots()
        .iter()
        .filter_map(BuilderSlot::required_state_ident)
    {
        builder_generics.params.push(parse_quote!(#state_ident));
    }
    let field_defs: Vec<_> = plan
        .slots()
        .iter()
        .map(|slot| {
            let ident = slot.ident();
            let ty = slot.ty();
            quote! { #ident: ::std::option::Option<#ty> }
        })
        .collect();
    let state_idents: Vec<_> = plan
        .slots()
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

fn render_builder_impl(plan: &ValidatorBuilderPlan) -> Result<TokenStream2, syn::Error> {
    let builder_name = &plan.builder_name;
    let mut builder_generics = plan.input.generics.clone();
    for state_ident in plan
        .slots()
        .iter()
        .filter_map(BuilderSlot::required_state_ident)
    {
        builder_generics.params.push(parse_quote!(#state_ident));
    }
    let (impl_generics, builder_ty_generics, where_clause) = builder_generics.split_for_impl();
    let initial_fields: Vec<_> = plan
        .slots()
        .iter()
        .map(|slot| {
            let ident = slot.ident();
            quote! { #ident: ::std::option::Option::None }
        })
        .collect();
    let setter_methods: Vec<_> = plan
        .slots()
        .iter()
        .map(|slot| render_builder_setter(plan, slot))
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

fn render_builder_setter(plan: &ValidatorBuilderPlan, slot: &BuilderSlot) -> TokenStream2 {
    match slot {
        BuilderSlot::CapturedValue(slot) => render_value_setter(plan, &slot.ident, &slot.ty, true),
        BuilderSlot::SkippedValue(slot) => render_value_setter(plan, &slot.ident, &slot.ty, false),
        BuilderSlot::RequiredSetter(_)
        | BuilderSlot::OptionalSetter(_)
        | BuilderSlot::DefaultedSetter(_) => {
            if let Some(parts) = slot.setter_render_parts() {
                render_setter_slot(plan, parts)
            } else {
                quote! {}
            }
        },
    }
}

fn render_setter_slot(plan: &ValidatorBuilderPlan, slot: SetterRenderParts<'_>) -> TokenStream2 {
    let builder_name = &plan.builder_name;
    let method = slot.method;
    let arg_ty = setter_arg_ty(slot.signature);
    let value_expr = setter_value_expr(slot.signature);
    let return_ty = if slot.required {
        plan.builder_type_with_replaced_state(slot.ident)
    } else {
        quote! { Self }
    };
    let assignments = builder_assignments(plan.slots(), slot.ident, quote! { #value_expr });
    let maybe_method = if let Some(inner_ty) = slot.maybe_inner_ty {
        let maybe_method = format_ident!("maybe_{}", method);
        let maybe_assignments = builder_assignments(plan.slots(), slot.ident, quote! { value });
        Some(quote! {
            pub fn #maybe_method(self, value: ::std::option::Option<#inner_ty>) -> Self {
                #builder_name {
                    #(#maybe_assignments,)*
                    _state: ::std::marker::PhantomData,
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

fn setter_arg_ty(input: &SetterSignature) -> TokenStream2 {
    match input {
        SetterSignature::Exact(ty) | SetterSignature::OptionalExact { option_ty: ty } => {
            quote! { #ty }
        },
        SetterSignature::Into(ty) => quote! { impl ::std::convert::Into<#ty> },
        SetterSignature::OptionalInner { inner, into } => {
            if *into {
                quote! { impl ::std::convert::Into<#inner> }
            } else {
                quote! { #inner }
            }
        },
    }
}

fn setter_value_expr(input: &SetterSignature) -> TokenStream2 {
    match input {
        SetterSignature::Exact(_) | SetterSignature::OptionalExact { .. } => quote! { value },
        SetterSignature::Into(_) => quote! { ::std::convert::Into::into(value) },
        SetterSignature::OptionalInner { into: false, .. } => {
            quote! { ::std::option::Option::Some(value) }
        },
        SetterSignature::OptionalInner { into: true, .. } => {
            quote! { ::std::option::Option::Some(::std::convert::Into::into(value)) }
        },
    }
}

fn render_value_setter(
    plan: &ValidatorBuilderPlan,
    ident: &Ident,
    ty: &Type,
    capture_required: bool,
) -> TokenStream2 {
    let builder_name = &plan.builder_name;
    let method = format_ident!("with_value");
    let inner_ty = option_inner_type(ty).unwrap_or(ty);
    let value_expr = if option_inner_type(ty).is_some() {
        quote! { ::std::option::Option::Some(value) }
    } else {
        quote! { value }
    };
    let return_ty = if capture_required {
        plan.builder_type_with_replaced_state(ident)
    } else {
        quote! { Self }
    };
    let assignments = builder_assignments(plan.slots(), ident, value_expr);

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
    plan: &ValidatorBuilderPlan,
    build_ready_builder_ty: &TokenStream2,
    koruma: &TokenStream2,
) -> Result<TokenStream2, syn::Error> {
    let struct_name = &plan.struct_name;
    let (impl_generics, type_generics, where_clause) = plan.input.generics.split_for_impl();
    let field_values: Vec<_> = plan
        .slots()
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

        impl #impl_generics #koruma::__private::BuildValidator for #build_ready_builder_ty #where_clause {
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
                SetterDefaultValue::Expr(expr) => {
                    let expr = expr.as_ref();
                    quote! {
                        self.#ident.unwrap_or_else(|| #expr)
                    }
                },
                SetterDefaultValue::Default => quote! {
                    self.#ident.unwrap_or_default()
                },
            }
        },
    }
}

fn render_capture_value_ref_impl(
    plan: &ValidatorBuilderPlan,
    koruma: &TokenStream2,
) -> Result<TokenStream2, syn::Error> {
    let mut builder_generics = plan.input.generics.clone();
    let inner_type = plan.value_inner_type();
    let value_field_name = plan.value_slot().ident();
    let builder_state_args: Vec<_> = plan
        .slots()
        .iter()
        .filter_map(BuilderSlot::required_state_ident)
        .map(|state_ident| quote! { #state_ident })
        .collect();
    for state_ident in plan
        .slots()
        .iter()
        .filter_map(BuilderSlot::required_state_ident)
    {
        builder_generics.params.push(parse_quote!(#state_ident));
    }
    if plan.capture_policy == CapturePolicy::CloneInput {
        builder_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#inner_type: #koruma::__private::CapturedInputCanBeCloned));
    }
    let builder_ty = plan.builder_type_path_with_states(&builder_state_args);

    match plan.capture_policy {
        CapturePolicy::CloneInput => {
            let output_ty = plan.builder_type_with_replaced_state(value_field_name);
            let mut capture_generics = builder_generics;
            capture_generics
                .make_where_clause()
                .predicates
                .push(parse_quote!(#output_ty: #koruma::__private::BuildValidator));
            let (impl_generics, _, where_clause) = capture_generics.split_for_impl();
            Ok(quote! {
                impl #impl_generics #koruma::__private::CaptureValueRef<#inner_type>
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
                .push(parse_quote!(#builder_ty: #koruma::__private::BuildValidator));
            let (impl_generics, _, where_clause) = capture_generics.split_for_impl();
            Ok(quote! {
                impl #impl_generics #koruma::__private::CaptureValueRef<#inner_type>
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

fn direct_builder_methods(plan: &ValidatorBuilderPlan) -> Vec<TokenStream2> {
    plan.direct_methods()
        .iter()
        .map(|direct_method| render_direct_builder_method(plan, direct_method))
        .collect()
}

fn render_direct_builder_method(
    plan: &ValidatorBuilderPlan,
    direct_method: &DirectBuilderMethodPlan,
) -> TokenStream2 {
    let slot = &direct_method.setter;
    let method = &slot.method;
    let arg_ty = setter_arg_ty(&slot.signature);
    let module_name = &plan.module_name;
    let required_slots = plan.required_slots();
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
    let output_builder_ty = if slot.required {
        plan.builder_type_path_with_states(&state_args)
    } else {
        let empty_state_args: Vec<_> = required_slots
            .iter()
            .map(|_| quote! { #module_name::Empty })
            .collect();
        plan.builder_type_path_with_states(&empty_state_args)
    };
    let method_name_str = method.to_string();

    let maybe_method = if let (Some(inner_ty), Some(maybe_method)) = (
        slot.maybe_inner_ty.as_ref(),
        direct_method.maybe_method.as_ref(),
    ) {
        let maybe_method_name_str = maybe_method.to_string();
        Some(quote! {
            #[doc = concat!(
                "Starts building this validator with `",
                #maybe_method_name_str,
                "` set."
            )]
            pub fn #maybe_method(value: ::std::option::Option<#inner_ty>) -> #output_builder_ty {
                Self::__koruma_builder().#maybe_method(value)
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
