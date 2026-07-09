use attribute_dsl::{
    AttributeChain, ChainCompletion, ChainParseOptions, SingleTypeArg,
    split_terminal_single_type_arg,
};
use heck::{ToSnakeCase as _, ToUpperCamelCase as _};
use quote::ToTokens;
use syn::{
    Error, Expr, Ident, Path, PathArguments, Result, Type,
    parse::{Parse, ParseStream, discouraged::Speculative as _},
    spanned::Spanned as _,
};

use super::keywords::ReservedBuilderMethod;

/// Represents a single parsed validator configuration chain.
///
/// Validator configuration uses Koruma dot-method chains and also supports
/// fully-qualified validator paths like
/// `module::path::ValidatorName::<_>`.
///
/// # Examples
///
/// ```rust
/// use koruma_derive_core::ValidatorAttr;
///
/// let simple: ValidatorAttr = syn::parse_quote!(NonEmptyValidation);
/// assert_eq!(simple.name().to_string(), "NonEmptyValidation");
/// assert!(!simple.uses_type_inference());
///
/// let inferred: ValidatorAttr =
///     syn::parse_quote!(RangeValidation::<_>.min(0).max(100));
/// assert!(inferred.uses_type_inference());
/// assert_eq!(inferred.setter_calls().len(), 2);
///
/// let explicit: ValidatorAttr =
///     syn::parse_quote!(RangeValidation::<i32>.min(0).max(100));
/// assert!(explicit.has_explicit_type());
///
/// let full_path: ValidatorAttr =
///     syn::parse_quote!(validators::numeric::RangeValidation::<_>.min(0));
/// assert_eq!(full_path.path_name(), "validators::numeric::RangeValidation");
/// ```
#[derive(Clone, Debug)]
pub enum ValidatorSetterArg {
    /// A normal Rust expression passed directly to a generated validator setter.
    Expr(Expr),
}

impl ValidatorSetterArg {
    pub fn as_expr(&self) -> &Expr {
        match self {
            Self::Expr(expr) => expr,
        }
    }
}

impl ToTokens for ValidatorSetterArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BuilderMethodCall {
    /// The builder setter method name, such as `min` or `exclusive_max`.
    method: Ident,
    /// Positional typed arguments passed to the setter.
    args: Vec<ValidatorSetterArg>,
}

#[derive(Clone, Debug)]
pub enum ValidatorChainCompletion {
    None,
    DotProbe { marker: Ident },
}

impl BuilderMethodCall {
    pub fn method(&self) -> &Ident {
        &self.method
    }

    pub fn args(&self) -> &[ValidatorSetterArg] {
        &self.args
    }
}

/// A parsed validator path that is guaranteed to have a terminal segment.
#[derive(Clone, Debug)]
pub struct ValidatorPath {
    path: Path,
    name: Ident,
}

impl ValidatorPath {
    fn new(path: Path) -> Result<Self> {
        let path_span = path.span();
        let name = path
            .segments
            .last()
            .ok_or_else(|| Error::new(path_span, "expected validator path"))?
            .ident
            .clone();

        Ok(Self { path, name })
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn span(&self) -> proc_macro2::Span {
        self.path.span()
    }
}

/// Type argument syntax used by a validator path.
#[derive(Clone, Debug)]
pub enum ValidatorTypeArg {
    /// No validator type argument was supplied.
    None,
    /// The validator used `::<_>` and should infer from the validation target.
    Infer,
    /// The validator supplied an explicit type argument.
    Explicit(Box<Type>),
}

#[derive(Clone, Debug)]
pub struct ValidatorAttr {
    /// The validator path, which may be a simple identifier or a full path.
    /// Examples: `StringLengthValidation`, `validators::normal::NumberRangeValidation`
    validator: ValidatorPath,
    /// Parsed validator type argument syntax.
    type_arg: ValidatorTypeArg,
    /// Builder setter method calls collected from a direct validator chain.
    /// Used by `Validator.arg(value)...`.
    builder_methods: Vec<BuilderMethodCall>,
    completion: ValidatorChainCompletion,
}

impl ValidatorAttr {
    /// Returns the simple name of the validator (the last segment of the path).
    /// Used for generating field names and enum variants.
    pub fn name(&self) -> &Ident {
        self.validator.name()
    }

    /// Returns the validator path as written, without generic arguments.
    pub fn path(&self) -> &Path {
        self.validator.as_path()
    }

    /// Returns the parsed type argument syntax for this validator path.
    pub fn type_arg(&self) -> &ValidatorTypeArg {
        &self.type_arg
    }

