use std::sync::OnceLock;

use es_fluent_manager_embedded as embedded_i18n;
use koruma_shared_lib::Languages;

es_fluent_manager_embedded::define_i18n_module!();

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
