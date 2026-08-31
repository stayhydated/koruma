//! Utility functions for type manipulation and analysis.
//!
//! These utilities are used for working with syn types, particularly
//! for handling `Option<T>`, `Vec<T>`, and type inference placeholders.

use proc_macro2::Span;
use syn::{
    AngleBracketedGenericArguments, Expr, GenericArgument, Ident, ParenthesizedGenericArguments,
    Path, PathArguments, PathSegment, ReturnType, Type, TypeParamBound, spanned::Spanned as _,
};

mod expression;
mod infer;
mod known_types;
mod source_shape;
mod substitution;

pub use expression::expr_as_simple_ident;
pub use infer::{contains_infer_type, first_generic_arg};
pub use known_types::{
    KnownTypeShape, is_option_type, option_inner_type, type_to_ident, vec_inner_type,
};
pub use source_shape::substitute_infer_type_from_source;
pub use substitution::substitute_infer_type;