    /// Returns the full validator path as written, without generic arguments.
    pub fn path_name(&self) -> String {
        self.validator
            .as_path()
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    /// Returns a stable snake_case stem for generated field and getter names.
    ///
    /// This returns the fully qualified path flattened into snake_case.
    /// Callers that need to resolve collisions should combine this with
    /// additional disambiguation logic.
    pub fn codegen_snake_name(&self) -> String {
        self.validator
            .as_path()
            .segments
            .iter()
            .map(|segment| segment.ident.to_string().to_snake_case())
            .collect::<Vec<_>>()
            .join("_")
    }

    /// Returns a stable UpperCamelCase stem for generated enum variants.
    ///
    /// This returns the fully qualified path flattened into UpperCamelCase.
    /// Callers that need to resolve collisions should combine this with
    /// additional disambiguation logic.
    pub fn codegen_upper_camel_name(&self) -> String {
        self.validator
            .as_path()
            .segments
            .iter()
            .map(|segment| segment.ident.to_string().to_upper_camel_case())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Returns whether this validator has any arguments.
    pub fn has_args(&self) -> bool {
        !self.builder_methods.is_empty()
    }

    /// Returns true when the parsed chain ends in a completion probe.
    pub fn has_completion_probe(&self) -> bool {
        matches!(self.completion, ValidatorChainCompletion::DotProbe { .. })
    }

    /// Returns the completion probe marker that rust-analyzer maps back to the cursor.
    pub fn completion_marker(&self) -> Option<&Ident> {
        match &self.completion {
            ValidatorChainCompletion::DotProbe { marker } => Some(marker),
            ValidatorChainCompletion::None => None,
        }
    }

    /// Returns validator configuration as normalized builder setter calls.
    pub fn setter_calls(&self) -> &[BuilderMethodCall] {
        &self.builder_methods
    }

    /// Returns whether this validator uses type inference (`<_>` syntax).
    pub fn uses_type_inference(&self) -> bool {
        matches!(self.type_arg, ValidatorTypeArg::Infer)
    }

    /// Returns whether this validator has an explicit type parameter.
    pub fn has_explicit_type(&self) -> bool {
        self.explicit_type().is_some()
    }

    /// Returns the explicit validator type parameter when one was supplied.
    pub fn explicit_type(&self) -> Option<&Type> {
        match &self.type_arg {
            ValidatorTypeArg::Explicit(ty) => Some(ty.as_ref()),
            ValidatorTypeArg::None | ValidatorTypeArg::Infer => None,
        }
    }
}

impl Parse for ValidatorAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork();
        let parse_options = ChainParseOptions::new();
        match AttributeChain::parse_with_options(&fork, &parse_options) {
            Ok(chain) => {
                input.advance_to(&fork);
                ValidatorAttr::try_from_chain(chain)
            },
            Err(_) => Err(invalid_validator_syntax_error(input)),
        }
    }
}

impl ValidatorAttr {
    fn try_from_chain(chain: AttributeChain) -> Result<Self> {
        let (validator, type_arg) = split_validator_path_type_args(chain.root_path().clone())?;
        let validator = ValidatorPath::new(validator)?;
        let mut builder_methods = Vec::new();

        for call in chain.calls() {
            if let Some(method) = ReservedBuilderMethod::from_ident(call.method()) {
                let method_name = call.method().to_string();
                return Err(Error::new(
                    call.method().span(),
                    reserved_builder_method_error(method, &method_name),
                ));
            }

            validate_builder_method_syntax(
                call.method(),
                call.turbofish().map(syn::spanned::Spanned::span),
                call.args().len(),
            )?;

            builder_methods.push(BuilderMethodCall {
                method: call.method().clone(),
                args: call
                    .args()
                    .iter()
                    .cloned()
                    .map(ValidatorSetterArg::Expr)
                    .collect(),
            });
        }

        let completion = match chain.completion() {
            ChainCompletion::None => ValidatorChainCompletion::None,
            ChainCompletion::DotProbe { marker } => ValidatorChainCompletion::DotProbe {
                marker: marker.clone(),
            },
        };

        Ok(ValidatorAttr {
            validator,
            type_arg,
            builder_methods,
            completion,
        })
    }
}

fn split_validator_path_type_args(validator: Path) -> Result<(Path, ValidatorTypeArg)> {
    let (path, type_arg) = split_terminal_single_type_arg(validator, "validator")?;
    let type_arg = match type_arg {
        SingleTypeArg::None => ValidatorTypeArg::None,
        SingleTypeArg::Infer => ValidatorTypeArg::Infer,
        SingleTypeArg::Explicit(ty) => ValidatorTypeArg::Explicit(ty),
    };
    Ok((path, type_arg))
}

fn invalid_validator_syntax_error(input: ParseStream) -> Error {
    if input.is_empty() || looks_like_angle_generic_shorthand(input) {
        return Error::new(
            input.span(),
            "validator syntax requires a dot validator chain such as \
             `RequiredValidation::<_>` or `RangeValidation::<_>.min(value).max(value)`",
        );
    }

    Error::new(input.span(), "expected validator chain")
}

fn looks_like_angle_generic_shorthand(input: ParseStream) -> bool {
    let fork = input.fork();
    let Ok(path) = fork.parse::<Path>() else {
        return false;
    };

    if !fork.is_empty() {
        return false;
    }

    path.segments.last().is_some_and(|segment| {
        matches!(
            &segment.arguments,
            PathArguments::AngleBracketed(args) if args.colon2_token.is_none()
        )
    })
}

fn validate_builder_method_syntax(
    method: &Ident,
    generic_args_span: Option<proc_macro2::Span>,
    arg_count: usize,
) -> Result<()> {
    if let Some(span) = generic_args_span {
        return Err(Error::new(
            span,
            format!(
                "validator setter `{method}(...)` does not accept generic arguments; put type arguments on the validator path, such as `Validator::<_>.{method}(value)`"
            ),
        ));
    }

    if arg_count != 1 {
        return Err(Error::new(
            method.span(),
            format!(
                "validator setter `{method}(...)` expects exactly one argument; use bare validator syntax for validators without configuration fields"
            ),
        ));
    }

    Ok(())
}

fn reserved_builder_method_error(method: ReservedBuilderMethod, method_name: &str) -> String {
    if method.is_builder() {
        return "`.builder(...)` is outside Koruma's validator attribute grammar; use dot-chain syntax such as `Validator::<_>.min(value)` or bare `Validator::<_>` for validators without configuration fields"
            .to_owned();
    }

    if matches!(method, ReservedBuilderMethod::WithValue) {
        return "`.with_value(...)` is only for manual validator builders outside `#[koruma(...)]`; in a Koruma attribute, the value is supplied automatically by the field or collection element"
            .to_owned();
    }

    format!(
        "validator chains should stop before `.{method_name}(...)`; koruma injects builder creation, value capture, and `.build()` automatically"
    )
}
