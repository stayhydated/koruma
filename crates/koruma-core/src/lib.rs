#![doc = include_str!("../README.md")]

/// Trait for types that can validate a value of type `T`.
///
/// Implementors should return `true` if validation passes,
/// or `false` if validation fails. The error details are
/// captured in the validation struct itself.
pub trait Validate<T> {
    fn validate(&self, value: &T) -> bool;
}

/// Trait for validation error structs that can report whether they contain errors.
///
/// This is auto-implemented by the derive macro for generated
/// error structs, allowing easy checking if any validation failed.
pub trait ValidationError {
    /// Returns `true` if there are no validation errors.
    fn is_empty(&self) -> bool;

    /// Returns `true` if there are any validation errors.
    fn has_errors(&self) -> bool {
        !self.is_empty()
    }
}

/// Static metadata describing one configurable parameter on a validator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatorParamDescriptor {
    name: &'static str,
    type_name: &'static str,
    required: bool,
}

impl ValidatorParamDescriptor {
    /// Describe a validator parameter.
    pub const fn new(name: &'static str, type_name: &'static str, required: bool) -> Self {
        Self {
            name,
            type_name,
            required,
        }
    }

    /// Parameter name as it appears in the validator struct.
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Rust type name recorded for the parameter.
    pub const fn type_name(self) -> &'static str {
        self.type_name
    }

    /// Whether this parameter must be supplied before building the validator.
    pub const fn required(self) -> bool {
        self.required
    }
}

/// Static metadata describing a validator type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatorDescriptor {
    type_name: &'static str,
    params: &'static [ValidatorParamDescriptor],
}

impl ValidatorDescriptor {
    /// Describe a validator type and its configurable parameters.
    pub const fn new(type_name: &'static str, params: &'static [ValidatorParamDescriptor]) -> Self {
        Self { type_name, params }
    }

    /// Fully qualified Rust type name for the validator.
    pub const fn type_name(self) -> &'static str {
        self.type_name
    }

    /// Parameter descriptors for this validator.
    pub const fn params(self) -> &'static [ValidatorParamDescriptor] {
        self.params
    }
}

/// Runtime value captured for one validator parameter.
#[derive(Clone, Debug, PartialEq)]
pub enum ValidatorParamValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    None,
    Opaque { type_name: &'static str },
}

impl ValidatorParamValue {
    /// Create an opaque value when the parameter cannot be represented without
    /// adding trait bounds to the validator type.
    pub fn opaque<T: ?Sized>(_: &T) -> Self {
        Self::Opaque {
            type_name: ::core::any::type_name::<T>(),
        }
    }
}

/// Runtime metadata for one validator parameter.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatorParam {
    name: &'static str,
    value: ValidatorParamValue,
}

impl ValidatorParam {
    /// Pair a validator parameter name with its runtime value.
    pub const fn new(name: &'static str, value: ValidatorParamValue) -> Self {
        Self { name, value }
    }

    /// Parameter name.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Runtime parameter value.
    pub const fn value(&self) -> &ValidatorParamValue {
        &self.value
    }
}

/// Optional metadata companion for validators.
///
/// This trait intentionally sits beside [`Validate`] instead of changing it.
/// Implementing it does not prove that the validator can validate `T`; use
/// [`Validate<T>`] for that runtime contract.
pub trait ValidatorMetadata<T> {
    /// Static validator descriptor.
    fn validator_descriptor() -> ValidatorDescriptor;

    /// Runtime parameter values captured by this validator instance.
    fn validator_params(&self) -> Vec<ValidatorParam>;
}

/// Scope of a structured validation issue.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationIssueScope {
    Form,
    Field,
    Element,
}

/// Structured validation issue emitted by generated Koruma error types.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationIssue {
    field: Option<&'static str>,
    scope: ValidationIssueScope,
    validator: Option<&'static str>,
    label: Option<&'static str>,
    element_index: Option<usize>,
    message: String,
    params: Vec<ValidatorParam>,
}

impl ValidationIssue {
    /// Create a form-scoped issue.
    pub fn form(message: impl Into<String>) -> Self {
        Self {
            field: None,
            scope: ValidationIssueScope::Form,
            validator: None,
            label: None,
            element_index: None,
            message: message.into(),
            params: Vec::new(),
        }
    }

    /// Create a field-scoped issue.
    pub fn field(
        field: &'static str,
        validator: &'static str,
        label: Option<&'static str>,
        message: impl Into<String>,
        params: Vec<ValidatorParam>,
    ) -> Self {
        Self {
            field: Some(field),
            scope: ValidationIssueScope::Field,
            validator: Some(validator),
            label,
            element_index: None,
            message: message.into(),
            params,
        }
    }

