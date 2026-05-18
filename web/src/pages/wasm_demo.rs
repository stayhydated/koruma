use std::collections::HashMap;

use dioxus::events::FormData;
use dioxus::prelude::*;
use dioxus_primitives::label::Label;
use dioxus_primitives::tabs::{TabContent, TabList, TabTrigger, Tabs};
use es_fluent::{FluentLocalizer, FluentValue};
use es_fluent_manager_dioxus::{DioxusI18n, use_i18n};
use koruma::showcase::{ValidatorShowcase, validators};
use koruma_collection::__link_showcase_validators;

use crate::components::{ContributePanel, FooterPanel, PageHeader};
use crate::site::i18n::DioxusShowcaseMessage;
use crate::site::routing::PageKind;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValidatorModule {
    String,
    Format,
    Numeric,
    Collection,
    General,
}

impl ValidatorModule {
    const ALL: [Self; 5] = [
        Self::String,
        Self::Format,
        Self::Numeric,
        Self::Collection,
        Self::General,
    ];

    fn available_modules(all_validators: &[&'static ValidatorShowcase]) -> Vec<Self> {
        Self::ALL
            .iter()
            .filter(|&&m| all_validators.iter().any(|&v| m.contains_validator(v)))
            .copied()
            .collect()
    }

    fn name(self, i18n: &DioxusI18n) -> String {
        match self {
            Self::String => i18n.localize_message(&DioxusShowcaseMessage::ModuleString),
            Self::Format => i18n.localize_message(&DioxusShowcaseMessage::ModuleFormat),
            Self::Numeric => i18n.localize_message(&DioxusShowcaseMessage::ModuleNumeric),
            Self::Collection => i18n.localize_message(&DioxusShowcaseMessage::ModuleCollection),
            Self::General => i18n.localize_message(&DioxusShowcaseMessage::ModuleGeneral),
        }
    }

    fn tab_value(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Format => "format",
            Self::Numeric => "numeric",
            Self::Collection => "collection",
            Self::General => "general",
        }
    }

    fn contains_validator(self, showcase: &ValidatorShowcase) -> bool {
        match self {
            Self::String => showcase.module == "string",
            Self::Format => showcase.module == "format",
            Self::Numeric => showcase.module == "numeric",
            Self::Collection => showcase.module == "collection",
            Self::General => showcase.module == "general",
        }
    }
}

#[derive(Clone, Copy)]
enum ValidatorState {
    Valid,
    Invalid,
    Error,
}

impl ValidatorState {
    fn status_emoji(self) -> &'static str {
        match self {
            Self::Valid => "✓",
            Self::Invalid => "✗",
            Self::Error => "!",
        }
    }

    fn status_class(self) -> &'static str {
        match self {
            Self::Valid => "validator-card is-valid",
            Self::Invalid => "validator-card is-invalid",
            Self::Error => "validator-card is-error",
        }
    }

    fn message_heading(self, i18n: &DioxusI18n) -> String {
        match self {
            Self::Valid | Self::Invalid => {
                i18n.localize_message(&DioxusShowcaseMessage::MessageHeadingResult)
            },
            Self::Error => i18n.localize_message(&DioxusShowcaseMessage::MessageHeadingError),
        }
    }
}

#[component]
pub(crate) fn CollectionDioxusPage() -> Element {
    match use_i18n() {
        Ok(_) => rsx! { CollectionDioxusShowcase {} },
        Err(error) => {
            rsx! {
                div { class: "page-shell",
                    PageHeader { current_page: PageKind::CollectionDioxus }
                    main { class: "stack",
                        section { class: "page-title-band",
                            span { class: "panel-label", "i18n load failure" }
                            h1 { "koruma-collection Dioxus demo" }
                            p { "Failed to initialize i18n: {error}" }
                        }
                    }
                    FooterPanel {}
                }
            }
        },
    }
}

