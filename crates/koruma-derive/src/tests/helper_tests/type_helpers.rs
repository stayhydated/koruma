use super::support::*;

#[test]
fn test_option_inner_type_extracts_inner() {
    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    let inner = option_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("i32"),
        "Expected i32, got: {}",
        inner_str
    );
}

#[test]
fn test_option_inner_type_nested() {
    let ty: syn::Type = syn::parse_quote!(Option<Vec<String>>);
    let inner = option_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("Vec"),
        "Expected Vec<String>, got: {}",
        inner_str
    );
}

#[test]
fn test_option_inner_type_supports_qualified_paths() {
    let std_ty: syn::Type = syn::parse_quote!(std::option::Option<i32>);
    assert_eq!(
        option_inner_type(&std_ty).map(|inner| quote!(#inner).to_string()),
        Some("i32".to_string())
    );

    let core_ty: syn::Type = syn::parse_quote!(core::option::Option<String>);
    assert_eq!(
        option_inner_type(&core_ty).map(|inner| quote!(#inner).to_string()),
        Some("String".to_string())
    );
}

#[test]
fn test_option_inner_type_returns_none_for_non_option() {
    let ty: syn::Type = syn::parse_quote!(i32);
    assert!(option_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(Vec<i32>);
    assert!(option_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(String);
    assert!(option_inner_type(&ty).is_none());
}

#[test]
fn test_vec_inner_type_extracts_inner() {
    let ty: syn::Type = syn::parse_quote!(Vec<f64>);
    let inner = vec_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("f64"),
        "Expected f64, got: {}",
        inner_str
    );
}

#[test]
fn test_vec_inner_type_complex() {
    let ty: syn::Type = syn::parse_quote!(Vec<Option<String>>);
    let inner = vec_inner_type(&ty);
    assert!(inner.is_some());
    let inner_str = quote!(#inner).to_string();
    assert!(
        inner_str.contains("Option"),
        "Expected Option<String>, got: {}",
        inner_str
    );
}

#[test]
fn test_vec_inner_type_supports_qualified_paths() {
    let std_ty: syn::Type = syn::parse_quote!(std::vec::Vec<f64>);
    assert_eq!(
        vec_inner_type(&std_ty).map(|inner| quote!(#inner).to_string()),
        Some("f64".to_string())
    );

    let alloc_ty: syn::Type = syn::parse_quote!(alloc::vec::Vec<String>);
    assert_eq!(
        vec_inner_type(&alloc_ty).map(|inner| quote!(#inner).to_string()),
        Some("String".to_string())
    );
}

#[test]
fn test_vec_inner_type_returns_none_for_non_vec() {
    let ty: syn::Type = syn::parse_quote!(i32);
    assert!(vec_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(Option<i32>);
    assert!(vec_inner_type(&ty).is_none());

    let ty: syn::Type = syn::parse_quote!(HashMap<String, i32>);
    assert!(vec_inner_type(&ty).is_none());
}

#[test]
fn test_effective_validation_type_for_each_on_optional_vec_uses_element_type() {
    let ty: syn::Type = syn::parse_quote!(Option<Vec<i32>>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "i32");
}

#[test]
fn test_effective_validation_type_for_each_on_qualified_option_vec_uses_element_type() {
    let ty: syn::Type =
        syn::parse_quote!(core::option::Option<std::vec::Vec<core::option::Option<String>>>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_effective_validation_type_for_each_on_vec_option_unwraps_inner_option() {
    let ty: syn::Type = syn::parse_quote!(Vec<Option<String>>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_effective_validation_type_for_each_on_slice_uses_element_type() {
    let ty: syn::Type = syn::parse_quote!(&[i32]);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "i32");
}

#[test]
fn test_effective_validation_type_for_each_on_optional_slice_option_unwraps_inner_option() {
    let ty: syn::Type = syn::parse_quote!(Option<&[Option<String>]>);
    let effective = effective_validation_type(&ty, ValidationSite::Element);
    assert_eq!(quote!(#effective).to_string(), "String");
}

#[test]
fn test_helper_generics_tracks_lifetimes_consts_and_where_dependencies() {
    let item: ItemStruct = syn::parse_quote! {
        struct Demo<'a, 'b, T, U, const N: usize>
        where
            T: Into<U>,
            U: Clone,
            [u8; N]: Default,
            &'a T: Default,
            &'b str: Default,
        {
            value: &'a T,
        }
    };

    let usages: Vec<syn::Type> = vec![syn::parse_quote! { (&'a T, [u8; N], &'z str) }];
    let helper = helper_generics_for_usages(&item.generics, &usages);
    let definition_generics = &helper.definition;
    let definition = quote!(#definition_generics).to_string();
    assert!(definition.contains("'a"));
    assert!(!definition.contains("'b"));
    assert!(definition.contains("T"));
    assert!(definition.contains("U"));
    assert!(definition.contains("N"));

    let helper_ident = format_ident!("Helper");
    assert_eq!(
        helper.type_path(&helper_ident).to_string(),
        "Helper < 'a , T , U , N >"
    );
    assert!(helper.where_clause.to_string().contains("T : Into < U >"));
}

#[test]
fn test_helper_generics_ignores_non_generic_path_segments() {
    let item: ItemStruct = syn::parse_quote! {
        struct Demo<T, U, Result, const N: usize>
        where
            U: Iterator<Item = T>,
            Result: Default,
        {
            value: U,
        }
    };

    let usages: Vec<syn::Type> =
        vec![syn::parse_quote! { (::std::result::Result<U, ()>, [u8; N]) }];
    let helper = helper_generics_for_usages(&item.generics, &usages);
    let definition_generics = &helper.definition;
    let definition = quote!(#definition_generics).to_string();
    assert!(definition.contains("T"));
    assert!(definition.contains("U"));
    assert!(definition.contains("N"));
    assert!(!definition.contains("Result : Default"));

    let helper_ident = format_ident!("Helper");
    assert_eq!(
        helper.type_path(&helper_ident).to_string(),
        "Helper < T , U , N >"
    );
}