    /// Create an element-scoped issue.
    pub fn element(
        field: &'static str,
        element_index: usize,
        validator: &'static str,
        label: Option<&'static str>,
        message: impl Into<String>,
        params: Vec<ValidatorParam>,
    ) -> Self {
        Self {
            field: Some(field),
            scope: ValidationIssueScope::Element,
            validator: Some(validator),
            label,
            element_index: Some(element_index),
            message: message.into(),
            params,
        }
    }

    /// Field name, if this issue is field- or element-scoped.
    pub const fn field_name(&self) -> Option<&'static str> {
        self.field
    }

    /// Issue scope.
    pub const fn scope(&self) -> ValidationIssueScope {
        self.scope
    }

    /// Validator type name, if known.
    pub const fn validator(&self) -> Option<&'static str> {
        self.validator
    }

    /// Validator label from the source attribute, if present.
    pub const fn label(&self) -> Option<&'static str> {
        self.label
    }

    /// Collection element index for element-scoped issues.
    pub const fn element_index(&self) -> Option<usize> {
        self.element_index
    }

    /// Human-readable validation message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Runtime validator parameters.
    pub fn params(&self) -> &[ValidatorParam] {
        &self.params
    }
}

/// Optional structured issue enumeration for generated validation errors.
pub trait ValidationIssues {
    /// Return all validation issues represented by this error value.
    fn issues(&self) -> Vec<ValidationIssue>;
}

#[doc(hidden)]
pub mod __private {
    /// Hidden marker implemented by `#[derive(Koruma)]` for the source type.
    ///
    /// Companion derives use this through marker supertraits so deriving only
    /// `KorumaAllDisplay` or `KorumaAllFluent` produces a direct trait error
    /// instead of only missing generated-type errors.
    pub trait KorumaWasDerived {}

    /// Hidden marker implemented by `#[derive(KorumaAllDisplay)]`.
    pub trait KorumaAllDisplayRequiresKoruma: KorumaWasDerived {}

    /// Hidden marker implemented by `#[derive(KorumaAllFluent)]`.
    pub trait KorumaAllFluentRequiresKoruma: KorumaWasDerived {}

    /// Hidden helper trait used in generated capture impls for validators that
    /// store a clone of the validated input.
    pub trait CapturedInputCanBeCloned: Clone {}

    impl<T: Clone> CapturedInputCanBeCloned for T {}

    /// Hidden trait used by generated validation code to turn a ready validator
    /// builder into its validator instance.
    pub trait BuildValidator {
        type Validator;

        fn build_validator(self) -> Self::Validator;
    }

    /// Hidden trait used by derived validation code to apply borrowed values to
    /// validator builders according to their capture policy.
    ///
    /// Validators that capture the input clone from the borrowed value inside the
    /// generated impl. Validators marked with `#[koruma(skip_capture)]` on
    /// an `Option<T>` value field can ignore the borrowed input and keep their
    /// default value instead.
    pub trait CaptureValueRef<T> {
        type Output: BuildValidator;

        fn capture_value_ref(self, value: &T) -> Self::Output;
    }

    /// Hidden trait used by generated `each(...)` checks to prove that a
    /// syntactically recognized collection is also the collection Koruma expects
    /// after Rust name resolution.
    pub trait EachCollectionRef {
        type Element;
    }

    impl<T> EachCollectionRef for ::std::vec::Vec<T> {
        type Element = T;
    }

    impl<T> EachCollectionRef for [T] {
        type Element = T;
    }

    impl<T, const N: usize> EachCollectionRef for [T; N] {
        type Element = T;
    }

    impl<C> EachCollectionRef for &C
    where
        C: EachCollectionRef + ?Sized,
    {
        type Element = C::Element;
    }

    impl<C> EachCollectionRef for &mut C
    where
        C: EachCollectionRef + ?Sized,
    {
        type Element = C::Element;
    }

    /// Hidden trait used by generated `each(...)` checks for optional
    /// collections.
    pub trait OptionalEachCollectionRef {
        type Element;
    }

    impl<C> OptionalEachCollectionRef for ::std::option::Option<C>
    where
        C: EachCollectionRef,
    {
        type Element = C::Element;
    }

    /// Hidden type assertion used by generated `each(...)` collection checks.
    pub fn assert_each_collection_ref<Collection, Element>()
    where
        Collection: EachCollectionRef<Element = Element> + ?Sized,
    {
    }

    /// Hidden type assertion used by generated `each(...)` optional collection checks.
    pub fn assert_optional_each_collection_ref<Collection, Element>()
    where
        Collection: OptionalEachCollectionRef<Element = Element>,
    {
    }

    /// Hidden type assertion used by generated validation checks.
    pub fn assert_validator_ready<Builder, Target, Validator>(_: &Builder)
    where
        Builder: CaptureValueRef<Target>,
        <Builder as CaptureValueRef<Target>>::Output: BuildValidator<Validator = Validator>,
        Validator: super::Validate<Target>,
    {
    }