#[component]
fn CollectionDioxusShowcase() -> Element {
    let i18n = match use_i18n() {
        Ok(i18n) => i18n,
        Err(error) => {
            return rsx! {
                div { class: "page-shell",
                    PageHeader { current_page: PageKind::CollectionDioxus }
                    main { class: "stack",
                        section { class: "page-title-band",
                            span { class: "panel-label", "i18n load failure" }
                            h1 { "koruma-collection Dioxus demo" }
                            p { "Failed to initialize i18n: {error}" }
                        }
                    }
                    FooterPanel {}
                }
            };
        },
    };
    let (panel_label, page_title, page_intro_body, validation_placeholder, error_prefix) = (
        i18n.localize_message(&DioxusShowcaseMessage::ShowcasePanelLabel),
        i18n.localize_message(&DioxusShowcaseMessage::ShowcaseIntroTitle),
        i18n.localize_message(&DioxusShowcaseMessage::ShowcaseIntroBody),
        i18n.localize_message(&DioxusShowcaseMessage::ValidationPlaceholder),
        i18n.localize_message(&DioxusShowcaseMessage::ErrorPrefix),
    );

    let mut inputs = use_signal(HashMap::<&'static str, String>::new);

    __link_showcase_validators();
    let all_validators = validators();
    let available_modules = ValidatorModule::available_modules(&all_validators);
    let default_module = available_modules
        .first()
        .map(|module| module.tab_value())
        .unwrap_or("string");

    rsx! {
        div { class: "page-shell",
            PageHeader { current_page: PageKind::CollectionDioxus }
            main { class: "stack",
                section { class: "page-title-band",
                    span { class: "panel-label", "{panel_label}" }
                    h1 { "{page_title}" }
                    p { "{page_intro_body}" }
                }
                section { class: "section-band",
                    Tabs {
                        class: "collection-module-tabs",
                        default_value: default_module.to_string(),
                        horizontal: true,
                        TabList {
                            class: "collection-module-tab-list",
                            for (index, module) in available_modules.iter().enumerate() {
                                TabTrigger {
                                    value: module.tab_value().to_string(),
                                    index: index,
                                    class: Some("collection-module-tab".to_string()),
                                    "{module.name(&i18n)}"
                                }
                            }
                        }
                        for (index, module) in available_modules.iter().enumerate() {
                            TabContent {
                                class: Some("collection-module-content".to_string()),
                                index: index,
                                value: module.tab_value().to_string(),
                                div {
                                    class: "collection-module",
                                    h2 { "{module.name(&i18n)}" }
                                    for validator in all_validators
                                        .iter()
                                        .filter(|&validator| module.contains_validator(validator))
                                    {
                                        {
                                            let name = validator.name;
                                            let input = inputs.read().get(name).cloned().unwrap_or_default();
                                            let current_validator = (validator.create_validator)(&input);
                                            let current_state = match &current_validator {
                                                Ok(v) if v.is_valid() => ValidatorState::Valid,
                                                Ok(_) => ValidatorState::Invalid,
                                                Err(_) => ValidatorState::Error,
                                            };
                                            let mut localize =
                                                |domain: &str, id: &str, args: Option<&HashMap<&str, FluentValue<'_>>>| {
                                                    localize_with_args(&i18n, domain, id, args)
                                                };
                                            let message_heading = current_state.message_heading(&i18n);
                                            let (display_msg, fluent_msg, error_msg) =
                                                match current_validator {
                                                    Ok(v) if v.is_valid() => (
                                                        v.display_string(),
                                                        v.fluent_string_with(&mut localize),
                                                        None,
                                                    ),
                                                    Ok(v) => (
                                                        v.display_string(),
                                                        v.fluent_string_with(&mut localize),
                                                        None,
                                                    ),
                                                    Err(error) => (
                                                        String::new(),
                                                        String::new(),
                                                        Some(format!("{error_prefix} {error}")),
                                                    ),
                                                };
                                            let module_key = module.tab_value();
                                            let input_id = format!(
                                                "validator-{module_key}-{}",
                                                name.to_lowercase().replace(' ', "-")
                                            );
                                            validator_row(
                                                input_id,
                                                *validator,
                                                input,
                                                EventHandler::new(
                                                    {
                                                        let name = name;
                                                        move |event: Event<FormData>| {
                                                            inputs.write().insert(name, event.value());
                                                        }
                                                    },
                                                ),
                                                current_state,
                                                display_msg,
                                                fluent_msg,
                                                error_msg,
                                                validation_placeholder.clone(),
                                                message_heading,
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                ContributePanel {}
            }
            FooterPanel {}
        }
    }
}

fn validator_row(
    input_id: String,
    validator: &'static ValidatorShowcase,
    input: String,
    on_input: EventHandler<Event<FormData>>,
    state: ValidatorState,
    display_msg: String,
    fluent_msg: String,
    error_msg: Option<String>,
    input_placeholder: String,
    message_heading: String,
) -> Element {
    rsx! {
        div { class: state.status_class(),
            div { class: "validator-row-head",
                Label {
                    html_for: input_id.clone(),
                    class: "validator-row-label".to_string(),
                    "{validator.name}"
                }
                span { class: "validator-status", "{state.status_emoji()}" }
            }
            p { class: "validator-row-description", "{validator.description}" }
            input {
                id: input_id,
                class: "validator-input",
                r#type: "text",
                value: input,
                placeholder: input_placeholder,
                oninput: on_input,
            }
            p { class: "validator-message-heading", "{message_heading}:" }
            {
                match error_msg {
                    Some(error) => rsx! {
                        p { class: "validator-message validator-message-error", "{error}" }
                    },
                    None => rsx! {
                        p { class: "validator-message", "{display_msg}" }
                        if !fluent_msg.is_empty() {
                            p { class: "validator-message validator-message-subtle", "{fluent_msg}" }
                        }
                    },
                }
            }
        }
    }
}

fn localize_with_args<'a>(
    i18n: &DioxusI18n,
    domain: &str,
    id: &str,
    args: Option<&HashMap<&str, FluentValue<'a>>>,
) -> String {
    i18n.localize_in_domain(domain, id, args)
        .unwrap_or(id.to_string())
}
