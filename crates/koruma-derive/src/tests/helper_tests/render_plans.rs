use super::support::*;

#[test]
fn test_validation_plan_exposes_render_ready_validation_operations() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(full(RequiredValidation::<_>), LengthValidation::<_>.min(1))]
            name: Option<String>,
            #[koruma(each(full(RequiredValidation::<_>), ItemLength::<_>.min(1)))]
            tags: Vec<Option<String>>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let render_plan = plan.validation_render_plan();
    insta::assert_debug_snapshot!(
        validation_render_plan_summary(&render_plan),
        @r###"
    [
        (
            "regular_optional",
            "name",
            1,
            1,
            "none",
        ),
        (
            "regular_required",
            "tags",
            0,
            0,
            "required_collection_optional_element(full=1, unwrapped=1)",
        ),
    ]
    "###
    );
    let operations = &render_plan.operations;
    assert_eq!(operations.len(), 2);

    let PlannedValidationOperation::RegularOptional(name_operation) = &operations[0] else {
        panic!("expected optional regular validation operation");
    };
    assert_eq!(name_operation.field.name.to_string(), "name");
    assert_eq!(
        name_operation.field_validators.full_type_validators.len(),
        1
    );
    assert_eq!(
        name_operation.field_validators.unwrapped_validators.len(),
        1
    );
    assert!(name_operation.element_validators.is_none());

    let PlannedValidationOperation::RegularRequired(tags_operation) = &operations[1] else {
        panic!("expected required regular validation operation");
    };
    assert!(!tags_operation.field_validators.has_any());
    let element = tags_operation
        .element_validators
        .as_ref()
        .expect("expected element operation");
    let PlannedElementValidation::RequiredCollectionOptional(element) = element else {
        panic!("expected optional element validation operation");
    };
    assert_eq!(element.full_type_validators.len(), 1);
    assert_eq!(element.unwrapped_validators.len(), 1);
}

fn validation_render_plan_summary(
    render_plan: &crate::expand::plan::ValidationRenderPlan<'_>,
) -> Vec<(&'static str, String, usize, usize, String)> {
    render_plan
        .operations
        .iter()
        .map(|operation| match operation {
            PlannedValidationOperation::NestedRequired(operation) => (
                "nested_required",
                operation.field.name.to_string(),
                0,
                0,
                "none".to_owned(),
            ),
            PlannedValidationOperation::NestedOptional(operation) => (
                "nested_optional",
                operation.field.name.to_string(),
                0,
                0,
                "none".to_owned(),
            ),
            PlannedValidationOperation::NewtypeRequired(operation) => (
                "newtype_required",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                "none".to_owned(),
            ),
            PlannedValidationOperation::NewtypeOptional(operation) => (
                "newtype_optional",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                "none".to_owned(),
            ),
            PlannedValidationOperation::RegularRequired(operation) => (
                "regular_required",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                element_summary(operation.element_validators.as_ref()),
            ),
            PlannedValidationOperation::RegularOptional(operation) => (
                "regular_optional",
                operation.field.name.to_string(),
                operation.field_validators.full_type_validators.len(),
                operation.field_validators.unwrapped_validators.len(),
                element_summary(operation.element_validators.as_ref()),
            ),
        })
        .collect()
}

