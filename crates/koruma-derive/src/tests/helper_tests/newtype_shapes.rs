use super::support::*;

#[test]
fn test_validation_plan_uses_shape_specific_field_data() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct PlannedShapes {
            #[koruma(nested)]
            child: Child,
            #[koruma(newtype, RequiredValidation)]
            wrapped: Option<Wrapped>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    assert_eq!(plan.fields.len(), 2);

    let PlannedField::Nested(nested) = &plan.fields[0].shape else {
        panic!("expected nested planned field");
    };
    assert_eq!(nested.cardinality, Cardinality::Required);
    let nested_inner_type = &nested.inner_type;
    assert_eq!(quote!(#nested_inner_type).to_string(), "Child");
    assert!(plan.fields[0].field_validators().is_empty());
    assert!(plan.fields[0].element_validators().is_empty());
    assert!(matches!(
        plan.fields[0].error_storage(),
        ErrorStorage::Nested {
            cardinality: Cardinality::Required
        }
    ));

    let PlannedField::Newtype(newtype) = &plan.fields[1].shape else {
        panic!("expected newtype planned field");
    };
    assert_eq!(newtype.cardinality, Cardinality::Optional);
    let newtype_inner_type = &newtype.inner_type;
    assert_eq!(quote!(#newtype_inner_type).to_string(), "Wrapped");
    assert_eq!(newtype.field_validators.len(), 1);
    assert!(plan.fields[1].element_validators().is_empty());
    assert!(matches!(
        plan.fields[1].error_storage(),
        ErrorStorage::NewtypeWithValidators {
            cardinality: Cardinality::Optional
        }
    ));
}

#[test]
fn test_validation_plan_encodes_struct_level_newtype_shape() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        struct Email(#[koruma(newtype)] InnerEmail);
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let Some(field_plan) = plan.struct_newtype() else {
        panic!("expected struct-level newtype plan");
    };
    assert_eq!(plan.fields.len(), 1);
    assert!(matches!(
        plan.struct_plan,
        StructPlan::Newtype { field_index: 0 }
    ));
    assert_eq!(field_plan.name.to_string(), "_0");
    assert_eq!(field_plan.source.index, 0);
    assert!(field_plan.is_newtype());
}
