//! Case validation for koruma.
//!
//! This module provides:
//! - `Case` enum representing different string case formats
//! - `CaseValidation` validator to check if a string matches a specific case format
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::string::case::{CaseValidation, Case};
//!
//! #[derive(Koruma)]
//! struct Config {
//!     #[koruma(CaseValidation<_>(case = Case::Snake))]
//!     env_var_name: String,
//!
//!     #[koruma(CaseValidation<_>(case = Case::Kebab))]
//!     css_class: String,
//! }
//! ```

use heck::{
    ToKebabCase, ToLowerCamelCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase, ToTitleCase,
    ToTrainCase, ToUpperCamelCase,
};
use koruma::{Validate, validator};

/// Represents different string case formats.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub enum Case {
    /// snake_case
    Snake,
    /// kebab-case
    Kebab,
    /// camelCase (lower camel case)
    Camel,
    /// PascalCase (upper camel case)
    Pascal,
    /// SCREAMING_SNAKE_CASE
    ShoutySnake,
    /// SCREAMING-KEBAB-CASE
    ShoutyKebab,
    /// Title Case
    Title,
    /// Train-Case
    Train,
}

impl Case {
    /// Converts the given string to this case format.
    #[must_use]
    pub fn convert(&self, s: &str) -> String {
        match self {
            Case::Snake => s.to_snake_case(),
            Case::Kebab => s.to_kebab_case(),
            Case::Camel => s.to_lower_camel_case(),
            Case::Pascal => s.to_upper_camel_case(),
            Case::ShoutySnake => s.to_shouty_snake_case(),
            Case::ShoutyKebab => s.to_shouty_kebab_case(),
            Case::Title => s.to_title_case(),
            Case::Train => s.to_train_case(),
        }
    }

    /// Returns the name of this case format as a human-readable string.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Case::Snake => "snake_case",
            Case::Kebab => "kebab-case",
            Case::Camel => "camelCase",
            Case::Pascal => "PascalCase",
            Case::ShoutySnake => "SCREAMING_SNAKE_CASE",
            Case::ShoutyKebab => "SCREAMING-KEBAB-CASE",
            Case::Title => "Title Case",
            Case::Train => "Train-Case",
        }
    }
}

/// Validates that a string matches a specific case format.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct CaseValidation<T: AsRef<str>> {
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,

    /// The expected case format
    pub case: Case,
}

impl<T: AsRef<str>> Validate<T> for CaseValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        s == self.case.convert(s)
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for CaseValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "value is not in {} format", self.case.name())
    }
}
