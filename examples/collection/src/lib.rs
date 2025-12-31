pub mod i18n;

use es_fluent::EsFluent;
use es_fluent_lang::es_fluent_language;
use koruma::Koruma;
use koruma_collection::len::LenValidation;
use strum::EnumIter;

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, EsFluent, PartialEq)]
pub enum Languages {}

impl Languages {
    pub fn next(self) -> Self {
        use strum::IntoEnumIterator as _;
        let all = Self::iter().collect::<Vec<_>>();
        let current_index = all.iter().position(|&l| l == self).unwrap_or(0);
        all[(current_index + 1) % all.len()]
    }
}

/// Example: Validating a shopping order with item count constraints.
#[derive(Koruma)]
pub struct Order {
    /// Items must have between 1 and 10 entries
    #[koruma(LenValidation<_>(min = 1, max = 10))]
    pub items: Vec<String>,
}

/// Example: Validating user profile with string length constraints.
#[derive(Koruma)]
pub struct UserProfile {
    /// Username must be 3-20 characters
    #[koruma(LenValidation<_>(min = 3, max = 20))]
    pub username: String,

    /// Bio must be at most 500 characters (can be empty)
    #[koruma(LenValidation<_>(min = 0, max = 500))]
    pub bio: String,
}

/// Example: Validating a blog post with tag constraints.
#[derive(Koruma)]
pub struct BlogPost {
    /// Title must be 5-100 characters
    #[koruma(LenValidation<_>(min = 5, max = 100))]
    pub title: String,

    /// Must have 1-5 tags
    #[koruma(LenValidation<_>(min = 1, max = 5))]
    pub tags: std::collections::HashSet<String>,
}
