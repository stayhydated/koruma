use koruma_derive_core::{
    BuilderMethodCall, FieldInfo, ParsedValidatorUse, StructMode, StructOptions, ValidatorAttr,
    ValidatorLabel, ValidatorSetterArg, ValidatorTargetSelector, contains_infer_type,
    expr_as_simple_ident, option_inner_type, parse_struct_options,
    substitute_infer_type_from_source,
};
use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, Fields, Ident, Member, Path, Type, spanned::Spanned as _};

use super::codegen::{Cardinality, EachCollection, classify_each_collection};
use super::collect_field_infos;
use super::error_bag::ErrorBag;
use super::names::{GeneratedDeriveApi, GeneratedNames, ValidatorNamePlan, plan_validator_names};

mod error;
mod field;
mod operations;
mod setters;
mod storage;
mod target;
mod validation;
mod validator;

pub(crate) use error::*;
pub(crate) use field::*;
pub(crate) use operations::*;
pub(crate) use setters::*;
pub(crate) use storage::*;
pub(crate) use target::*;
pub(crate) use validation::*;
pub(crate) use validator::*;
