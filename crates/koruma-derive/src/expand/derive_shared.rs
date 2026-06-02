use crate::expand::codegen::helper_generics_for_usages;
use crate::expand::plan::{FieldPlan, PlannedSetterArg, PlannedValidator, PlannedValidatorType};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Generics, Type};

impl ToTokens for PlannedValidatorType {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let validator = &self.validator;
        if let Some(type_arg) = &self.type_arg {
            quote! { #validator<#type_arg> }.to_tokens(tokens);
        } else {
            quote! { #validator }.to_tokens(tokens);
        }
    }
}

impl ToTokens for PlannedSetterArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

pub(crate) fn validator_builder_expr(validator: &PlannedValidator) -> TokenStream2 {
    let builder_type = validator_builder_type_expr(validator);
    let mut setter_calls = validator.setter_calls.iter();
    let Some(first_call) = setter_calls.next() else {
        return quote! { #builder_type::__koruma_builder() };
    };

    let first_method = &first_call.method;
    let first_args = &first_call.args;
    let rest_calls: Vec<TokenStream2> = setter_calls
        .map(|call| {
            let method = &call.method;
            let args = &call.args;
            quote! { .#method(#(#args),*) }
        })
        .collect();

    quote! {
        #builder_type::#first_method(#(#first_args),*)
            #(#rest_calls)*
    }
}

fn validator_builder_type_expr(validator: &PlannedValidator) -> TokenStream2 {
    let validator_path = &validator.builder_type.validator;
    if let Some(type_arg) = &validator.builder_type.type_arg {
        quote! { #validator_path::<#type_arg> }
    } else {
        quote! { #validator_path }
    }
}

fn validator_type_usages<'a>(
    validators: impl IntoIterator<Item = &'a PlannedValidator>,
) -> Vec<Type> {
    validators
        .into_iter()
        .map(|validator| validator.validator_type.as_type())
        .collect()
}

pub(crate) fn field_error_type_path(
    generics: &Generics,
    field_plan: &FieldPlan,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let ty = field_error_type(generics, field_plan, koruma);
    quote! { #ty }
}

pub(crate) fn field_error_type(
    generics: &Generics,
    field_plan: &FieldPlan,
    koruma: &TokenStream2,
) -> Type {
    let field_error_struct_name = &field_plan.generated_names.field_error_struct;

    let mut usages = validator_type_usages(field_plan.field_validators());
    if field_plan.is_newtype() {
        let inner_ty = field_plan.inner_type();
        usages.push(syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error });
    }

    if !field_plan.is_newtype() && field_plan.has_element_validators() {
        let element_error_struct_name = &field_plan.generated_names.element_error_struct;
        let element_usages = validator_type_usages(field_plan.element_validators());
        let element_helper = helper_generics_for_usages(generics, &element_usages);
        let element_error_ty = element_helper.type_path_type(element_error_struct_name);
        usages.push(syn::parse_quote! { Vec<(usize, #element_error_ty)> });
    }

    let helper = helper_generics_for_usages(generics, &usages);
    helper.type_path_type(field_error_struct_name)
}
