use super::*;

pub(super) fn render_builder_setter(
    plan: &ValidatorBuilderPlan,
    slot: &BuilderSlot,
) -> TokenStream2 {
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

pub(super) fn render_setter_slot(
    plan: &ValidatorBuilderPlan,
    slot: SetterRenderParts<'_>,
) -> TokenStream2 {
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

pub(super) fn setter_arg_ty(input: &SetterSignature) -> TokenStream2 {
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

pub(super) fn setter_value_expr(input: &SetterSignature) -> TokenStream2 {
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

pub(super) fn render_value_setter(
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
        #[doc(hidden)]
        pub fn #method(self, value: #inner_ty) -> #return_ty {
            #builder_name {
                #(#assignments,)*
                _state: ::std::marker::PhantomData,
            }
        }
    }
}

pub(super) fn builder_assignments(
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

pub(super) fn builder_type_with_replaced_state(
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

pub(super) fn render_build_impl(
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

pub(super) fn build_value_expr(slot: &BuilderSlot) -> TokenStream2 {
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
