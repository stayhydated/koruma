use crate::pages::i18n::DemoLanguage;
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

    let options = DemoLanguage::iter()
        .map(|language| (language, i18n.localize_message(&language)))
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
            span { class: "locale-label", "Language" }
            select::Select::<DemoLanguage> {
                value: Some(selected_value.into()),
                on_value_change,
                select::SelectTrigger {
                    aria_label: "Language",
                    select::SelectValue {
                        placeholder: "Language",
                        class: Some("header-locale-value".to_string()),
                    }
                }
                select::SelectList {
                    for (index, (language, option_label)) in options.iter().enumerate() {
                        {
                            let active = Some(*language) == selected_value();
                            let option_class = if active {
                                "header-locale-option is-active".to_string()
                            } else {
                                "header-locale-option".to_string()
                            };
                            rsx! {
                                select::SelectOption::<DemoLanguage> {
                                    key: "{language:?}",
                                    index,
                                    value: *language,
                                    text_value: Some(option_label.clone()),
                                    class: Some(option_class),
                                    "{option_label}"
                                    if active {
                                        select::SelectItemIndicator {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
