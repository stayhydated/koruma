use crate::site::{
    i18n::SiteLanguage,
    routing::{AppRoute, app_base_href},
};
use dioxus::{document, prelude::*};
use es_fluent_manager_dioxus::I18nProvider;

#[component]
pub fn App() -> Element {
    let stylesheet_href = format!("{}assets/site.css", app_base_href());
    let components_theme_href = format!("{}dx-components-theme.css", app_base_href());

    rsx! {
        document::Stylesheet { href: stylesheet_href }
        document::Stylesheet { href: components_theme_href }
        I18nProvider {
            initial_language: SiteLanguage::default().lang(),
            Router::<AppRoute> {}
        }
    }
}