    /// Hidden type assertion used by generated field-validator checks.
    pub fn assert_field_validator_ready<Builder, Target, Validator>(builder: &Builder)
    where
        Builder: CaptureValueRef<Target>,
        <Builder as CaptureValueRef<Target>>::Output: BuildValidator<Validator = Validator>,
        Validator: super::Validate<Target>,
    {
        assert_validator_ready::<Builder, Target, Validator>(builder);
    }

    /// Hidden type assertion used by generated `each(...)` element-validator checks.
    pub fn assert_element_validator_ready<Builder, Target, Validator>(builder: &Builder)
    where
        Builder: CaptureValueRef<Target>,
        <Builder as CaptureValueRef<Target>>::Output: BuildValidator<Validator = Validator>,
        Validator: super::Validate<Target>,
    {
        assert_validator_ready::<Builder, Target, Validator>(builder);
    }

    /// Hidden type assertion used by generated nested field checks.
    pub fn assert_validate_ext<T>()
    where
        T: super::ValidateExt,
    {
    }

    /// Hidden type assertion used by generated nested field checks.
    pub fn assert_nested_validation_ready<T>()
    where
        T: super::ValidateExt,
        <T as super::ValidateExt>::Error: ::std::default::Default,
    {
        assert_validate_ext::<T>();
    }

    /// Hidden type assertion used by generated newtype field checks.
    pub fn assert_newtype_validation<T>()
    where
        T: super::NewtypeValidation,
    {
    }

    /// Hidden type assertion used by generated and downstream newtype-value checks.
    pub fn assert_newtype_value<T>()
    where
        T: super::NewtypeValue,
    {
    }

    /// Hidden type assertion used by generated and downstream checked newtype reconstruction.
    pub fn assert_newtype_try_from_inner<T>()
    where
        T: super::NewtypeTryFromInner,
    {
    }

    /// Hidden type assertion used by generated newtype field checks.
    pub fn assert_newtype_field_ready<T>()
    where
        T: super::NewtypeValidation,
        <T as super::ValidateExt>::Error: ::std::default::Default,
    {
        assert_newtype_validation::<T>();
    }

    /// Hidden type assertion used by generated Display companion derives.
    pub fn assert_display<T>()
    where
        T: ::std::fmt::Display,
    {
    }

    /// Hidden type assertion used by generated field-validator Display impls.
    pub fn assert_field_display<T>()
    where
        T: ::std::fmt::Display,
    {
        assert_display::<T>();
    }

    /// Hidden type assertion used by generated element-validator Display impls.
    pub fn assert_element_display<T>()
    where
        T: ::std::fmt::Display,
    {
        assert_display::<T>();
    }

    /// Hidden type assertion used by generated newtype error Display impls.
    pub fn assert_newtype_error_display<T>()
    where
        T: ::std::fmt::Display,
    {
        assert_display::<T>();
    }

    /// Hidden type assertion used by generated Fluent companion derives.
    #[cfg(feature = "fluent")]
    pub fn assert_fluent_message<T>()
    where
        T: ::es_fluent::FluentMessage,
    {
    }

    /// Hidden type assertion used by generated field-validator Fluent impls.
    #[cfg(feature = "fluent")]
    pub fn assert_field_fluent_message<T>()
    where
        T: ::es_fluent::FluentMessage,
    {
        assert_fluent_message::<T>();
    }

    /// Hidden type assertion used by generated element-validator Fluent impls.
    #[cfg(feature = "fluent")]
    pub fn assert_element_fluent_message<T>()
    where
        T: ::es_fluent::FluentMessage,
    {
        assert_fluent_message::<T>();
    }

    /// Hidden type assertion used by generated newtype error Fluent impls.
    #[cfg(feature = "fluent")]
    pub fn assert_newtype_error_fluent_message<T>()
    where
        T: ::es_fluent::FluentMessage,
    {
        assert_fluent_message::<T>();
    }

    /// Hidden type assertion used by generated aggregate error Fluent impls.
    #[cfg(feature = "fluent")]
    pub fn assert_error_fluent_message<T>()
    where
        T: ::es_fluent::FluentMessage,
    {
        assert_fluent_message::<T>();
    }
}

/// Trait for structs that derive `Koruma` and have a `validate()` method.
///
/// This trait provides an associated type for the validation error struct,
/// which is used by nested validation to properly type the error fields.
///
/// This is auto-implemented by the `#[derive(Koruma)]` macro.
pub trait ValidateExt {
    /// The validation error type for this struct.
    ///
    /// `Default` is required because generated nested and newtype validation
    /// code needs to construct an empty error value before merging field
    /// failures.
    type Error: ValidationError + Default;

