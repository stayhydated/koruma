use crate::site::{
    i18n::{SiteLanguage, dioxus_i18n_asset_modules},
    routing::{AppRoute, app_base_href},
};
use dioxus::{document, prelude::*};
use es_fluent_manager_dioxus::DioxusAssetI18nProvider;

#[component]
pub fn App() -> Element {
    let base_href = app_base_href();
    let stylesheet_href = format!("{base_href}assets/site.css");
    let components_theme_href = format!("{base_href}dx-components-theme.css");

    rsx! {
        stayhydated_dioxus::SharedStyles {}
        document::Stylesheet { href: stylesheet_href }
        document::Stylesheet { href: components_theme_href }
        stayhydated_dioxus::ShaderBackground {}
        DioxusAssetI18nProvider {
            modules: dioxus_i18n_asset_modules(),
            initial_language: SiteLanguage::default().lang(),
            Router::<AppRoute> {}
        }
    }
}
