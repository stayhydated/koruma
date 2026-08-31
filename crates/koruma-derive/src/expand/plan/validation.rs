use super::*;

#[derive(Clone, Debug)]
pub(crate) struct ValidationPlan {
    pub struct_options: StructOptions,
    pub struct_plan: StructPlan,
    pub main_error_struct: Ident,
    pub fields: Vec<FieldPlan>,
}

impl ValidationPlan {
    pub fn build(input: &DeriveInput, derive_name: &str) -> Result<Self, syn::Error> {
        let struct_options = parse_struct_options(&input.attrs)?;
        let fields = struct_fields(input, derive_name)?;
        let field_infos = collect_field_infos(fields, Some(&struct_options))?;
        let total_fields = fields.len();

        if matches!(struct_options.mode(), StructMode::Newtype { .. }) && total_fields != 1 {
            return Err(syn::Error::new_spanned(
                input,
                format!("newtype structs must have exactly one field, found {total_fields}"),
            ));
        }

        if struct_options.constructors().try_from() && total_fields != 1 {
            return Err(syn::Error::new_spanned(
                input,
                "try_from requires exactly one field; use try_new for multi-field constructors",
            ));
        }

        let known_field_names: Vec<Ident> = fields
            .iter()
            .filter_map(|field| field.ident.clone())
            .collect();
        let generated_api = GeneratedDeriveApi::build(&input.ident, &field_infos)?;

        let mut plan_errors = ErrorBag::new();
        let mut planned_fields = Vec::new();
        for field in &field_infos {
            if let Some(field) = plan_errors.push_result(FieldPlan::build(
                field,
                generated_api.field_names(field),
                &known_field_names,
            )) {
                planned_fields.push(field);
            }
        }
        plan_errors.finish()?;

        let struct_plan = match struct_options.mode() {
            StructMode::Newtype { .. } => {
                if planned_fields.is_empty() {
                    return Err(syn::Error::new_spanned(
                        input,
                        "newtype structs require a field validation plan",
                    ));
                }
                StructPlan::Newtype { field_index: 0 }
            },
            StructMode::Regular => match fields {
                Fields::Named(_) => StructPlan::Record,
                Fields::Unnamed(_) => StructPlan::Tuple,
                Fields::Unit => StructPlan::Unit,
            },
        };

        Ok(Self {
            struct_options,
            struct_plan,
            main_error_struct: generated_api.main_error_struct,
            fields: planned_fields,
        })
    }

    pub fn struct_newtype(&self) -> Option<&FieldPlan> {
        match &self.struct_plan {
            StructPlan::Newtype { field_index } => self.fields.get(*field_index),
            StructPlan::Record | StructPlan::Tuple | StructPlan::Unit => None,
        }
    }

    pub(crate) fn validation_render_plan(&self) -> ValidationRenderPlan<'_> {
        let struct_is_newtype = self.struct_newtype().is_some();

        let operations = self
            .fields
            .iter()
            .map(|field| {
                if field.is_nested() {
                    let operation = PlannedNestedValidation {
                        field,
                        direct_storage: struct_is_newtype,
                    };
                    return if field.field_optional() {
                        PlannedValidationOperation::NestedOptional(operation)
                    } else {
                        PlannedValidationOperation::NestedRequired(operation)
                    };
                }

                if field.is_newtype() {
                    let operation = PlannedNewtypeValidation {
                        field,
                        field_validators: PlannedFieldValidatorGroups::for_field(field),
                    };
                    return if field.field_optional() {
                        PlannedValidationOperation::NewtypeOptional(operation)
                    } else {
                        PlannedValidationOperation::NewtypeRequired(operation)
                    };
                }

                let operation = PlannedRegularValidation {
                    field,
                    field_validators: PlannedFieldValidatorGroups::for_field(field),
                    element_validators: field
                        .has_element_validators()
                        .then(|| PlannedElementValidation::for_field(field)),
                };

                if field.field_optional() {
                    PlannedValidationOperation::RegularOptional(operation)
                } else {
                    PlannedValidationOperation::RegularRequired(operation)
                }
            })
            .collect();

        ValidationRenderPlan { operations }
    }

    pub(crate) fn main_error_render_plan(&self) -> MainErrorRenderPlan<'_> {
        let struct_is_newtype = self.struct_newtype().is_some();
        let fields = self
            .fields
            .iter()
            .map(|field| PlannedMainErrorField::for_field(field, struct_is_newtype))
            .collect();

        MainErrorRenderPlan { fields }
    }

    pub(crate) fn field_error_render_plan(&self) -> FieldErrorRenderPlan<'_> {
        let fields = self
            .fields
            .iter()
            .filter_map(PlannedFieldError::for_field)
            .collect();

        FieldErrorRenderPlan { fields }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ValidationRenderPlan<'a> {
    pub operations: Vec<PlannedValidationOperation<'a>>,
}

pub(crate) fn struct_fields<'a>(
    input: &'a DeriveInput,
    derive_name: &str,
) -> Result<&'a Fields, syn::Error> {
    match &input.data {
        syn::Data::Struct(data) => Ok(&data.fields),
        _ => Err(syn::Error::new_spanned(
            input,
            format!("{derive_name} can only be derived for structs"),
        )),
    }
}