    /// Validates the struct and returns the error struct if validation fails.
    fn validate(&self) -> Result<(), Self::Error>;
}

/// Marker trait for newtype structs (single-field wrappers) that derive `Koruma`.
///
/// This trait is auto-implemented by `#[derive(Koruma)]` when `#[koruma(newtype)]`
/// is used at the struct level. It signals that this type is a newtype wrapper
/// and its error type supports transparent `Deref` access.
///
/// When using a newtype as a field in another struct, use `#[koruma(newtype)]`
/// on the field (instead of `#[koruma(nested)]`) to get transparent error access.
pub trait NewtypeValidation: ValidateExt {}

/// Public inner-value contract for single-field structs that derive `Koruma`
/// with `#[koruma(newtype)]`.
///
/// This trait is implemented by the derive macro in the defining module, so it
/// works for private tuple and named fields without requiring downstream crates
/// to rely on public fields or `Deref` implementations.
pub trait NewtypeValue: NewtypeValidation {
    /// The wrapped value type.
    type Inner;

    /// Borrow the wrapped value.
    fn as_inner(&self) -> &Self::Inner;

    /// Consume the wrapper and return the wrapped value.
    fn into_inner(self) -> Self::Inner
    where
        Self: Sized;

    /// Validate a candidate wrapped value without constructing or cloning the
    /// wrapper.
    fn validate_inner(value: &Self::Inner) -> Result<(), Self::Error>;
}

/// Checked reconstruction contract for Koruma newtypes.
///
/// The derive macro implements this for every struct-level
/// `#[koruma(newtype)]` wrapper. Standard-library `TryFrom<Inner>` remains
/// controlled by the existing `#[koruma(try_from)]` option.
pub trait NewtypeTryFromInner: NewtypeValue {
    /// Validate `value` and construct the wrapper when validation succeeds.
    fn try_from_inner(value: Self::Inner) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// Showcase module for validator discovery and registration.
///
/// When the `internal-showcase` feature is enabled, validators decorated with
/// `#[showcase(...)]` attributes are automatically registered for
/// programmatic discovery by showcase consumers (for example, UIs, examples, or tooling).
#[cfg(feature = "internal-showcase")]
pub mod showcase {
    /// Localizer callback used by showcased validators to render Fluent messages.
    #[cfg(feature = "fluent")]
    pub type FluentLocalizer<'localizer> = ::es_fluent::FluentMessageLookup<'localizer>;

    /// Trait for validators that can be presented by showcase consumers.
    ///
    /// This trait provides a type-erased interface for validators,
    /// allowing consumers to work with any validator regardless of its
    /// generic type parameters.
    ///
    pub trait DynValidator: Send + Sync {
        /// Check if the validation passed.
        fn is_valid(&self) -> bool;

        /// Get the display string via `Display::to_string()`.
        ///
        /// Showcased validators should implement `Display` when they want a
        /// user-facing message in showcase UIs.
        fn display_string(&self) -> String;

        /// Get the fluent i18n string via `FluentMessage::to_fluent_string_with(...)`
        /// `fluent` feature is enabled for the generated showcase impl.
        ///
        /// Returns the message identifier when no localizer callback is provided.
        #[cfg(feature = "fluent")]
        fn fluent_string_with(&self, localize: &mut FluentLocalizer<'_>) -> String;

        #[cfg(feature = "fluent")]
        fn fluent_string(&self) -> String {
            self.fluent_string_with(&mut |_, id, _| id.as_str().to_string())
        }

        #[cfg(not(feature = "fluent"))]
        fn fluent_string(&self) -> String;
    }

    /// The type of input expected by the validator.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum InputType {
        /// Any text input.
        Text,
        /// Numeric input only (integers)
        Numeric,
    }

    koruma_derive::showcase_module_enum!(string, format, numeric, collection, general);

    /// Information about a validator for showcase purposes.
    ///
    /// This struct is registered via `inventory` when a validator uses
    /// `#[showcase(...)]` attributes.
    pub struct ValidatorShowcase {
        /// Human-readable name of the validator
        pub name: &'static str,
        /// Description of what the validator checks
        pub description: &'static str,
        /// The type of input expected by the validator
        pub input_type: InputType,
        /// The module/category this validator belongs to.
        pub module: ValidatorModule,
        /// Factory function that creates a validator from string input.
        /// Returns Ok(validator) on success, or Err(error) if input cannot be parsed.
        pub create_validator: fn(&str) -> ::anyhow::Result<Box<dyn DynValidator>>,
    }

    inventory::collect!(ValidatorShowcase);

    /// Get all registered showcase validators.
    pub fn validators() -> Vec<&'static ValidatorShowcase> {
        inventory::iter::<ValidatorShowcase>().collect()
    }
}
