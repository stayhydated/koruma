use super::*;

pub(crate) struct ValidatorBuilderPlan {
    pub(crate) struct_name: Ident,
    pub(crate) input: ItemStruct,
    pub(crate) builder_name: Ident,
    pub(crate) module_name: Ident,
    pub(crate) slots: Vec<BuilderSlot>,
    pub(crate) direct_methods: Vec<DirectBuilderMethodPlan>,
    pub(crate) generic_args: Vec<TokenStream2>,
    pub(crate) capture_policy: CapturePolicy,
    pub(crate) value_slot_index: usize,
}

impl ValidatorBuilderPlan {
    pub(crate) fn build(
        input: &ItemStruct,
        validator_spec: &ValidatorStructSpec,
    ) -> Result<Self, syn::Error> {
        let struct_name = input.ident.clone();
        let builder_name = format_ident!("{}Builder", struct_name);
        let module_name = format_ident!("{}_builder", struct_name.to_string().to_snake_case());
        reject_builder_helper_name_collisions(&builder_name, &module_name)?;
        let slots = builder_slots(validator_spec, &input.generics)?;
        let direct_methods = direct_method_plans(&slots);
        let capture_policy = validator_spec.value_spec().capture();
        let value_slot_index = validator_spec.value_index();
        let Some(value_slot) = slots.get(value_slot_index) else {
            return Err(syn::Error::new_spanned(
                input,
                "internal error: validator value field did not produce a builder slot",
            ));
        };

        if capture_policy == CapturePolicy::Skip && option_inner_type(value_slot.ty()).is_none() {
            return Err(syn::Error::new_spanned(
                value_slot.ty(),
                "`#[koruma(skip_capture)]` requires an `Option<T>` field",
            ));
        }

        Ok(Self {
            struct_name,
            input: input.clone(),
            builder_name,
            module_name,
            slots,
            direct_methods,
            generic_args: generic_args(&input.generics),
            capture_policy,
            value_slot_index,
        })
    }

    pub(crate) fn slots(&self) -> &[BuilderSlot] {
        &self.slots
    }

    pub(crate) fn direct_methods(&self) -> &[DirectBuilderMethodPlan] {
        &self.direct_methods
    }

    pub(crate) fn value_slot(&self) -> &BuilderSlot {
        &self.slots[self.value_slot_index]
    }

    pub(crate) fn value_inner_type(&self) -> &Type {
        option_inner_type(self.value_slot().ty()).unwrap_or(self.value_slot().ty())
    }

    pub(crate) fn required_slots(&self) -> Vec<&BuilderSlot> {
        self.slots
            .iter()
            .filter(|slot| slot.is_required())
            .collect()
    }

