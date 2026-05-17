use es_fluent::FluentLocalizer;
use std::collections::HashMap;

use es_fluent_manager_embedded as i18n_manager;
use koruma_shared_lib::Languages;

pub type I18n = i18n_manager::EmbeddedI18n;

pub fn init() -> I18n {
    i18n_manager::EmbeddedI18n::try_new_with_language(Languages::default())
        .expect("Failed to initialize embedded es-fluent manager")
}

pub fn localize<T: es_fluent::FluentMessage + ?Sized>(i18n: &I18n, message: &T) -> String {
    i18n.localize_message(message)
}

pub fn localize_with_args<'a>(
    i18n: &I18n,
    domain: &str,
    id: &str,
    args: Option<&HashMap<&str, es_fluent::FluentValue<'a>>>,
) -> String {
    i18n.localize_in_domain(domain, id, args)
        .unwrap_or_else(|| id.to_string())
}

pub fn change_locale(
    i18n: &I18n,
    language: Languages,
) -> Result<(), i18n_manager::LocalizationError> {
    i18n.select_language(language)?;
    Ok(())
}
