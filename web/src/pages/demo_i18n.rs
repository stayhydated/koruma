use crate::pages::i18n::DemoLanguage;
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use stayhydated_dioxus::{
    LanguageSelect, StayhydatedSiteLanguage as _, stayhydated_all_language_options,
    stayhydated_selected_language_or_default,
};

#[component]
pub(crate) fn DemoLanguageSwitcher() -> Element {
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { class: "locale-switcher-dropdown", "Failed to initialize i18n: {error}" }
            };
        },
    };

    let selected =
        stayhydated_selected_language_or_default::<DemoLanguage>(i18n.requested_language());
    let options = stayhydated_all_language_options::<DemoLanguage>(|language| {
        i18n.localize_message(&language)
    });
    let i18n_for_select = i18n.clone();
    let on_change = move |next_language: DemoLanguage| {
        let _ = i18n_for_select.select_language(next_language.language_identifier());
    };

    rsx! {
        LanguageSelect::<DemoLanguage> {
            label: "Language",
            selected,
            options,
            on_change,
        }
    }
}
