use heck::{ToSnakeCase as _, ToUpperCamelCase as _};
use proc_macro2::TokenTree;
use quote::ToTokens;
use syn::{
    Error, Expr, GenericArgument, Ident, Path, PathArguments, Result, Type,
    parse::{Parse, ParseStream, discouraged::Speculative as _},
    spanned::Spanned as _,
};

use super::keywords::ReservedBuilderMethod;

const COMPLETION_MARKER: &str = "__koruma_ra_completion_marker";

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
    DotProbe,
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
        matches!(self.completion, ValidatorChainCompletion::DotProbe)
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
        if let Some(attr) = try_parse_direct_validator(input)? {
            return Ok(attr);
        }

        Err(invalid_validator_syntax_error(input))
    }
}

fn try_parse_direct_validator(input: ParseStream) -> Result<Option<ValidatorAttr>> {
    let fork = input.fork();
    let (expr, synthetic_probe, advanced_fork) = match fork.parse::<Expr>() {
        Ok(expr) => {
            if fork.is_empty() || !fork.peek(syn::Token![.]) {
                (expr, false, fork)
            } else {
                match parse_trailing_dot_probe_expr(&fork, expr.to_token_stream())? {
                    Some(expr) => (expr, true, fork),
                    None => return Ok(None),
                }
            }
        },
        Err(_) => {
            let fallback = input.fork();
            match parse_trailing_dot_probe_expr(&fallback, proc_macro2::TokenStream::new())? {
                Some(expr) => (expr, true, fallback),
                None => return Ok(None),
            }
        },
    };
    input.advance_to(&advanced_fork);

    let Some((validator, builder_methods, completion)) =
        analyze_direct_validator_expr(&expr, synthetic_probe)?
    else {
        return Err(Error::new(expr.span(), "expected validator chain"));
    };

    let (validator, type_arg) = split_validator_path_type_args(validator)?;
    let validator = ValidatorPath::new(validator)?;

    Ok(Some(ValidatorAttr {
        validator,
        type_arg,
        builder_methods,
        completion,
    }))
}

fn parse_trailing_dot_probe_expr(
    input: ParseStream,
    probe_expr: proc_macro2::TokenStream,
) -> Result<Option<Expr>> {
    let mut probe_expr = probe_expr;
    let mut saw_token = false;
    let mut ends_with_dot = false;

    while !input.is_empty() && !input.peek(syn::Token![,]) {
        let token: TokenTree = input.parse()?;
        ends_with_dot = matches!(&token, TokenTree::Punct(punct) if punct.as_char() == '.');
        probe_expr.extend(token.to_token_stream());
        saw_token = true;
    }

    if !saw_token || !ends_with_dot {
        return Ok(None);
    }

    let marker = Ident::new(COMPLETION_MARKER, proc_macro2::Span::call_site());
    probe_expr.extend(marker.to_token_stream());

    Ok(syn::parse2::<Expr>(probe_expr).ok())
}

fn invalid_validator_syntax_error(input: ParseStream) -> Error {
    Error::new(
        input.span(),
        "validator syntax requires a dot validator chain such as \
         `RequiredValidation::<_>` or `RangeValidation::<_>.min(value).max(value)`",
    )
}

fn split_validator_path_type_args(mut validator: Path) -> Result<(Path, ValidatorTypeArg)> {
    let validator_span = validator.span();
    let last_segment = validator
        .segments
        .last_mut()
        .ok_or_else(|| Error::new(validator_span, "expected validator path"))?;

    let args = std::mem::replace(&mut last_segment.arguments, PathArguments::None);
    let type_arg = match args {
        PathArguments::None => ValidatorTypeArg::None,
        PathArguments::AngleBracketed(mut angle_args) => {
            if angle_args.args.len() != 1 {
                return Err(Error::new(
                    angle_args.span(),
                    "validator type syntax expects exactly one type argument",
                ));
            }

            let arg = angle_args.args.pop().expect("len checked").into_value();
            match arg {
                GenericArgument::Type(Type::Infer(_)) => ValidatorTypeArg::Infer,
                GenericArgument::Type(ty) => ValidatorTypeArg::Explicit(Box::new(ty)),
                _ => Err(Error::new(
                    arg.span(),
                    "validator type syntax expects a type argument",
                ))?,
            }
        },
        PathArguments::Parenthesized(args) => {
            return Err(Error::new(
                args.span(),
                "validator path does not support parenthesized arguments",
            ));
        },
    };

    Ok((validator, type_arg))
}

fn analyze_direct_validator_expr(
    expr: &Expr,
    allow_completion_probe: bool,
) -> Result<Option<(Path, Vec<BuilderMethodCall>, ValidatorChainCompletion)>> {
    match expr {
        Expr::Group(group) => analyze_direct_validator_expr(&group.expr, allow_completion_probe),
        Expr::Paren(paren) => analyze_direct_validator_expr(&paren.expr, allow_completion_probe),
        Expr::Field(field) => {
            let Some((validator, builder_methods, _completion)) =
                analyze_direct_validator_expr(&field.base, false)?
            else {
                return Ok(None);
            };

            let syn::Member::Named(marker) = &field.member else {
                return Ok(None);
            };

            if !allow_completion_probe || marker != COMPLETION_MARKER {
                return Ok(None);
            }

            Ok(Some((
                validator,
                builder_methods,
                ValidatorChainCompletion::DotProbe,
            )))
        },
        Expr::MethodCall(method_call) => {
            let Some((validator, mut builder_methods, _completion)) =
                analyze_direct_validator_expr(&method_call.receiver, false)?
            else {
                return Ok(None);
            };

            if let Some(method) = ReservedBuilderMethod::from_ident(&method_call.method) {
                let method_name = method_call.method.to_string();
                return Err(Error::new(
                    method_call.method.span(),
                    reserved_builder_method_error(method, &method_name),
                ));
            }

            validate_builder_method_syntax(
                &method_call.method,
                method_call
                    .turbofish
                    .as_ref()
                    .map(syn::spanned::Spanned::span),
                method_call.args.len(),
            )?;

            builder_methods.push(BuilderMethodCall {
                method: method_call.method.clone(),
                args: method_call
                    .args
                    .iter()
                    .cloned()
                    .map(ValidatorSetterArg::Expr)
                    .collect(),
            });
            Ok(Some((
                validator,
                builder_methods,
                ValidatorChainCompletion::None,
            )))
        },
        Expr::Call(_) => Ok(None),
        Expr::Path(path) => Ok(Some((
            path.path.clone(),
            Vec::new(),
            ValidatorChainCompletion::None,
        ))),
        _ => Ok(None),
    }
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
        return "`::builder()` is outside Koruma's validator attribute grammar; use dot-chain syntax such as `Validator::<_>.min(value)` or bare `Validator::<_>` for validators without configuration fields"
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
