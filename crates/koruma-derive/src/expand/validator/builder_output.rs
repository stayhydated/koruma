use super::*;

pub(super) fn generic_args(generics: &syn::Generics) -> Vec<TokenStream2> {
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

pub(super) fn builder_type_path(
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

pub(super) fn render_builder_struct(
    plan: &ValidatorBuilderPlan,
) -> Result<TokenStream2, syn::Error> {
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

pub(super) fn render_builder_impl(plan: &ValidatorBuilderPlan) -> Result<TokenStream2, syn::Error> {
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
