use super::*;

pub(crate) struct FieldTargetContext {
    field: ValueShape,
    collection: Option<CollectionShape>,
}

impl FieldTargetContext {
    pub(crate) fn new<'a>(field_ty: &Type, each_collection: Option<&EachCollection<'a>>) -> Self {
        let field = ValueShape::new(field_ty, each_collection.map(|c| c.outer_cardinality));
        let collection = each_collection.map(|collection| {
            CollectionShape::new(
                collection.outer_cardinality,
                ValueShape::new(collection.element_ty, Some(collection.element_cardinality)),
            )
        });

        Self { field, collection }
    }

    pub(crate) fn field(&self) -> &ValueShape {
        &self.field
    }

    pub(crate) fn collection(&self) -> Option<&CollectionShape> {
        self.collection.as_ref()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValueShape {
    pub raw_ty: Type,
    pub validate_ty: Type,
    pub cardinality: Cardinality,
}

impl ValueShape {
    pub(crate) fn new(raw_ty: &Type, cardinality: Option<Cardinality>) -> Self {
        Self {
            raw_ty: raw_ty.clone(),
            validate_ty: option_inner_type(raw_ty).unwrap_or(raw_ty).clone(),
            cardinality: cardinality.unwrap_or_else(|| Cardinality::for_type(raw_ty)),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum CollectionShape {
    RequiredCollection { element: ValueShape },
    OptionalCollection { element: ValueShape },
}

impl CollectionShape {
    pub(crate) fn new(cardinality: Cardinality, element: ValueShape) -> Self {
        match cardinality {
            Cardinality::Required => Self::RequiredCollection { element },
            Cardinality::Optional => Self::OptionalCollection { element },
        }
    }

    pub(crate) fn element(&self) -> &ValueShape {
        match self {
            Self::RequiredCollection { element } | Self::OptionalCollection { element } => element,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ValidationTarget {
    FieldFull(FullFieldTarget),
    FieldUnwrapped(UnwrappedFieldTarget),
    ElementFull(FullElementTarget),
    ElementUnwrapped(UnwrappedElementTarget),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ValidationTargetContext<'a> {
    Field(&'a ValueShape),
    Element(&'a ValueShape),
}

impl<'a> ValidationTargetContext<'a> {
    pub(crate) fn value(self) -> &'a ValueShape {
        match self {
            Self::Field(shape) | Self::Element(shape) => shape,
        }
    }

    pub(crate) fn is_element(self) -> bool {
        matches!(self, Self::Element(_))
    }

    pub(crate) fn description(self, field_name: &Ident) -> String {
        if self.is_element() {
            format!("element validators on field `{field_name}`")
        } else {
            format!("field `{field_name}`")
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TargetBorrow {
    Reference,
    AlreadyBorrowed,
}

#[derive(Clone, Debug)]
pub(crate) struct FullFieldTarget {
    pub ty: Type,
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "planner unit tests assert full-field cardinality")
    )]
    pub cardinality: Cardinality,
    pub borrow: TargetBorrow,
}

#[derive(Clone, Debug)]
pub(crate) struct UnwrappedFieldTarget {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "planner unit tests assert raw unwrapped field types"
        )
    )]
    pub raw_type: Type,
    pub validate_type: Type,
    pub borrow: TargetBorrow,
}

#[derive(Clone, Debug)]
pub(crate) struct FullElementTarget {
    pub ty: Type,
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "planner unit tests assert full-element cardinality"
        )
    )]
    pub cardinality: Cardinality,
    pub borrow: TargetBorrow,
}

#[derive(Clone, Debug)]
pub(crate) struct UnwrappedElementTarget {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "planner unit tests assert raw unwrapped element types"
        )
    )]
    pub raw_type: Type,
    pub validate_type: Type,
    pub borrow: TargetBorrow,
}