fn element_summary(element: Option<&PlannedElementValidation<'_>>) -> String {
    match element {
        None => "none".to_owned(),
        Some(PlannedElementValidation::RequiredCollectionRequired(element)) => format!(
            "required_collection_required_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
        Some(PlannedElementValidation::RequiredCollectionOptional(element)) => format!(
            "required_collection_optional_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
        Some(PlannedElementValidation::OptionalCollectionRequired(element)) => format!(
            "optional_collection_required_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
        Some(PlannedElementValidation::OptionalCollectionOptional(element)) => format!(
            "optional_collection_optional_element(full={}, unwrapped={})",
            element.full_type_validators.len(),
            element.unwrapped_validators.len()
        ),
    }
}

#[test]
fn test_validation_plan_exposes_main_error_render_plan() {
    let input: syn::DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        struct Planned(#[koruma(nested)] Child);
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let layout = plan.main_error_render_plan();
    assert_eq!(layout.fields.len(), 1);
    assert_eq!(layout.fields[0].field.name.to_string(), "_0");
    assert_eq!(
        layout.fields[0].storage,
        PlannedMainErrorStorage::NestedDirect
    );
    assert_eq!(layout.fields[0].getter, PlannedErrorGetter::NestedDirect);
    assert_eq!(layout.fields[0].default, PlannedErrorDefault::NestedDirect);
    assert_eq!(layout.fields[0].is_empty, PlannedErrorIsEmpty::NestedDirect);

    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(nested)]
            child: Option<Child>,
            #[koruma(newtype)]
            raw: Wrapper,
            #[koruma(newtype)]
            wrapper: Option<Wrapper>,
            #[koruma(newtype, RequiredValidation)]
            checked: Wrapper,
            #[koruma(newtype, RequiredValidation)]
            maybe_checked: Option<Wrapper>,
            #[koruma(RangeValidation.min(0), each(ItemValidation))]
            values: Vec<i32>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let layout = plan.main_error_render_plan();
    assert_eq!(layout.fields.len(), 6);

    assert_eq!(
        layout.fields[0].storage,
        PlannedMainErrorStorage::NestedOptional
    );
    assert_eq!(layout.fields[0].getter, PlannedErrorGetter::NestedOptional);
    assert_eq!(layout.fields[0].default, PlannedErrorDefault::None);
    assert_eq!(
        layout.fields[0].is_empty,
        PlannedErrorIsEmpty::NestedOptional
    );

    assert_eq!(
        layout.fields[1].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(
        layout.fields[1].getter,
        PlannedErrorGetter::NewtypeInnerDirect
    );
    assert_eq!(
        layout.fields[1].default,
        PlannedErrorDefault::FieldErrorDefault
    );
    assert_eq!(layout.fields[1].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[2].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(
        layout.fields[2].getter,
        PlannedErrorGetter::NewtypeInnerOptional
    );
    assert_eq!(
        layout.fields[2].default,
        PlannedErrorDefault::FieldErrorDefault
    );
    assert_eq!(layout.fields[2].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[3].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(layout.fields[3].getter, PlannedErrorGetter::FieldError);
    assert_eq!(
        layout.fields[3].default,
        PlannedErrorDefault::NewtypeWithValidators {
            inner_optional: false,
        }
    );
    assert_eq!(layout.fields[3].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[4].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(layout.fields[4].getter, PlannedErrorGetter::FieldError);
    assert_eq!(
        layout.fields[4].default,
        PlannedErrorDefault::NewtypeWithValidators {
            inner_optional: true,
        }
    );
    assert_eq!(layout.fields[4].is_empty, PlannedErrorIsEmpty::FieldError);

    assert_eq!(
        layout.fields[5].storage,
        PlannedMainErrorStorage::FieldError
    );
    assert_eq!(layout.fields[5].getter, PlannedErrorGetter::FieldError);
    assert_eq!(
        layout.fields[5].default,
        PlannedErrorDefault::Regular {
            has_field_validators: true,
            has_element_validators: true,
        }
    );
    assert_eq!(layout.fields[5].is_empty, PlannedErrorIsEmpty::FieldError);
}

#[test]
fn test_validation_plan_exposes_field_error_render_plan() {
    let input: syn::DeriveInput = syn::parse_quote! {
        struct Planned {
            #[koruma(nested)]
            child: Child,
            #[koruma(newtype)]
            raw: Wrapper,
            #[koruma(newtype)]
            wrapper: Option<Wrapper>,
            #[koruma(newtype, RequiredValidation)]
            checked: Wrapper,
            #[koruma(RequiredValidation)]
            name: String,
            #[koruma(each(ItemValidation))]
            tags: Vec<String>,
            #[koruma(LengthValidation.min(1), each(ItemValidation))]
            values: Vec<String>,
        }
    };

    let plan = ValidationPlan::build(&input, "Koruma").expect("expected plan");
    let layout = plan.field_error_render_plan();
    assert_eq!(layout.fields.len(), 6);

    assert_eq!(layout.fields[0].field().name.to_string(), "raw");
    assert_eq!(
        layout.fields[0].shape(),
        FieldErrorShape::NewtypeInnerRequired
    );

    assert_eq!(layout.fields[1].field().name.to_string(), "wrapper");
    assert_eq!(
        layout.fields[1].shape(),
        FieldErrorShape::NewtypeInnerOptional
    );

    assert_eq!(layout.fields[2].field().name.to_string(), "checked");
    assert_eq!(
        layout.fields[2].shape(),
        FieldErrorShape::NewtypeWithValidatorsRequired
    );
    assert_eq!(layout.fields[2].field_validators().len(), 1);

    assert_eq!(layout.fields[3].field().name.to_string(), "name");
    assert_eq!(layout.fields[3].shape(), FieldErrorShape::RegularFieldOnly);

    assert_eq!(layout.fields[4].field().name.to_string(), "tags");
    assert_eq!(
        layout.fields[4].shape(),
        FieldErrorShape::RegularElementOnly
    );
    assert_eq!(layout.fields[4].element_validators().len(), 1);

    assert_eq!(layout.fields[5].field().name.to_string(), "values");
    assert_eq!(
        layout.fields[5].shape(),
        FieldErrorShape::RegularFieldAndElement
    );
    assert_eq!(layout.fields[5].field_validators().len(), 1);
    assert_eq!(layout.fields[5].element_validators().len(), 1);
}
