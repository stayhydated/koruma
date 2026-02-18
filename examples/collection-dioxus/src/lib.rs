mod i18n;

use dioxus::prelude::*;
use es_fluent::ToFluentString;
use koruma::showcase::{ValidatorShowcase, validators};
use koruma_shared_lib::Languages;
use std::collections::HashMap;
use strum::IntoEnumIterator;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidatorModule {
    String,
    Format,
    Numeric,
    Collection,
    General,
}

impl ValidatorModule {
    pub const ALL: [Self; 5] = [
        Self::String,
        Self::Format,
        Self::Numeric,
        Self::Collection,
        Self::General,
    ];

    pub fn available_modules(all_validators: &[&'static ValidatorShowcase]) -> Vec<Self> {
        Self::ALL
            .iter()
            .filter(|&&m| all_validators.iter().any(|&v| m.contains_validator(v)))
            .copied()
            .collect()
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Format => "Format",
            Self::Numeric => "Numeric",
            Self::Collection => "Collection",
            Self::General => "General",
        }
    }

    pub fn contains_validator(&self, showcase: &ValidatorShowcase) -> bool {
        match self {
            Self::String => showcase.module == "string",
            Self::Format => showcase.module == "format",
            Self::Numeric => showcase.module == "numeric",
            Self::Collection => showcase.module == "collection",
            Self::General => showcase.module == "general",
        }
    }
}

fn filter_validators_by_module(
    all: &[&'static ValidatorShowcase],
    module: ValidatorModule,
) -> Vec<&'static ValidatorShowcase> {
    all.iter()
        .filter(|&&v| module.contains_validator(v))
        .copied()
        .collect()
}

#[component]
pub fn App() -> Element {
    let mut inputs = use_signal(HashMap::<&'static str, String>::new);
    let mut current_language = use_signal(Languages::default);

    use_resource(move || async move {
        i18n::init();
        let _ = i18n::change_locale(current_language());
    });

    koruma_collection::__link_showcase_validators();
    let all_validators = validators();
    let available_modules = ValidatorModule::available_modules(&all_validators);

    rsx! {
        div { class: "min-h-screen bg-black p-8 font-sans",
            style { {include_str!("styles.css")} }
            div { class: "max-w-4xl mx-auto space-y-8",
                div { class: "flex justify-between items-center mb-8",
                    h1 { class: "text-2xl font-bold", style: "color: #00f0ff;", "Koruma Validators" }
                    div { class: "flex items-center gap-4",
                        select {
                            class: "bg-black border rounded px-3 py-1 text-sm",
                            style: "color: #ff00a0; border-color: rgba(255, 0, 160, 0.5);",
                            onchange: move |e| {
                                for lang in Languages::iter() {
                                    if lang.to_fluent_string() == e.value() {
                                        current_language.set(lang);
                                        let _ = i18n::change_locale(lang);
                                    }
                                }
                            },
                            for lang in Languages::iter() {
                                option {
                                    value: lang.to_fluent_string(),
                                    selected: lang == current_language(),
                                    {lang.to_fluent_string()}
                                }
                            }
                        }
                    }
                }

                for module in available_modules.iter() {
                    div { class: "space-y-4",
                        h1 {
                            class: "text-xl font-bold pb-2 border-b",
                            style: "color: #00f0ff; border-color: rgba(0, 240, 255, 0.3);",
                            {module.name()}
                        }

                        for validator in filter_validators_by_module(&all_validators, *module) {
                            {
                                let name = validator.name;
                                let description = validator.description;
                                let input_val = inputs.read().get(name).cloned().unwrap_or_default();
                                let current_validator = (validator.create_validator)(&input_val);

                                let (border_color, text_color, glow) = match &current_validator {
                                    Ok(v) if v.is_valid() => ("#a0ff00", "#a0ff00", "box-shadow: 0 0 10px rgba(160, 255, 0, 0.3), inset 0 0 5px rgba(160, 255, 0, 0.2);"),
                                    Ok(_) => ("#ff00a0", "#ff00a0", "box-shadow: 0 0 10px rgba(255, 0, 160, 0.3), inset 0 0 5px rgba(255, 0, 160, 0.2);"),
                                    Err(_) => ("#00f0ff", "#00f0ff", "box-shadow: 0 0 10px rgba(0, 240, 255, 0.3), inset 0 0 5px rgba(0, 240, 255, 0.2);"),
                                };

                                let (status_emoji, display_msg, fluent_msg, error_msg) = match &current_validator {
                                    Ok(v) if v.is_valid() => ("✓", v.display_string(), v.fluent_string(), None),
                                    Ok(v) => ("✗", v.display_string(), v.fluent_string(), None),
                                    Err(e) => ("!", String::new(), String::new(), Some(format!("Error: {}", e))),
                                };

                                rsx! {
                                    div { class: "space-y-2",
                                        h2 {
                                            class: "text-lg font-semibold",
                                            style: "color: {text_color};",
                                            {name}
                                        }
                                        p { class: "text-xs mb-2", style: "color: #555555;", {description} }

                                        div { class: "relative",
                                            input {
                                                r#type: "text",
                                                class: "w-full bg-black border rounded px-4 py-3 pr-10 focus:outline-none focus:ring-2",
                                                style: "color: {text_color}; border-color: {border_color}; {glow} --tw-ring-color: rgba(0, 240, 255, 0.5);",
                                                value: input_val.clone(),
                                                oninput: move |e| {
                                                    inputs.write().insert(name, e.value());
                                                },
                                                placeholder: "Enter value to validate..."
                                            }
                                            span { class: "absolute right-3 top-1/2 -translate-y-1/2 text-xl", {status_emoji} }
                                        }

                                        if !display_msg.is_empty() || error_msg.is_some() {
                                            div {
                                                class: "border rounded p-3 text-sm",
                                                style: "color: {text_color}; border-color: {border_color}; {glow}",
                                                if let Some(err) = error_msg {
                                                    p { {err.clone()} }
                                                } else {
                                                    div { class: "space-y-2",
                                                        p { style: "color: {text_color};", {display_msg} }
                                                        if !fluent_msg.is_empty() {
                                                            p { class: "text-xs opacity-75", {fluent_msg} }
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
                }
            }
        }
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    dioxus::launch(App);
}
