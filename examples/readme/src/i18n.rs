use es_fluent::EsFluent;
use std::sync::OnceLock;
use strum::EnumIter;

use es_fluent_lang::es_fluent_language;
use es_fluent_manager_embedded as embedded_i18n;

es_fluent_manager_embedded::define_i18n_module!();

#[es_fluent_language]
#[derive(Clone, Copy, Debug, EnumIter, Eq, EsFluent, PartialEq)]
pub enum Languages {}

impl Languages {
    pub fn next(self) -> Self {
        use strum::IntoEnumIterator as _;
        let all = Self::iter().collect::<Vec<_>>();
        let current_index = all.iter().position(|&l| l == self).unwrap_or(0);
        all[(current_index + 1) % all.len()]
    }
}

static LOCALIZER: OnceLock<embedded_i18n::EmbeddedI18n> = OnceLock::new();

fn localizer() -> &'static embedded_i18n::EmbeddedI18n {
    LOCALIZER.get_or_init(|| {
        embedded_i18n::EmbeddedI18n::try_new()
            .expect("Failed to initialize embedded es-fluent localizer")
    })
}

pub fn init() -> &'static embedded_i18n::EmbeddedI18n {
    localizer()
}

pub fn localize<T: es_fluent::FluentMessage + ?Sized>(message: &T) -> String {
    localizer().localize_message(message)
}

pub fn change_locale(language: Languages) -> Result<(), embedded_i18n::LocalizationError> {
    localizer().select_language(language)?;
    Ok(())
}
