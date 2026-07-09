use es_fluent::EsFluent;
use std::sync::OnceLock;
use strum::EnumIter;

use es_fluent_lang::es_fluent_language;
use es_fluent_manager_embedded as embedded_i18n;

es_fluent_manager_embedded::define_i18n_module!();

pub type LocalizationError = embedded_i18n::LocalizationError;

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, Eq, EsFluent, PartialEq)]
pub enum Languages {}

impl Languages {
    pub fn next(self) -> Self {
        use strum::IntoEnumIterator as _;

        let mut first = None;
        let mut return_next = false;
        for language in Self::iter() {
            if first.is_none() {
                first = Some(language);
            }

            if return_next {
                return language;
            }

            return_next = language == self;
        }

        first.unwrap_or(self)
    }
}

static LOCALIZER: OnceLock<embedded_i18n::EmbeddedI18n> = OnceLock::new();

fn localizer() -> &'static embedded_i18n::EmbeddedI18n {
    LOCALIZER.get_or_init(|| {
        embedded_i18n::EmbeddedI18n::try_new()
            .expect("example embedded i18n assets should be generated and registered at build time")
    })
}

pub fn init() -> &'static embedded_i18n::EmbeddedI18n {
    localizer()
}

pub fn localize<T: es_fluent::FluentMessage + ?Sized>(message: &T) -> String {
    localizer().localize_message(message)
}

pub fn change_locale(language: Languages) -> Result<(), LocalizationError> {
    localizer().select_language(language)?;
    Ok(())
}