impl ValidationTarget {
    pub(crate) fn for_validator(
        validator: &ValidatorAttr,
        target_selector: &ValidatorTargetSelector,
        target_context: ValidationTargetContext<'_>,
        field_name: &Ident,
    ) -> Result<Self, syn::Error> {
        let value_context = target_context.value();

        Self::reject_option_target_type_arg_on_explicit_unwrapped_target(
            validator,
            target_selector,
            target_context,
            field_name,
        )?;

        let full_target_selected = target_selector.is_full()
            || Self::should_infer_full_target_from_option_type(validator, target_selector);

        if full_target_selected {
            if validator
                .explicit_type()
                .is_some_and(|ty| option_inner_type(ty).is_some())
                && option_inner_type(&value_context.raw_ty).is_none()
            {
                let target_description = target_context.description(field_name);
                return Err(syn::Error::new(
                    target_selector
                        .marker_span()
                        .unwrap_or_else(|| validator.path().span()),
                    format!(
                        "explicit `Option<...>` validator type arguments require an optional validation target for {target_description}; `{}` is targeting a non-optional value",
                        validator.path_name()
                    ),
                ));
            }

            return Ok(match target_context {
                ValidationTargetContext::Field(_) => Self::FieldFull(FullFieldTarget {
                    ty: value_context.raw_ty.clone(),
                    cardinality: value_context.cardinality,
                    borrow: TargetBorrow::Reference,
                }),
                ValidationTargetContext::Element(_) => Self::ElementFull(FullElementTarget {
                    ty: value_context.raw_ty.clone(),
                    cardinality: value_context.cardinality,
                    borrow: TargetBorrow::AlreadyBorrowed,
                }),
            });
        }

        Ok(match target_context {
            ValidationTargetContext::Field(_) => Self::FieldUnwrapped(UnwrappedFieldTarget {
                raw_type: value_context.raw_ty.clone(),
                validate_type: value_context.validate_ty.clone(),
                borrow: TargetBorrow::AlreadyBorrowed,
            }),
            ValidationTargetContext::Element(_) => Self::ElementUnwrapped(UnwrappedElementTarget {
                raw_type: value_context.raw_ty.clone(),
                validate_type: value_context.validate_ty.clone(),
                borrow: TargetBorrow::AlreadyBorrowed,
            }),
        })
    }

    pub(crate) fn should_infer_full_target_from_option_type(
        validator: &ValidatorAttr,
        target_selector: &ValidatorTargetSelector,
    ) -> bool {
        matches!(target_selector, ValidatorTargetSelector::Default)
            && validator
                .explicit_type()
                .is_some_and(|ty| option_inner_type(ty).is_some())
    }

    pub(crate) fn reject_option_target_type_arg_on_explicit_unwrapped_target(
        validator: &ValidatorAttr,
        target_selector: &ValidatorTargetSelector,
        target_context: ValidationTargetContext<'_>,
        field_name: &Ident,
    ) -> Result<(), syn::Error> {
        let Some(explicit_ty) = validator.explicit_type() else {
            return Ok(());
        };
        if option_inner_type(explicit_ty).is_none() {
            return Ok(());
        }
        if target_selector.is_full() || matches!(target_selector, ValidatorTargetSelector::Default)
        {
            return Ok(());
        }

        let target_description = target_context.description(field_name);
        let validator_name = validator.path_name();

        Err(syn::Error::new_spanned(
            explicit_ty,
            format!(
                "explicit `Option<...>` validator type arguments require full-option target selection for {target_description}; remove `unwrapped(...)`, write `{validator_name}::<Option<_>>`, or use `full({validator_name}::<_>)`"
            ),
        ))
    }

    pub(crate) fn resolve_explicit_infer_type(
        &self,
        validator: &ValidatorAttr,
    ) -> Result<Option<Type>, syn::Error> {
        let Some(explicit_ty) = validator.explicit_type() else {
            return Ok(None);
        };

        if !contains_infer_type(explicit_ty) {
            return Ok(None);
        }

        let validate_type = self.validate_type();
        substitute_infer_type_from_source(explicit_ty, validate_type)
            .map(Some)
            .ok_or_else(|| {
                let rendered_explicit = quote! { #explicit_ty }.to_string();
                let infer_source = validate_type;
                let rendered_source = quote! { #infer_source }.to_string();
                syn::Error::new_spanned(
                    explicit_ty,
                    format!(
                        "cannot infer `_` in `{rendered_explicit}` from `{rendered_source}`; use concrete type arguments or a matching generic shape"
                    ),
                )
            })
    }

    pub(crate) fn validate_type(&self) -> &Type {
        match self {
            Self::FieldFull(target) => &target.ty,
            Self::FieldUnwrapped(target) => &target.validate_type,
            Self::ElementFull(target) => &target.ty,
            Self::ElementUnwrapped(target) => &target.validate_type,
        }
    }

    pub(crate) fn borrow(&self) -> TargetBorrow {
        match self {
            Self::FieldFull(target) => target.borrow,
            Self::FieldUnwrapped(target) => target.borrow,
            Self::ElementFull(target) => target.borrow,
            Self::ElementUnwrapped(target) => target.borrow,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum PlannedValidatorTypeArg {
    None,
    Resolved(
        #[cfg_attr(
            not(test),
            expect(
                dead_code,
                reason = "planner unit tests inspect resolved type arguments"
            )
        )]
        Box<Type>,
    ),
}

impl PlannedValidatorTypeArg {
    pub(crate) fn for_validator(
        validator: &ValidatorAttr,
        target: &ValidationTarget,
        resolved_explicit_type: Option<Type>,
    ) -> Self {
        if let Some(resolved) = resolved_explicit_type {
            return Self::Resolved(Box::new(resolved));
        }

        if let Some(explicit) = validator.explicit_type() {
            return Self::Resolved(Box::new(explicit.clone()));
        }

        if validator.uses_type_inference() {
            return Self::Resolved(Box::new(target.validate_type().clone()));
        }

        Self::None
    }
}
