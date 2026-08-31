use super::support::*;

#[test]
fn test_each_collection_accepts_arrays_groups_and_parentheses() {
    let array_ty: syn::Type = syn::parse_quote!([i32; 3]);
    let array_collection =
        classify_each_collection(&array_ty).expect("arrays should support each(...)");
    assert_eq!(array_collection.outer_cardinality, Cardinality::Required);
    let array_element_ty = array_collection.element_ty;
    assert_eq!(quote!(#array_element_ty).to_string(), "i32");

    let paren_ty: syn::Type = syn::parse_quote!((Vec<i32>));
    let paren_collection =
        classify_each_collection(&paren_ty).expect("parenthesized Vec should support each(...)");
    let paren_element_ty = paren_collection.element_ty;
    assert_eq!(quote!(#paren_element_ty).to_string(), "i32");

    let group_ty = syn::Type::Group(syn::TypeGroup {
        attrs: Vec::new(),
        group_token: Default::default(),
        elem: Box::new(syn::parse_quote!(Vec<i32>)),
    });
    let group_collection =
        classify_each_collection(&group_ty).expect("grouped Vec should support each(...)");
    let group_element_ty = group_collection.element_ty;
    assert_eq!(quote!(#group_element_ty).to_string(), "i32");
}

#[test]
fn test_each_collection_classifier_covers_supported_collection_shapes() {
    let optional_std_vec: syn::Type = syn::parse_quote!(Option<std::vec::Vec<Option<i32>>>);
    let collection =
        classify_each_collection(&optional_std_vec).expect("std::vec::Vec should classify");
    assert_eq!(collection.outer_cardinality, Cardinality::Optional);
    assert_eq!(collection.element_cardinality, Cardinality::Optional);
    let element_ty = collection.element_ty;
    assert_eq!(quote!(#element_ty).to_string(), "Option < i32 >");

    let alloc_vec: syn::Type = syn::parse_quote!(alloc::vec::Vec<String>);
    let collection = classify_each_collection(&alloc_vec).expect("alloc::vec::Vec should classify");
    let element_ty = collection.element_ty;
    assert_eq!(quote!(#element_ty).to_string(), "String");

    let slice: syn::Type = syn::parse_quote!(&[u8]);
    let collection = classify_each_collection(&slice).expect("borrowed slice should classify");
    let element_ty = collection.element_ty;
    assert_eq!(quote!(#element_ty).to_string(), "u8");

    let unsupported: syn::Type = syn::parse_quote!(std::collections::HashMap<String, String>);
    assert!(classify_each_collection(&unsupported).is_err());
}

#[test]
fn test_resolve_explicit_infer_type_reports_unmatched_shapes() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct BadInfer {
            #[koruma(GenericValidation::<std::collections::HashMap<_, _>>)]
            value: Option<String>,
        }
    };

    let err = ValidationPlan::build(&input, "Koruma")
        .expect_err("expected unmatched explicit infer shape to fail");
    assert!(err.to_string().contains("cannot infer `_`"));
}
