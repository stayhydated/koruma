use koruma_derive_core::is_option_type;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};

use crate::expand::codegen::{helper_generics_for_usages, ref_enum_generics_for_usages};
use crate::expand::derive_shared::field_error_type;
use crate::expand::koruma_crate_path;
use crate::expand::plan::{PlannedMainErrorField, PlannedMainErrorStorage, ValidationPlan};
use syn::{DeriveInput, Generics, Type, parse_quote};

fn add_fluent_message_bounds(generics: &mut Generics, usages: &[Type]) {
    let where_clause = generics.make_where_clause();
    for usage in usages {
        where_clause
            .predicates
            .push(parse_quote!(#usage: ::es_fluent::FluentMessage));
    }
}

fn main_error_storage_type(
    field: &PlannedMainErrorField<'_>,
    generics: &Generics,
    koruma: &TokenStream2,
) -> Type {
    let field_plan = field.field;
    let inner_ty = field_plan.inner_type();
    match field.storage {
        PlannedMainErrorStorage::NestedDirect => {
            parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error }
        },
        PlannedMainErrorStorage::NestedOptional => {
            parse_quote! { Option<<#inner_ty as #koruma::ValidateExt>::Error> }
        },
        PlannedMainErrorStorage::FieldError => field_error_type(generics, field_plan, koruma),
    }
}

/// Core expansion logic for the `#[derive(KorumaAllFluent)]` derive macro.
///
/// Generates `FluentMessage` implementations for the borrowed
/// `{Struct}{Field}KorumaValidatorRef` enums returned by the `all()` method.
/// Each variant delegates to its inner validator's `FluentMessage` implementation.
#[cfg(feature = "fluent")]
pub fn expand_koruma_all_fluent(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let koruma = koruma_crate_path();

    let plan = ValidationPlan::build(&input, "KorumaAllFluent")?;
    let requires_koruma_impl = quote! {
        impl #impl_generics #koruma::__private::KorumaAllFluentRequiresKoruma
            for #struct_name #ty_generics #where_clause
        {
        }
    };

    // Generate FluentMessage impls for each field's validator enum
    let fluent_impls: Vec<TokenStream2> = plan
        .fields
        .iter()
        .filter(|field_plan| field_plan.has_field_validators())
        .map(|field_plan| {
            let enum_name = &field_plan.generated_names.field_validator_ref_enum;
            let mut helper_usages: Vec<Type> = field_plan
                .field_validators()
                .iter()
                .map(|planned| planned.validator_type.as_type())
                .collect();
            let mut fluent_assertions: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|planned| {
                    let span = planned.source_span;
                    let validator_ty = planned.validator_type.as_type();
                    quote_spanned! {span=>
                        #koruma::__private::assert_fluent_message::<#validator_ty>();
                    }
                })
                .collect();
            if field_plan.is_newtype() {
                let inner_ty = field_plan.inner_type();
                helper_usages
                    .push(syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error });
                let span = field_plan
                    .source
                    .marker_span
                    .unwrap_or_else(|| field_plan.name.span());
                fluent_assertions.push(quote_spanned! {span=>
                    #koruma::__private::assert_fluent_message::<<#inner_ty as #koruma::ValidateExt>::Error>();
                });
            }
            let helper_generics = ref_enum_generics_for_usages(generics, &helper_usages);
            let mut fluent_generics = helper_generics.definition;
            add_fluent_message_bounds(&mut fluent_generics, &helper_usages);
            let (helper_impl_generics, helper_ty_generics, helper_where_clause) =
                fluent_generics.split_for_impl();

            let match_arms: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => {
                            ::es_fluent::FluentMessage::to_fluent_string_with(*v, localize)
                        }
                    }
                })
                .collect();

            // Add Inner variant arm for newtype fields with additional validators
            let inner_arm = if field_plan.is_newtype() {
                Some(quote! {
                    #enum_name::Inner(inner) => {
                        ::es_fluent::FluentMessage::to_fluent_string_with(*inner, localize)
                    }
                })
            } else {
                None
            };

            quote! {
                impl #helper_impl_generics ::es_fluent::FluentMessage for #enum_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string_with(
                        &self,
                        localize: &mut dyn for<'a> FnMut(
                            ::es_fluent::registry::StaticFluentDomain,
                            ::es_fluent::registry::StaticFluentEntryId,
                            Option<&::es_fluent::FluentArgs<'a>>,
                        ) -> String,
                    ) -> String {
                        #(#fluent_assertions)*

                        match self {
                            #(#match_arms,)*
                            #inner_arm
                        }
                    }
                }
            }
        })
        .collect();

    // Generate FluentMessage impls for element validator enums (if any)
    let element_fluent_impls: Vec<TokenStream2> = plan
        .fields
        .iter()
        .filter(|field_plan| field_plan.has_element_validators())
        .map(|field_plan| {
            let enum_name = &field_plan.generated_names.element_validator_ref_enum;
            let helper_usages: Vec<Type> = field_plan
                .element_validators()
                .iter()
                .map(|planned| planned.validator_type.as_type())
                .collect();
            let fluent_assertions: Vec<TokenStream2> = field_plan
                .element_validators()
                .iter()
                .map(|planned| {
                    let span = planned.source_span;
                    let validator_ty = planned.validator_type.as_type();
                    quote_spanned! {span=>
                        #koruma::__private::assert_fluent_message::<#validator_ty>();
                    }
                })
                .collect();
            let helper_generics = ref_enum_generics_for_usages(generics, &helper_usages);
            let mut fluent_generics = helper_generics.definition;
            add_fluent_message_bounds(&mut fluent_generics, &helper_usages);
            let (helper_impl_generics, helper_ty_generics, helper_where_clause) =
                fluent_generics.split_for_impl();

            let match_arms: Vec<TokenStream2> = field_plan
                .element_validators()
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => {
                            ::es_fluent::FluentMessage::to_fluent_string_with(*v, localize)
                        }
                    }
                })
                .collect();

            quote! {
                impl #helper_impl_generics ::es_fluent::FluentMessage for #enum_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string_with(
                        &self,
                        localize: &mut dyn for<'a> FnMut(
                            ::es_fluent::registry::StaticFluentDomain,
                            ::es_fluent::registry::StaticFluentEntryId,
                            Option<&::es_fluent::FluentArgs<'a>>,
                        ) -> String,
                    ) -> String {
                        #(#fluent_assertions)*

                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    // Generate FluentMessage impls for error structs
    let error_struct_impls: Vec<TokenStream2> = plan
        .fields
        .iter()
        .filter(|field_plan| field_plan.has_field_validators() || field_plan.is_newtype())
        .map(|field_plan| {
            let field_ty = &field_plan.source.ty;
            let error_struct_name = &field_plan.generated_names.field_error_struct;
            let mut helper_usages: Vec<Type> = field_plan
                .field_validators()
                .iter()
                .map(|planned| planned.validator_type.as_type())
                .collect();
            let mut fluent_assertions: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|planned| {
                    let span = planned.source_span;
                    let validator_ty = planned.validator_type.as_type();
                    quote_spanned! {span=>
                        #koruma::__private::assert_fluent_message::<#validator_ty>();
                    }
                })
                .collect();
            if field_plan.is_newtype() {
                let inner_ty = field_plan.inner_type();
                helper_usages
                    .push(syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error });
                let span = field_plan
                    .source
                    .marker_span
                    .unwrap_or_else(|| field_plan.name.span());
                fluent_assertions.push(quote_spanned! {span=>
                    #koruma::__private::assert_fluent_message::<<#inner_ty as #koruma::ValidateExt>::Error>();
                });
            }
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let mut fluent_generics = helper_generics.definition;
            add_fluent_message_bounds(&mut fluent_generics, &helper_usages);
            let (helper_impl_generics, helper_ty_generics, helper_where_clause) =
                fluent_generics.split_for_impl();

            // Join all field-level validator messages, and include the delegated
            // newtype error when present.
            let message_pushes: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|planned| {
                    let validator_snake = &planned.field_ident;
                    quote! {
                        if let Some(v) = &self.#validator_snake {
                            messages.push(
                                ::es_fluent::FluentMessage::to_fluent_string_with(v, localize)
                            );
                        }
                    }
                })
                .collect();
            let inner_message_push = if field_plan.is_newtype() {
                if is_option_type(field_ty) {
                    Some(quote! {
                        if let Some(inner) = self.inner() {
                            if !inner.is_empty() {
                                messages.push(inner.to_fluent_string_with(localize));
                            }
                        }
                    })
                } else {
                    Some(quote! {
                        if !self.inner().is_empty() {
                            messages.push(self.inner().to_fluent_string_with(localize));
                        }
                    })
                }
            } else {
                None
            };
            let fluent_message_import = if field_plan.is_newtype() {
                quote! { use ::es_fluent::FluentMessage; }
            } else {
                quote! {}
            };

            quote! {
                impl #helper_impl_generics ::es_fluent::FluentMessage for #error_struct_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string_with(
                        &self,
                        localize: &mut dyn for<'a> FnMut(
                            ::es_fluent::registry::StaticFluentDomain,
                            ::es_fluent::registry::StaticFluentEntryId,
                            Option<&::es_fluent::FluentArgs<'a>>,
                        ) -> String,
                    ) -> String {
                        #(#fluent_assertions)*
                        #fluent_message_import

                        let mut messages = Vec::new();
                        #(#message_pushes)*
                        #inner_message_push
                        messages.join("\n")
                    }
                }
            }
        })
        .collect();

    let main_error_render_plan = plan.main_error_render_plan();
    let main_error_impl = if main_error_render_plan.fields.is_empty() {
        quote! {}
    } else {
        let main_error_usages: Vec<Type> = main_error_render_plan
            .fields
            .iter()
            .map(|field| main_error_storage_type(field, generics, &koruma))
            .collect();
        let main_error_assertions: Vec<TokenStream2> = main_error_usages
            .iter()
            .map(|usage| {
                let span = struct_name.span();
                quote_spanned! {span=>
                    #koruma::__private::assert_fluent_message::<#usage>();
                }
            })
            .collect();
        let main_error_generics = helper_generics_for_usages(generics, &main_error_usages);
        let mut main_error_fluent_generics = main_error_generics.definition;
        add_fluent_message_bounds(&mut main_error_fluent_generics, &main_error_usages);
        let (main_error_impl_generics, main_error_ty_generics, main_error_where_clause) =
            main_error_fluent_generics.split_for_impl();
        let main_error_struct = &plan.main_error_struct;
        let main_error_pushes: Vec<TokenStream2> = main_error_render_plan
            .fields
            .iter()
            .map(|field| {
                let field_name = &field.field.name;
                match field.storage {
                    PlannedMainErrorStorage::NestedOptional => quote! {
                        if let Some(error) = &self.#field_name {
                            if !error.is_empty() {
                                messages.push(
                                    ::es_fluent::FluentMessage::to_fluent_string_with(error, localize)
                                );
                            }
                        }
                    },
                    PlannedMainErrorStorage::NestedDirect | PlannedMainErrorStorage::FieldError => {
                        quote! {
                            if !self.#field_name.is_empty() {
                                messages.push(
                                    ::es_fluent::FluentMessage::to_fluent_string_with(
                                        &self.#field_name,
                                        localize
                                    )
                                );
                            }
                        }
                    },
                }
            })
            .collect();

        quote! {
            impl #main_error_impl_generics ::es_fluent::FluentMessage
                for #main_error_struct #main_error_ty_generics #main_error_where_clause
            {
                fn to_fluent_string_with(
                    &self,
                    localize: &mut dyn for<'a> FnMut(
                        ::es_fluent::registry::StaticFluentDomain,
                        ::es_fluent::registry::StaticFluentEntryId,
                        Option<&::es_fluent::FluentArgs<'a>>,
                    ) -> String,
                    ) -> String {
                    #(#main_error_assertions)*

                    let mut messages = Vec::new();
                    #(#main_error_pushes)*
                    messages.join("\n")
                }
            }
        }
    };

    Ok(quote! {
        #requires_koruma_impl
        #(#fluent_impls)*
        #(#element_fluent_impls)*
        #(#error_struct_impls)*
        #main_error_impl
    })
}