    pub(crate) fn initial_state_args(&self) -> Vec<TokenStream2> {
        self.required_slots()
            .iter()
            .map(|_| {
                let module_name = &self.module_name;
                quote! { #module_name::Empty }
            })
            .collect()
    }

    pub(crate) fn set_state_args(&self) -> Vec<TokenStream2> {
        self.required_slots()
            .iter()
            .map(|_| {
                let module_name = &self.module_name;
                quote! { #module_name::Set }
            })
            .collect()
    }

    pub(crate) fn builder_type_path_with_states(
        &self,
        state_args: &[TokenStream2],
    ) -> TokenStream2 {
        builder_type_path(&self.builder_name, &self.generic_args, state_args)
    }

    pub(crate) fn value_builder_type(&self) -> TokenStream2 {
        let value_field_name = self.value_slot().ident();
        let state_args: Vec<_> = self
            .required_slots()
            .iter()
            .map(|slot| {
                let module_name = &self.module_name;
                if slot.ident() == value_field_name {
                    quote! { #module_name::Set }
                } else {
                    quote! { #module_name::Empty }
                }
            })
            .collect();

        self.builder_type_path_with_states(&state_args)
    }

    pub(crate) fn builder_type_with_replaced_state(&self, target: &Ident) -> TokenStream2 {
        builder_type_with_replaced_state(
            &self.builder_name,
            &self.generic_args,
            &self.module_name,
            &self.slots,
            target,
        )
    }

    #[cfg(test)]
    pub(crate) fn test_build(input: &ItemStruct) -> Result<Self, syn::Error> {
        let validator_spec = parse_validator_struct(input)?;
        Self::build(input, &validator_spec)
    }

    #[cfg(test)]
    pub(crate) fn slot_summaries(&self) -> Vec<BuilderSlotSummary> {
        self.slots
            .iter()
            .map(BuilderSlotSummary::for_slot)
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn direct_method_summaries(&self) -> Vec<(String, Option<String>)> {
        self.direct_methods
            .iter()
            .map(|method| {
                (
                    method.setter.method.to_string(),
                    method.maybe_method.as_ref().map(ToString::to_string),
                )
            })
            .collect()
    }

    #[cfg(any(test, feature = "internal-showcase"))]
    pub(crate) fn capture_policy(&self) -> CapturePolicy {
        self.capture_policy
    }
}

pub(crate) struct DirectBuilderMethodPlan {
    pub(crate) setter: OwnedSetterRenderPlan,
    pub(crate) maybe_method: Option<Ident>,
}

pub(crate) fn direct_method_plans(slots: &[BuilderSlot]) -> Vec<DirectBuilderMethodPlan> {
    slots
        .iter()
        .filter_map(|slot| {
            let parts = slot.setter_render_parts()?;
            let maybe_method = parts
                .maybe_inner_ty
                .map(|_| format_ident!("maybe_{}", parts.method));
            let setter = OwnedSetterRenderPlan::from(parts);
            Some(DirectBuilderMethodPlan {
                setter,
                maybe_method,
            })
        })
        .collect()
}

#[cfg(test)]
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct BuilderSlotSummary {
    pub name: String,
    pub kind: &'static str,
    pub required: bool,
    pub state_ident: Option<String>,
    pub method: Option<String>,
    pub signature: Option<String>,
    pub maybe_method: Option<String>,
}

#[cfg(test)]
impl BuilderSlotSummary {
    pub(crate) fn for_slot(slot: &BuilderSlot) -> Self {
        let method = slot.setter_method().map(ToString::to_string);
        let maybe_method = match slot {
            BuilderSlot::OptionalSetter(slot) if slot.maybe_inner_ty.is_some() => {
                Some(format!("maybe_{}", slot.method))
            },
            _ => None,
        };

        Self {
            name: slot.ident().to_string(),
            kind: slot.kind_name(),
            required: slot.is_required(),
            state_ident: slot.required_state_ident().map(|ident| ident.to_string()),
            method,
            signature: slot.setter_signature().map(signature_summary),
            maybe_method,
        }
    }
}

#[cfg(test)]
pub(crate) fn signature_summary(signature: &SetterSignature) -> String {
    match signature {
        SetterSignature::Exact(ty) => format!("exact({})", quote!(#ty)),
        SetterSignature::Into(ty) => format!("into({})", quote!(#ty)),
        SetterSignature::OptionalInner { inner, into } => {
            format!("optional_inner({}, into={into})", quote!(#inner))
        },
        SetterSignature::OptionalExact { option_ty } => {
            format!("optional_exact({})", quote!(#option_ty))
        },
    }
}

pub(crate) enum BuilderSlot {
    CapturedValue(CapturedValueSlot),
    SkippedValue(SkippedValueSlot),
    RequiredSetter(RequiredSetterSlot),
    OptionalSetter(Box<OptionalSetterSlot>),
    DefaultedSetter(DefaultedSetterSlot),
}

pub(crate) struct CapturedValueSlot {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) state_ident: Ident,
}

pub(crate) struct SkippedValueSlot {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
}

pub(crate) struct RequiredSetterSlot {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) method: Ident,
    pub(crate) signature: SetterSignature,
    pub(crate) state_ident: Ident,
}

pub(crate) struct OptionalSetterSlot {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) method: Ident,
    pub(crate) signature: SetterSignature,
    pub(crate) maybe_inner_ty: Option<Type>,
}

pub(crate) struct DefaultedSetterSlot {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) method: Ident,
    pub(crate) signature: SetterSignature,
    pub(crate) default: SetterDefaultValue,
}

pub(crate) enum SetterDefaultValue {
    Default,
    Expr(Box<syn::Expr>),
}

pub(crate) struct SetterRenderParts<'a> {
    pub(crate) ident: &'a Ident,
    pub(crate) method: &'a Ident,
    pub(crate) signature: &'a SetterSignature,
    pub(crate) required: bool,
    pub(crate) maybe_inner_ty: Option<&'a Type>,
}

pub(crate) struct OwnedSetterRenderPlan {
    pub(crate) ident: Ident,
    pub(crate) method: Ident,
    pub(crate) signature: SetterSignature,
    pub(crate) required: bool,
    pub(crate) maybe_inner_ty: Option<Type>,
}

impl From<SetterRenderParts<'_>> for OwnedSetterRenderPlan {
    fn from(parts: SetterRenderParts<'_>) -> Self {
        Self {
            ident: parts.ident.clone(),
            method: parts.method.clone(),
            signature: parts.signature.clone(),
            required: parts.required,
            maybe_inner_ty: parts.maybe_inner_ty.cloned(),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum SetterSignature {
    Exact(Type),
    Into(Type),
    OptionalInner { inner: Type, into: bool },
    OptionalExact { option_ty: Type },
}

impl SetterSignature {
    pub(crate) fn descriptor_type(&self) -> &Type {
        match self {
            Self::Exact(ty) | Self::Into(ty) => ty,
            Self::OptionalInner { inner, .. } => inner,
            Self::OptionalExact { option_ty } => option_ty,
        }
    }
}

pub(crate) enum ValidatorSlotSpec {
    CapturedValue {
        ident: Ident,
        ty: Type,
    },
    SkippedValue {
        ident: Ident,
        ty: Type,
    },
    RequiredSetter {
        ident: Ident,
        ty: Type,
        method: Ident,
        input: SetterInputPolicy,
    },
    OptionalSetter {
        ident: Ident,
        ty: Type,
        method: Ident,
        input: SetterInputPolicy,
    },
    DefaultedSetter {
        ident: Ident,
        ty: Type,
        method: Ident,
        input: SetterInputPolicy,
        default: SetterDefault,
    },
}

impl ValidatorSlotSpec {
    pub(crate) fn for_field(field: &koruma_derive_core::ValidatorFieldSpec) -> Self {
        let ident = field.name().clone();
        let ty = field.ty().clone();

        match field.role() {
            ValidatorFieldRole::Value(value) => match value.capture() {
                CapturePolicy::CloneInput => Self::CapturedValue { ident, ty },
                CapturePolicy::Skip => Self::SkippedValue { ident, ty },
            },
            ValidatorFieldRole::Setter(setter) => match setter.presence() {
                SetterPresence::Required => Self::RequiredSetter {
                    ident,
                    ty,
                    method: setter.method().clone(),
                    input: setter.input(),
                },
                SetterPresence::Defaulted(default) => Self::DefaultedSetter {
                    ident,
                    ty,
                    method: setter.method().clone(),
                    input: setter.input(),
                    default: default.clone(),
                },
                SetterPresence::Optional if option_inner_type(field.ty()).is_some() => {
                    Self::OptionalSetter {
                        ident,
                        ty,
                        method: setter.method().clone(),
                        input: setter.input(),
                    }
                },
                SetterPresence::Optional => Self::RequiredSetter {
                    ident,
                    ty,
                    method: setter.method().clone(),
                    input: setter.input(),
                },
            },
        }
    }
}

impl BuilderSlot {
    #[cfg(test)]
    pub(crate) fn kind_name(&self) -> &'static str {
        match self {
            Self::CapturedValue(_) => "captured_value",
            Self::SkippedValue(_) => "skipped_value",
            Self::RequiredSetter(_) => "required_setter",
            Self::OptionalSetter(_) => "optional_setter",
            Self::DefaultedSetter(_) => "defaulted_setter",
        }
    }

    pub(crate) fn ident(&self) -> &Ident {
        match self {
            Self::CapturedValue(slot) => &slot.ident,
            Self::SkippedValue(slot) => &slot.ident,
            Self::RequiredSetter(slot) => &slot.ident,
            Self::OptionalSetter(slot) => &slot.ident,
            Self::DefaultedSetter(slot) => &slot.ident,
        }
    }

    pub(crate) fn ty(&self) -> &Type {
        match self {
            Self::CapturedValue(slot) => &slot.ty,
            Self::SkippedValue(slot) => &slot.ty,
            Self::RequiredSetter(slot) => &slot.ty,
            Self::OptionalSetter(slot) => &slot.ty,
            Self::DefaultedSetter(slot) => &slot.ty,
        }
    }

    pub(crate) fn is_required(&self) -> bool {
        matches!(self, Self::CapturedValue(_) | Self::RequiredSetter(_))
    }

    pub(crate) fn required_state_ident(&self) -> Option<Ident> {
        match self {
            Self::CapturedValue(slot) => Some(slot.state_ident.clone()),
            Self::RequiredSetter(slot) => Some(slot.state_ident.clone()),
            Self::SkippedValue(_) | Self::OptionalSetter(_) | Self::DefaultedSetter(_) => None,
        }
    }

    pub(crate) fn setter_method(&self) -> Option<&Ident> {
        match self {
            Self::RequiredSetter(slot) => Some(&slot.method),
            Self::OptionalSetter(slot) => Some(&slot.method),
            Self::DefaultedSetter(slot) => Some(&slot.method),
            Self::CapturedValue(_) | Self::SkippedValue(_) => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn setter_signature(&self) -> Option<&SetterSignature> {
        match self {
            Self::RequiredSetter(slot) => Some(&slot.signature),
            Self::OptionalSetter(slot) => Some(&slot.signature),
            Self::DefaultedSetter(slot) => Some(&slot.signature),
            Self::CapturedValue(_) | Self::SkippedValue(_) => None,
        }
    }

    pub(crate) fn setter_render_parts(&self) -> Option<SetterRenderParts<'_>> {
        match self {
            Self::RequiredSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                method: &slot.method,
                signature: &slot.signature,
                required: true,
                maybe_inner_ty: None,
            }),
            Self::OptionalSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                method: &slot.method,
                signature: &slot.signature,
                required: false,
                maybe_inner_ty: slot.maybe_inner_ty.as_ref(),
            }),
            Self::DefaultedSetter(slot) => Some(SetterRenderParts {
                ident: &slot.ident,
                method: &slot.method,
                signature: &slot.signature,
                required: false,
                maybe_inner_ty: None,
            }),
            Self::CapturedValue(_) | Self::SkippedValue(_) => None,
        }
    }
}

pub(crate) fn builder_slots(
    validator_spec: &koruma_derive_core::ValidatorStructSpec,
    generics: &syn::Generics,
) -> Result<Vec<BuilderSlot>, syn::Error> {
    let mut generated_names = user_generic_namespace(generics);
    let mut slots = Vec::new();

    for field in validator_spec.fields() {
        if field.name() == "_state" {
            return Err(syn::Error::new(
                field.name().span(),
                "validator field name `_state` is reserved for the generated builder",
            ));
        }

        let slot = match ValidatorSlotSpec::for_field(field) {
            ValidatorSlotSpec::CapturedValue { ident, ty } => {
                let state_ident = state_ident_for(&ident);
                reject_state_ident_collision(&mut generated_names, &state_ident)?;
                BuilderSlot::CapturedValue(CapturedValueSlot {
                    ident,
                    ty,
                    state_ident,
                })
            },
            ValidatorSlotSpec::SkippedValue { ident, ty } => {
                BuilderSlot::SkippedValue(SkippedValueSlot { ident, ty })
            },
            ValidatorSlotSpec::RequiredSetter {
                ident,
                ty,
                method,
                input,
            } => {
                let state_ident = state_ident_for(&ident);
                reject_state_ident_collision(&mut generated_names, &state_ident)?;
                let signature = required_setter_signature(&ty, input);
                BuilderSlot::RequiredSetter(RequiredSetterSlot {
                    ident,
                    ty,
                    method,
                    signature,
                    state_ident,
                })
            },
            ValidatorSlotSpec::OptionalSetter {
                ident,
                ty,
                method,
                input,
            } => {
                let (signature, maybe_inner_ty) = optional_setter_signature(&ty, input);
                BuilderSlot::OptionalSetter(Box::new(OptionalSetterSlot {
                    ident,
                    ty,
                    method,
                    signature,
                    maybe_inner_ty,
                }))
            },
            ValidatorSlotSpec::DefaultedSetter {
                ident,
                ty,
                method,
                input,
                default,
            } => {
                let signature = defaulted_setter_signature(&ty, input);
                BuilderSlot::DefaultedSetter(DefaultedSetterSlot {
                    ident,
                    ty,
                    method,
                    signature,
                    default: setter_default_value(&default),
                })
            },
        };
        slots.push(slot);
    }

    reject_generated_method_collisions(&slots)?;
    Ok(slots)
}

pub(crate) fn required_setter_signature(ty: &Type, input: SetterInputPolicy) -> SetterSignature {
    if option_inner_type(ty).is_some() {
        SetterSignature::OptionalExact {
            option_ty: ty.clone(),
        }
    } else if input.accepts_into() {
        SetterSignature::Into(ty.clone())
    } else {
        SetterSignature::Exact(ty.clone())
    }
}

pub(crate) fn optional_setter_signature(
    ty: &Type,
    input: SetterInputPolicy,
) -> (SetterSignature, Option<Type>) {
    if let Some(inner) = option_inner_type(ty) {
        (
            SetterSignature::OptionalInner {
                inner: inner.clone(),
                into: input.accepts_into(),
            },
            Some(inner.clone()),
        )
    } else if input.accepts_into() {
        (SetterSignature::Into(ty.clone()), None)
    } else {
        (SetterSignature::Exact(ty.clone()), None)
    }
}

pub(crate) fn defaulted_setter_signature(ty: &Type, input: SetterInputPolicy) -> SetterSignature {
    if input.accepts_into() {
        SetterSignature::Into(ty.clone())
    } else {
        SetterSignature::Exact(ty.clone())
    }
}

pub(crate) fn setter_default_value(default: &SetterDefault) -> SetterDefaultValue {
    match default {
        SetterDefault::Default => SetterDefaultValue::Default,
        SetterDefault::Expr(expr) => SetterDefaultValue::Expr(expr.clone()),
    }
}
