use std::collections::HashMap;

use dioxus::events::FormData;
use dioxus::prelude::*;
use dioxus_primitives::label::Label;
use es_fluent::registry::{StaticFluentDomain, StaticFluentEntryId};
use es_fluent::{FluentArgs, FluentLocalizer as _};
use es_fluent_manager_dioxus::{DioxusAssetI18nHandle, DioxusAssetI18nProvider, use_i18n};
use koruma::showcase::{ValidatorModule, ValidatorShowcase, validators};
use koruma_collection::__link_showcase_validators;
use stayhydated_dioxus::{
    StayhydatedSiteLanguage as _, TabContent, TabList, TabTrigger, Tabs, TabsOrientation,
    surface_reveal_style,
};

use crate::components::{ContributePanel, FooterPanel, PageHeader};
use crate::pages::DemoLanguageSwitcher;
use crate::pages::i18n::{DemoLanguage, DioxusShowcaseMessage};
use crate::site::routing::PageKind;

#[derive(Clone, Copy)]
enum ValidatorState {
    Valid,
    Invalid,
    Error,
}

struct ValidatorRowMessages {
    display_message: String,
    fluent_message: String,
    error_message: Option<String>,
    input_placeholder: String,
    message_heading: String,
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

    fn message_heading(self, i18n: &DioxusAssetI18nHandle) -> String {
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
    rsx! {
        DioxusAssetI18nProvider {
            initial_language: DemoLanguage::default().language_identifier(),
            CollectionDioxusShowcase {}
        }
    }
}

#[component]
fn CollectionDioxusShowcase() -> Element {
    let showcase_style = surface_reveal_style();
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
    let available_modules = available_modules(&all_validators);
    let default_module = available_modules
        .first()
        .map(|module| module.as_str())
        .unwrap_or("string");

    rsx! {
        div { class: "page-shell",
            PageHeader { current_page: PageKind::CollectionDioxus }
            main { class: "stack",
                section { class: "section-band collection-demo-card motion-reveal",
                    style: showcase_style.as_str(),
                    div { class: "demo-card-header",
                        div { class: "demo-card-heading",
                            span { class: "panel-label", "{panel_label}" }
                            h1 { "{page_title}" }
                            p { "{page_intro_body}" }
                        }
                        div { class: "demo-card-controls",
                            DemoLanguageSwitcher {}
                        }
                    }
                    Tabs {
                        default_value: default_module.to_string(),
                        orientation: TabsOrientation::Horizontal,
                        TabList {
                                for (index, module) in available_modules.iter().enumerate() {
                                    TabTrigger {
                                    value: module.as_str().to_string(),
                                    index: index,
                                    "{module_name(*module, &i18n)}"
                                }
                            }
                        }
                        for (index, module) in available_modules.iter().enumerate() {
                            TabContent {
                                index: index,
                                value: module.as_str().to_string(),
                                div {
                                    class: "collection-module",
                                    h2 { "{module_name(*module, &i18n)}" }
                                    for validator in all_validators
                                        .iter()
                                        .filter(|&validator| validator.module == *module)
                                    {
                                        {
                                            let validator_name = validator.name;
                                            let input =
                                                inputs.read().get(validator_name).cloned().unwrap_or_default();
                                            let current_validator = (validator.create_validator)(&input);
                                            let current_state = match &current_validator {
                                                Ok(v) if v.is_valid() => ValidatorState::Valid,
                                                Ok(_) => ValidatorState::Invalid,
                                                Err(_) => ValidatorState::Error,
                                            };
                                            let message_heading = current_state.message_heading(&i18n);
                                            let (display_msg, fluent_msg, error_msg) =
                                                match current_validator {
                                                    Ok(v) if v.is_valid() => (
                                                        v.display_string(),
                                                        v.fluent_string_with(&mut |domain, id, args| {
                                                            localize_with_args(
                                                                &i18n, domain, id, args,
                                                            )
                                                        }),
                                                        None,
                                                    ),
                                                    Ok(v) => (
                                                        v.display_string(),
                                                        v.fluent_string_with(&mut |domain, id, args| {
                                                            localize_with_args(
                                                                &i18n, domain, id, args,
                                                            )
                                                        }),
                                                        None,
                                                    ),
                                                    Err(error) => (
                                                        String::new(),
                                                        String::new(),
                                                        Some(format!("{error_prefix} {error}")),
                                                    ),
                                                };
                                            let module_key = module.as_str();
                                            let input_id = format!(
                                                "validator-{module_key}-{}",
                                                validator.name.to_lowercase().replace(' ', "-")
                                            );
                                            validator_row(
                                                input_id,
                                                validator,
                                                input,
                                                EventHandler::new({
                                                    move |event: Event<FormData>| {
                                                        inputs
                                                            .write()
                                                            .insert(validator_name, event.value());
                                                    }
                                                }),
                                                current_state,
                                                ValidatorRowMessages {
                                                    display_message: display_msg,
                                                    fluent_message: fluent_msg,
                                                    error_message: error_msg,
                                                    input_placeholder: validation_placeholder.clone(),
                                                    message_heading,
                                                },
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
    validator: &ValidatorShowcase,
    input: String,
    on_input: EventHandler<Event<FormData>>,
    state: ValidatorState,
    messages: ValidatorRowMessages,
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
            placeholder: messages.input_placeholder,
            oninput: on_input,
        }
            p { class: "validator-message-heading", "{messages.message_heading}:" }
            {
                match messages.error_message {
                    Some(error) => rsx! {
                        p { class: "validator-message validator-message-error", "{error}" }
                    },
                    None => rsx! {
                        p { class: "validator-message", "{messages.display_message}" }
                        if !messages.fluent_message.is_empty() {
                            p {
                                class: "validator-message validator-message-subtle",
                                "{messages.fluent_message}"
                            }
                        }
                    },
                }
            }
        }
    }
}

fn localize_with_args<'a>(
    i18n: &DioxusAssetI18nHandle,
    domain: StaticFluentDomain,
    id: StaticFluentEntryId,
    args: Option<&FluentArgs<'a>>,
) -> String {
    i18n.localize_in_domain(domain, id, args)
        .unwrap_or_else(|| id.as_str().to_string())
}
fn available_modules(all_validators: &[&'static ValidatorShowcase]) -> Vec<ValidatorModule> {
    ValidatorModule::ALL
        .iter()
        .filter(|&&module| {
            all_validators
                .iter()
                .any(|showcase| showcase.module == module)
        })
        .copied()
        .collect()
}

fn module_name(module: ValidatorModule, i18n: &DioxusAssetI18nHandle) -> String {
    match module {
        ValidatorModule::String => i18n.localize_message(&DioxusShowcaseMessage::ModuleString),
        ValidatorModule::Format => i18n.localize_message(&DioxusShowcaseMessage::ModuleFormat),
        ValidatorModule::Numeric => i18n.localize_message(&DioxusShowcaseMessage::ModuleNumeric),
        ValidatorModule::Collection => {
            i18n.localize_message(&DioxusShowcaseMessage::ModuleCollection)
        },
        ValidatorModule::General => i18n.localize_message(&DioxusShowcaseMessage::ModuleGeneral),
    }
}
