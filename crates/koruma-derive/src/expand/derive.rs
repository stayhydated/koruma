use crate::expand::derive_constructors::{render_try_from_impl, render_try_new_fn};
use crate::expand::derive_field_errors::render_field_error_structs;
use crate::expand::derive_main_error::render_main_error;
use crate::expand::derive_newtype::{
    NewtypeDerefInputs, render_newtype_deref_impl, render_newtype_marker_impl,
};
use crate::expand::derive_validation::render_validation_checks;
use crate::expand::koruma_crate_path;
use crate::expand::plan::ValidationPlan;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;

/// Core expansion logic for the `#[derive(Koruma)]` derive macro.
///
/// Takes a parsed DeriveInput and returns the expanded TokenStream.
pub fn expand_koruma(input: DeriveInput) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let plan = ValidationPlan::build(&input, "Koruma")?;
    let error_struct_name = plan.main_error_struct.clone();
    let koruma = koruma_crate_path();
    let fields = match &input.data {
        syn::Data::Struct(data) => &data.fields,
        _ => unreachable!("ValidationPlan already rejects non-struct inputs"),
    };

    let field_error_structs = render_field_error_structs(&plan, struct_name, generics, &koruma);

    let main_error = render_main_error(&plan, struct_name, &error_struct_name, generics, &koruma);
    let main_error_definition = &main_error.definition;
    let main_error_impl_generics = &main_error.impl_generics;
    let main_error_ty_generics = &main_error.ty_generics;
    let main_error_where_clause = &main_error.where_clause;
    let main_error_path = &main_error.path;
    let error_fields = &main_error.fields;
    let getter_methods = &main_error.getter_methods;
    let is_empty_body = &main_error.is_empty_body;
    let error_defaults = &main_error.defaults;

    let validation_checks = render_validation_checks(&plan, &koruma)?;

    let struct_name_str = struct_name.to_string();
    let impl_generics_tokens = quote! { #impl_generics };
    let ty_generics_tokens = quote! { #ty_generics };
    let where_clause_tokens = quote! { #where_clause };
    let try_new_fn = render_try_new_fn(&plan, fields, &struct_name_str, main_error_path);
    let newtype_marker_impl = render_newtype_marker_impl(
        &plan,
        struct_name,
        &impl_generics_tokens,
        &ty_generics_tokens,
        &where_clause_tokens,
        &koruma,
    );
    let try_from_impl = render_try_from_impl(
        &plan,
        struct_name,
        &impl_generics_tokens,
        &ty_generics_tokens,
        &where_clause_tokens,
        main_error_path,
    );
    let newtype_deref_impl = render_newtype_deref_impl(NewtypeDerefInputs {
        plan: &plan,
        generics,
        error_struct_name: &error_struct_name,
        main_error_impl_generics,
        main_error_ty_generics,
        main_error_where_clause,
        koruma: &koruma,
    });

    let field_names: Vec<String> = plan
        .fields
        .iter()
        .map(|field| field.name.to_string())
        .collect();
    let fields_doc = if field_names.is_empty() {
        String::new()
    } else if field_names.len() == 1 {
        format!("field: `{}`", field_names[0])
    } else {
        format!("fields: `{}`", field_names.join("`, `"))
    };

    Ok(quote! {
        #(#field_error_structs)*

        #[doc = concat!("Validation errors for [`", #struct_name_str, "`].\n\nContains per-field error structs for ", #fields_doc, ".")]
        #[derive(Debug, Default)]
        pub struct #error_struct_name #main_error_definition {
            #(#error_fields),*
        }

        impl #main_error_impl_generics #error_struct_name #main_error_ty_generics #main_error_where_clause {
            #(#getter_methods)*
        }

        #newtype_deref_impl

        impl #main_error_impl_generics #koruma::ValidationError for #error_struct_name #main_error_ty_generics #main_error_where_clause {
            fn is_empty(&self) -> bool {
                #is_empty_body
            }
        }

        impl #impl_generics #struct_name #ty_generics #where_clause {
            #try_new_fn

            #[doc = concat!("Validates all fields of `", #struct_name_str, "` and returns an error struct containing all validation failures.")]
            pub fn validate(&self) -> Result<(), #main_error_path> {
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

        impl #impl_generics #koruma::ValidateExt for #struct_name #ty_generics #where_clause {
            type Error = #main_error_path;

            fn validate(&self) -> Result<(), #main_error_path> {
                Self::validate(self)
            }
        }

        #newtype_marker_impl

        #try_from_impl
    })
}
