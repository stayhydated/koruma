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
    let metadata_impl = render_validator_metadata_impl(&builder_plan, &inner_type, &koruma);
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
        #metadata_impl

        #showcase_registration
    })
}

mod builder_output;
mod builder_plan;
mod capture_direct;
mod collisions;
mod metadata;
mod setters;

use builder_output::*;
pub(crate) use builder_plan::*;
use capture_direct::*;
use collisions::*;
use metadata::*;
use setters::*;
