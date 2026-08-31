use super::*;

pub(super) fn render_validator_metadata_impl(
    builder_plan: &ValidatorBuilderPlan,
    target_ty: &Type,
    koruma: &TokenStream2,
) -> TokenStream2 {
    let struct_name = &builder_plan.struct_name;
    let metadata_generics = builder_plan.input.generics.clone();
    let (impl_generics, type_generics, where_clause) = metadata_generics.split_for_impl();

    let descriptor_params = builder_plan
        .slots()
        .iter()
        .filter_map(|slot| {
            let parts = slot.setter_render_parts()?;
            let name = parts.ident.to_string();
            let ty = parts.signature.descriptor_type();
            let type_name = quote!(#ty).to_string();
            let required = parts.required;
            Some(quote! {
                #koruma::ValidatorParamDescriptor::new(#name, #type_name, #required)
            })
        })
        .collect::<Vec<_>>();

    let runtime_params = builder_plan
        .slots()
        .iter()
        .filter_map(|slot| {
            let parts = slot.setter_render_parts()?;
            let ident = parts.ident;
            let name = ident.to_string();
            let ty = slot.ty();
            let value = validator_param_value_expr(quote! { self.#ident }, ty, koruma);
            Some(quote! {
                #koruma::ValidatorParam::new(#name, #value)
            })
        })
        .collect::<Vec<_>>();

    quote! {
        impl #impl_generics #koruma::ValidatorMetadata<#target_ty>
            for #struct_name #type_generics #where_clause
        {
            fn validator_descriptor() -> #koruma::ValidatorDescriptor {
                const __KORUMA_PARAMS: &[#koruma::ValidatorParamDescriptor] = &[
                    #(#descriptor_params),*
                ];
                #koruma::ValidatorDescriptor::new(
                    ::core::any::type_name::<Self>(),
                    __KORUMA_PARAMS,
                )
            }

            fn validator_params(&self) -> ::std::vec::Vec<#koruma::ValidatorParam> {
                ::std::vec![#(#runtime_params),*]
            }
        }
    }
}

pub(super) fn validator_param_value_expr(
    value: TokenStream2,
    ty: &Type,
    koruma: &TokenStream2,
) -> TokenStream2 {
    if let Some(inner) = option_inner_type(ty) {
        let some_value =
            validator_param_ref_value_expr(quote! { __koruma_param_value }, inner, koruma);
        return quote! {
            match #value.as_ref() {
                Some(__koruma_param_value) => #some_value,
                None => #koruma::ValidatorParamValue::None,
            }
        };
    }

    match simple_type_name(ty).as_deref() {
        Some("bool") => quote! { #koruma::ValidatorParamValue::Bool(#value) },
        Some("i8" | "i16" | "i32" | "i64" | "isize") => {
            quote! { #koruma::ValidatorParamValue::I64(#value as i64) }
        },
        Some("u8" | "u16" | "u32" | "u64" | "usize") => {
            quote! { #koruma::ValidatorParamValue::U64(#value as u64) }
        },
        Some("f32" | "f64") => quote! { #koruma::ValidatorParamValue::F64(#value as f64) },
        Some("String") => quote! { #koruma::ValidatorParamValue::String(#value.clone()) },
        Some("str") if matches!(ty, Type::Reference(_)) => {
            quote! { #koruma::ValidatorParamValue::String(#value.to_string()) }
        },
        _ => quote! { #koruma::ValidatorParamValue::opaque(&#value) },
    }
}

pub(super) fn validator_param_ref_value_expr(
    value: TokenStream2,
    ty: &Type,
    koruma: &TokenStream2,
) -> TokenStream2 {
    match simple_type_name(ty).as_deref() {
        Some("bool") => quote! { #koruma::ValidatorParamValue::Bool(*#value) },
        Some("i8" | "i16" | "i32" | "i64" | "isize") => {
            quote! { #koruma::ValidatorParamValue::I64(*#value as i64) }
        },
        Some("u8" | "u16" | "u32" | "u64" | "usize") => {
            quote! { #koruma::ValidatorParamValue::U64(*#value as u64) }
        },
        Some("f32" | "f64") => quote! { #koruma::ValidatorParamValue::F64(*#value as f64) },
        Some("String") | Some("str") => {
            quote! { #koruma::ValidatorParamValue::String(#value.to_string()) }
        },
        _ => quote! { #koruma::ValidatorParamValue::opaque(#value) },
    }
}

pub(super) fn simple_type_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => path
            .path
            .segments
            .first()
            .map(|segment| segment.ident.to_string()),
        Type::Reference(reference) => simple_type_name(&reference.elem),
        _ => None,
    }
}
