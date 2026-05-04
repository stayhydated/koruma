use es_fluent::FluentLocalizer;
use std::collections::HashMap;
use std::sync::OnceLock;

use es_fluent_manager_embedded as i18n_manager;
use koruma_shared_lib::Languages;

es_fluent_manager_embedded::define_i18n_module!();

static I18N: OnceLock<i18n_manager::EmbeddedI18n> = OnceLock::new();

fn manager() -> &'static i18n_manager::EmbeddedI18n {
    I18N.get_or_init(|| {
        i18n_manager::EmbeddedI18n::try_new()
            .expect("Failed to initialize embedded es-fluent manager")
    })
}

pub fn init() -> &'static i18n_manager::EmbeddedI18n {
    manager()
}

pub fn localize<T: es_fluent::FluentMessage + ?Sized>(message: &T) -> String {
    manager().localize_message(message)
}

pub fn localize_with_args<'a>(
    domain: &str,
    id: &str,
    args: Option<&HashMap<&str, es_fluent::FluentValue<'a>>>,
) -> String {
    manager()
        .localize_in_domain(domain, id, args)
        .unwrap_or_else(|| id.to_string())
}

pub fn change_locale(language: Languages) -> Result<(), i18n_manager::LocalizationError> {
    manager().select_language(language)?;
    Ok(())
}
