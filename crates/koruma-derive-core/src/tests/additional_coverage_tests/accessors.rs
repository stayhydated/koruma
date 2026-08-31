use super::support::*;

#[test]
fn parsed_semantic_nodes_keep_actionable_source_markers() {
    let data_attr: DataFieldKorumaAttr = syn::parse_quote!(
        required = full(RequiredValidation::<_>),
        each(item_required = unwrapped(RequiredValidation::<_>))
    );

    let DataFieldKorumaItem::FieldValidation(field_spec) = &data_attr.items()[0] else {
        panic!("expected field validator");
    };
    assert_eq!(
        field_spec.validator().label().map(ToString::to_string),
        Some("required".to_owned())
    );
    assert!(matches!(
        field_spec.validator().target(),
        ValidatorTargetSelector::Full { .. }
    ));

    let DataFieldKorumaItem::ElementValidation(element_spec) = &data_attr.items()[1] else {
        panic!("expected element validator");
    };
    assert_eq!(element_spec.marker_source().value().to_string(), "each");
    assert_eq!(
        element_spec.validators()[0]
            .label()
            .map(ToString::to_string),
        Some("item_required".to_owned())
    );
    assert!(matches!(
        element_spec.validators()[0].target(),
        ValidatorTargetSelector::Unwrapped { .. }
    ));
}

#[test]
fn parsed_data_field_accessors_and_labels_cover_helpers() {
    let label = ValidatorLabel::new(quote::format_ident!("valid_label"))
        .expect("lower-snake label should be accepted");
    assert_eq!(label.ident().to_string(), "valid_label");

    let validator: ValidatorAttr = syn::parse_quote!(RangeValidation.min(0));
    let unlabeled = crate::ParsedValidatorUse::unlabeled(validator.clone());
    assert!(unlabeled.label().is_none());
    assert!(unlabeled.label_span().is_none());

    let labeled = crate::ParsedValidatorUse::labeled(quote::format_ident!("range"), validator)
        .expect("valid label should parse");
    assert_eq!(
        labeled.label().map(ToString::to_string),
        Some("range".to_owned())
    );
    assert!(labeled.label_span().is_some());

    let attr: DataFieldKorumaAttr =
        syn::parse_quote!(full(RequiredValidation), each(unwrapped(ItemValidation)));
    let DataFieldKorumaItem::FieldValidation(field_spec) = &attr.items()[0] else {
        panic!("expected field validation item");
    };
    assert!(field_spec.validator().target().marker_span().is_some());
    assert!(field_spec.validator().target().is_full());

    let DataFieldKorumaItem::ElementValidation(element_spec) = &attr.items()[1] else {
        panic!("expected element validation item");
    };
    assert_eq!(element_spec.marker().to_string(), "each");
    assert!(
        element_spec.validators()[0]
            .target()
            .marker_span()
            .is_some()
    );
    assert!(!element_spec.validators()[0].target().is_full());

    let skip_attr: DataFieldKorumaAttr = syn::parse_quote!(skip);
    assert!(skip_attr.is_skip());
    assert_eq!(
        skip_attr.items()[0].modifier(),
        Some(FieldModifierKind::Skip)
    );

    let nested_attr: DataFieldKorumaAttr = syn::parse_quote!(nested);
    assert!(nested_attr.is_nested());
    assert_eq!(
        nested_attr.items()[0].modifier(),
        Some(FieldModifierKind::Nested)
    );

    let field: syn::Field = syn::parse_quote! {
        #[koruma(skip)]
        skipped: String
    };
    let skipped = parse_field(&field, 3).expect("expected skip field to parse");
    assert_eq!(skipped.source().name().to_string(), "skipped");
    assert!(skipped.participating().is_none());

    let field: syn::Field = syn::parse_quote! {
        #[koruma(RangeValidation)]
        value: i32
    };
    let parsed = parse_field(&field, 4).expect("expected field to parse");
    assert_eq!(parsed.source().index(), 4);
    let participating = parsed.participating().expect("field should participate");
    assert_eq!(participating.source().name().to_string(), "value");
    assert_eq!(participating.index(), 4);
}

#[test]
fn value_field_info_wrappers_and_empty_marker_errors_are_covered() {
    let input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            #[koruma(value)]
            actual: String,
        }
    };

    let spec = parse_validator_struct(&input).expect("expected valid validator struct");
    assert_eq!(spec.value_field().name().to_string(), "actual");

    let bad_input: syn::ItemStruct = syn::parse_quote! {
        struct Validator {
            #[koruma()]
            actual: String,
        }
    };

    assert!(
        parse_validator_struct(&bad_input)
            .expect_err("expected empty marker error")
            .to_string()
            .contains(
                "validator fields must contain `value`, `skip_capture`, `setter`, or `setter(...)`",
            )
    );
}
