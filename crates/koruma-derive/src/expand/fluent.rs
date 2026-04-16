use heck::ToUpperCamelCase;
use koruma_derive_core::{
    FieldInfo, ValidatorAttr, is_option_type, option_inner_type, parse_struct_options,
};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

use crate::expand::codegen::{
    helper_generics_for_usages, validator_field_ident, validator_type_for_field,
    validator_variant_ident,
};
use crate::expand::collect_field_infos;
use syn::DeriveInput;

/// Core expansion logic for the `#[derive(KorumaAllFluent)]` derive macro.
///
/// Generates `ToFluentString` implementations for the `{Struct}{Field}KorumaValidator` enums
/// returned by the `all()` method. Each variant delegates to its inner validator's ToFluentString.
#[cfg(feature = "fluent")]
pub fn expand_koruma_all_fluent(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let generics = &input.generics;
    let struct_options = parse_struct_options(&input.attrs)?;

    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "KorumaAllFluent can only be derived for structs",
            ));
        },
    };

    // Parse all fields and extract validation info
    let field_infos: Vec<FieldInfo> = collect_field_infos(fields, Some(&struct_options))?;

    // Generate ToFluentString impls for each field's validator enum
    let fluent_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.field_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let enum_name = format_ident!(
                "{}{}KorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let mut helper_usages: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, false);
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

            let match_arms: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v: &ValidatorAttr| {
                    let variant_name =
                        validator_variant_ident(v, &f.validation.field_validators);
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string()
                    }
                })
                .collect();

            // Add Inner variant arm for newtype fields with additional validators
            let inner_arm = if f.is_newtype() {
                Some(quote! {
                    #enum_name::Inner(inner) => inner.to_fluent_string()
                })
            } else {
                None
            };

            quote! {
                impl #helper_impl_generics ::es_fluent::ToFluentString for #enum_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string(&self) -> String {
                        use ::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms,)*
                            #inner_arm
                        }
                    }
                }
            }
        })
        .collect();

    // Generate ToFluentString impls for element validator enums (if any)
    let element_fluent_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.element_validators.is_empty())
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let enum_name = format_ident!(
                "{}{}ElementKorumaValidator",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let helper_usages: Vec<TokenStream2> = f
                .validation
                .element_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, true);
                    quote! { #vtype }
                })
                .collect();
            let helper_generics = helper_generics_for_usages(generics, &helper_usages);
            let helper_impl_generics = &helper_generics.impl_generics;
            let helper_ty_generics = &helper_generics.ty_generics;
            let helper_where_clause = &helper_generics.where_clause;

            let match_arms: Vec<TokenStream2> = f
                .validation
                .element_validators
                .iter()
                .map(|v: &ValidatorAttr| {
                    let variant_name =
                        validator_variant_ident(v, &f.validation.element_validators);
                    quote! {
                        #enum_name::#variant_name(v) => v.to_fluent_string()
                    }
                })
                .collect();

            quote! {
                impl #helper_impl_generics ::es_fluent::ToFluentString for #enum_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string(&self) -> String {
                        use ::es_fluent::ToFluentString;
                        match self {
                            #(#match_arms),*
                        }
                    }
                }
            }
        })
        .collect();

    // Generate ToFluentString impls for error structs
    let error_struct_impls: Vec<TokenStream2> = field_infos
        .iter()
        .filter(|f| !f.validation.field_validators.is_empty() || f.is_newtype())
        .map(|f| {
            let field_name = &f.name;
            let field_ty = &f.ty;
            let error_struct_name = format_ident!(
                "{}{}KorumaValidationError",
                struct_name,
                field_name.to_string().to_upper_camel_case()
            );
            let mut helper_usages: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v| {
                    let vtype = validator_type_for_field(v, field_ty, false);
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
            let message_pushes: Vec<TokenStream2> = f
                .validation
                .field_validators
                .iter()
                .map(|v| {
                    let validator_snake =
                        validator_field_ident(v, &f.validation.field_validators);
                    quote! {
                        if let Some(v) = &self.#validator_snake {
                            messages.push(v.to_fluent_string());
                        }
                    }
                })
                .collect();
            let inner_message_push = if f.is_newtype() {
                if is_option_type(field_ty) {
                    Some(quote! {
                        if let Some(inner) = self.inner() {
                            if !inner.is_empty() {
                                messages.push(inner.to_fluent_string());
                            }
                        }
                    })
                } else {
                    Some(quote! {
                        if !self.inner().is_empty() {
                            messages.push(self.inner().to_fluent_string());
                        }
                    })
                }
            } else {
                None
            };

            quote! {
                impl #helper_impl_generics ::es_fluent::ToFluentString for #error_struct_name #helper_ty_generics #helper_where_clause {
                    fn to_fluent_string(&self) -> String {
                        use ::es_fluent::ToFluentString;
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
