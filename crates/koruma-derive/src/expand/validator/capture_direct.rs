use super::*;

pub(super) fn render_capture_value_ref_impl(
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

pub(super) fn direct_builder_methods(plan: &ValidatorBuilderPlan) -> Vec<TokenStream2> {
    plan.direct_methods()
        .iter()
        .map(|direct_method| render_direct_builder_method(plan, direct_method))
        .collect()
}

pub(super) fn render_direct_builder_method(
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
