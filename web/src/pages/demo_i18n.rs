use crate::pages::i18n::{DemoLanguage, DioxusShowcaseMessage};
use dioxus::prelude::*;
use es_fluent_manager_dioxus::use_i18n;
use stayhydated_dioxus::select;
use strum::IntoEnumIterator as _;

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

    let language_label = i18n.localize_message(&DioxusShowcaseMessage::LanguageLabel);
    let search_placeholder =
        i18n.localize_message(&DioxusShowcaseMessage::SearchLanguagesPlaceholder);
    let empty_message = i18n.localize_message(&DioxusShowcaseMessage::NoMatchingLanguages);
    let requested_language = i18n.requested_language();
    let selected = DemoLanguage::try_from(&requested_language).unwrap_or_default();
    let mut selected_value = use_signal(move || Some(selected));
    let selected_for_effect = selected;
    use_effect(move || {
        let next_selected = Some(selected_for_effect);
        if selected_value() != next_selected {
            selected_value.set(next_selected);
        }
    });

    let items = DemoLanguage::iter()
        .map(|language| select::SelectItem::new(language, i18n.localize_message(&language)))
        .collect::<Vec<_>>();
    let on_value_change = move |next_language: Option<DemoLanguage>| {
        let Some(next_language) = next_language else {
            return;
        };

        if Some(next_language) == selected_value() {
            return;
        }

        selected_value.set(Some(next_language));
        let _ = i18n.select_language(next_language);
    };

    rsx! {
        div { class: "locale-switcher-dropdown",
            span { class: "locale-label", "{language_label}" }
            select::SearchableSelect::<DemoLanguage> {
                items,
                aria_label: language_label,
                placeholder: search_placeholder,
                empty_message,
                value: Some(selected_value.into()),
                on_value_change,
            }
        }
    }
}
