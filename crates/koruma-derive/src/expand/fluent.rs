use koruma_derive_core::{is_option_type, option_inner_type};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::expand::codegen::helper_generics_for_usages;
use crate::expand::plan::ValidationPlan;
use syn::DeriveInput;

/// Core expansion logic for the `#[derive(KorumaAllFluent)]` derive macro.
///
/// Generates `FluentMessage` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's
/// `FluentMessage` implementation.
#[cfg(feature = "fluent")]
pub fn expand_koruma_all_fluent(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let generics = &input.generics;

    let plan = ValidationPlan::build(&input, "KorumaAllFluent")?;

    // Generate FluentMessage impls for each field's validator enum
    let fluent_impls: Vec<TokenStream2> = plan
        .fields
        .iter()
        .zip(plan.field_infos())
        .filter(|(field_plan, _)| !field_plan.field_validators.is_empty())
        .map(|(field_plan, f)| {
            let field_ty = &f.ty;
            let enum_name = &field_plan.generated_names.field_validator_enum;
            let mut helper_usages: Vec<TokenStream2> = field_plan
                .field_validators
                .iter()
                .map(|planned| {
                    let vtype = &planned.validator_type;
                    quote! { #vtype }
                })
                .collect();
            if f.is_newtype() {
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                helper_usages.push(quote! { <#inner_ty as koruma::ValidateExt>::Error });
            }
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = field_plan
                .field_validators
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string_with(localize)
                    }
                })
                .collect();

            // Add Inner variant arm for newtype fields with additional validators
            let inner_arm = if f.is_newtype() {
                Some(quote! {
                    #enum_name::Inner(inner) => inner.to_fluent_string_with(localize)
                })
            } else {
                None
            };

            quote! {
                impl #helper_impl_generics ::es_fluent::FluentMessage for #enum_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string_with(
                        &self,
                        localize: &mut dyn for<'a> FnMut(
                            &str,
                            &str,
                            Option<&std::collections::HashMap<&str, ::es_fluent::FluentValue<'a>>>,
                        ) -> String,
                    ) -> String {
                        use ::es_fluent::FluentMessage;
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
        .filter(|field_plan| !field_plan.element_validators.is_empty())
        .map(|field_plan| {
            let enum_name = &field_plan.generated_names.element_validator_enum;
            let helper_usages: Vec<TokenStream2> = field_plan
                .element_validators
                .iter()
                .map(|planned| {
                    let vtype = &planned.validator_type;
                    quote! { #vtype }
                })
                .collect();
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = field_plan
                .element_validators
                .iter()
                .map(|planned| {
                    let variant_name = &planned.variant_ident;
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string_with(localize)
                    }
                })
                .collect();

            quote! {
                impl #helper_impl_generics ::es_fluent::FluentMessage for #enum_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string_with(
                        &self,
                        localize: &mut dyn for<'a> FnMut(
                            &str,
                            &str,
                            Option<&std::collections::HashMap<&str, ::es_fluent::FluentValue<'a>>>,
                        ) -> String,
                    ) -> String {
                        use ::es_fluent::FluentMessage;
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
        .zip(plan.field_infos())
        .filter(|(field_plan, f)| !field_plan.field_validators.is_empty() || f.is_newtype())
        .map(|(field_plan, f)| {
            let field_ty = &f.ty;
            let error_struct_name = &field_plan.generated_names.field_error_struct;
            let mut helper_usages: Vec<TokenStream2> = field_plan
                .field_validators
                .iter()
                .map(|planned| {
                    let vtype = &planned.validator_type;
                    quote! { #vtype }
                })
                .collect();
            if f.is_newtype() {
                let inner_ty = option_inner_type(field_ty).unwrap_or(field_ty);
                helper_usages.push(quote! { <#inner_ty as koruma::ValidateExt>::Error });
            }
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            // Join all field-level validator messages, and include the delegated
            // newtype error when present.
            let message_pushes: Vec<TokenStream2> = field_plan
                .field_validators
                .iter()
                .map(|planned| {
                    let validator_snake = &planned.field_ident;
                    quote! {
                        if let Some(v) = &self.#validator_snake {
                            messages.push(v.to_fluent_string_with(localize));
                        }
                    }
                })
                .collect();
            let inner_message_push = if f.is_newtype() {
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

            quote! {
                impl #helper_impl_generics ::es_fluent::FluentMessage for #error_struct_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string_with(
                        &self,
                        localize: &mut dyn for<'a> FnMut(
                            &str,
                            &str,
                            Option<&std::collections::HashMap<&str, ::es_fluent::FluentValue<'a>>>,
                        ) -> String,
                    ) -> String {
                        use ::es_fluent::FluentMessage;
                        let mut messages = Vec::new();
                        #(#message_pushes)*
                        #inner_message_push
                        messages.join("\n")
                    }
                }
            }
        })
        .collect();

    Ok(quote! {
        #(#fluent_impls)*
        #(#element_fluent_impls)*
        #(#error_struct_impls)*
    })
}
