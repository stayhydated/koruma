mod i18n;

use dioxus::prelude::*;
use dioxus_primitives::select::{
    Select, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue,
};
use es_fluent::ToFluentString;
use koruma::showcase::{ValidatorShowcase, validators};
use koruma_shared_lib::Languages;
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
    let mut input = use_signal(String::new);
    let mut selected_module = use_signal(|| ValidatorModule::String);
    let mut selected_validator_name = use_signal(|| None::<String>);
    let mut current_language = use_signal(Languages::default);

    use_resource(move || async move {
        i18n::init();
        let _ = i18n::change_locale(current_language());
    });

    koruma_collection::__link_showcase_validators();
    let all_validators = validators();
    let available_modules = ValidatorModule::available_modules(&all_validators);

    let current_validators = filter_validators_by_module(&all_validators, selected_module());
    let current_showcase = selected_validator_name()
        .and_then(|name| current_validators.iter().find(|v| v.name == name).copied());

    let current_validator = current_showcase.map(|showcase| (showcase.create_validator)(&input()));

    let status_class = match &current_validator {
        Some(Ok(v)) if v.is_valid() => {
            "w-full bg-black border rounded px-4 py-3 focus:outline-none focus:ring-2"
        },
        Some(Ok(_)) => "w-full bg-black border rounded px-4 py-3 focus:outline-none focus:ring-2",
        Some(Err(_)) => "w-full bg-black border rounded px-4 py-3 focus:outline-none focus:ring-2",
        None => "w-full bg-black border rounded px-4 py-3 focus:outline-none focus:ring-2",
    };

    let status_style = match &current_validator {
        Some(Ok(v)) if v.is_valid() => {
            "color: #a0ff00; border-color: #a0ff00; box-shadow: 0 0 10px rgba(160, 255, 0, 0.3), inset 0 0 5px rgba(160, 255, 0, 0.2); --tw-ring-color: rgba(0, 240, 255, 0.5);"
        },
        Some(Ok(_)) => {
            "color: #ff00a0; border-color: #ff00a0; box-shadow: 0 0 10px rgba(255, 0, 160, 0.3), inset 0 0 5px rgba(255, 0, 160, 0.2); --tw-ring-color: rgba(0, 240, 255, 0.5);"
        },
        Some(Err(_)) => {
            "color: #a0ff00; border-color: #a0ff00; box-shadow: 0 0 10px rgba(160, 255, 0, 0.3), inset 0 0 5px rgba(160, 255, 0, 0.2); --tw-ring-color: rgba(0, 240, 255, 0.5);"
        },
        None => "color: #555555; border-color: #555555; --tw-ring-color: rgba(0, 240, 255, 0.5);",
    };

    let status_emoji = match &current_validator {
        Some(Ok(v)) if v.is_valid() => "✓",
        Some(Ok(_)) => "✗",
        Some(Err(_)) => "!",
        None => "",
    };

    let result_class = match &current_validator {
        Some(Ok(v)) if v.is_valid() => "border rounded p-4",
        Some(Ok(_)) => "border rounded p-4",
        _ => "",
    };

    let result_style = match &current_validator {
        Some(Ok(v)) if v.is_valid() => {
            "color: #a0ff00; border-color: #a0ff00; box-shadow: 0 0 10px rgba(160, 255, 0, 0.3), inset 0 0 5px rgba(160, 255, 0, 0.2);"
        },
        Some(Ok(_)) => {
            "color: #ff00a0; border-color: #ff00a0; box-shadow: 0 0 10px rgba(255, 0, 160, 0.3), inset 0 0 5px rgba(255, 0, 160, 0.2);"
        },
        _ => "",
    };

    let display_msg = match &current_validator {
        Some(Ok(v)) => v.display_string(),
        _ => String::new(),
    };

    let fluent_msg = match &current_validator {
        Some(Ok(v)) => v.fluent_string(),
        _ => String::new(),
    };

    let error_msg = match &current_validator {
        Some(Err(e)) => format!("Error: {}", e),
        _ => String::new(),
    };

    rsx! {
        div { class: "min-h-screen bg-black p-8 font-sans",
            style { {include_str!("styles.css")} }
            div { class: "max-w-4xl mx-auto space-y-6",
                div { class: "grid grid-cols-2 gap-4",
                    div { class: "space-y-2",
                        label { class: "text-sm", style: "color: #00f0ff;", "Module" }
                        Select::<String> {
                            placeholder: "Select module...",
                            default_value: selected_module().name().to_string(),
                            on_value_change: move |v: Option<String>| {
                                if let Some(v) = v {
                                    for module in ValidatorModule::ALL {
                                        if module.name() == v {
                                            selected_module.set(module);
                                            selected_validator_name.set(None);
                                            input.set(String::new());
                                        }
                                    }
                                }
                            },
                            SelectTrigger {
                                aria_label: "Module selector",
                                class: "w-full bg-black border rounded px-3 py-2",
                                style: "color: #00f0ff; border-color: rgba(0, 240, 255, 0.5);",
                                SelectValue {}
                            }
                            SelectList {
                                aria_label: "Module list",
                                class: "bg-black border rounded shadow-lg select-list-primary",
                                for (idx, module) in available_modules.iter().enumerate() {
                                    SelectOption::<String> {
                                        index: idx,
                                        value: module.name().to_string(),
                                        class: "select-option-primary",
                                        span { style: "color: #e0e0e0;", {module.name()} }
                                        SelectItemIndicator { span { style: "color: #00f0ff;", "✓" } }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "space-y-2",
                        label { class: "text-sm", style: "color: #00f0ff;", "Validator" }
                        Select::<String> {
                            placeholder: "Select validator...",
                            default_value: selected_validator_name().unwrap_or_default(),
                            on_value_change: move |v: Option<String>| {
                                selected_validator_name.set(v);
                                input.set(String::new());
                            },
                            SelectTrigger {
                                aria_label: "Validator selector",
                                class: "w-full bg-black border rounded px-3 py-2",
                                style: "color: #a0ff00; border-color: rgba(160, 255, 0, 0.5);",
                                SelectValue {}
                            }
                            SelectList {
                                aria_label: "Validator list",
                                class: "bg-black border rounded shadow-lg max-h-60 overflow-y-auto select-list-accent",
                                for (idx, validator) in current_validators.iter().enumerate() {
                                    SelectOption::<String> {
                                        index: idx,
                                        value: validator.name.to_string(),
                                        class: "select-option-accent",
                                        div { class: "flex flex-col",
                                            span { class: "font-medium", style: "color: #e0e0e0;", {validator.name} }
                                            span { class: "text-xs", style: "color: #555555;", {validator.description} }
                                        }
                                        SelectItemIndicator { span { style: "color: #a0ff00;", "✓" } }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-2",
                    label { class: "text-sm", style: "color: #00f0ff;", "Input" }
                    div { class: "relative",
                        input {
                            r#type: "text",
                            class: status_class,
                            style: status_style,
                            value: input(),
                            oninput: move |e| input.set(e.value()),
                            placeholder: "Enter value to validate..."
                        }
                        span { class: "absolute right-3 top-1/2 -translate-y-1/2 text-xl", {status_emoji} }
                    }
                }

                if current_validator.is_some() {
                    if matches!(&current_validator, Some(Ok(_))) {
                        div { class: "space-y-4",
                            div {
                                class: result_class,
                                style: result_style,
                                label { class: "text-sm block mb-2", style: "color: #00f0ff;", "Display Output" }
                                p { class: "text-lg", style: "color: inherit;", {display_msg} }
                            }

                            div {
                                class: "border rounded p-4",
                                style: "color: #00f0ff; border-color: rgba(0, 240, 255, 0.5); box-shadow: 0 0 10px rgba(0, 240, 255, 0.3), inset 0 0 5px rgba(0, 240, 255, 0.2);",
                                label { class: "text-sm block mb-2", style: "color: #00f0ff;", "Fluent Output" }
                                p { class: "text-lg", {fluent_msg} }
                            }
                        }
                    } else if matches!(&current_validator, Some(Err(_))) {
                        div {
                            class: "border rounded p-4",
                            style: "color: #a0ff00; border-color: #a0ff00; box-shadow: 0 0 10px rgba(160, 255, 0, 0.3), inset 0 0 5px rgba(160, 255, 0, 0.2);",
                            label { class: "text-sm block mb-2", style: "color: #00f0ff;", "Parse Error" }
                            p { class: "text-lg", {error_msg} }
                        }
                    }
                } else {
                    div {
                        class: "border rounded p-4 text-center",
                        style: "color: #555555; border-color: #1a1a1a;",
                        "No validator selected"
                    }
                }

                div { class: "flex justify-between items-center pt-6 border-t",
                    style: "border-color: #1a1a1a;",
                    Select::<String> {
                        placeholder: "Language...",
                        default_value: current_language().to_fluent_string(),
                        on_value_change: move |v: Option<String>| {
                            if let Some(v) = v {
                                for lang in Languages::iter() {
                                    if lang.to_fluent_string() == v {
                                        current_language.set(lang);
                                        let _ = i18n::change_locale(lang);
                                    }
                                }
                            }
                        },
                        SelectTrigger {
                            aria_label: "Language selector",
                            class: "bg-black border rounded px-3 py-1 text-sm",
                            style: "color: #ff00a0; border-color: rgba(255, 0, 160, 0.5);",
                            SelectValue {}
                        }
                        SelectList {
                            aria_label: "Language list",
                            class: "bg-black border rounded shadow-lg select-list-secondary",
                            for (idx, lang) in Languages::iter().enumerate() {
                                SelectOption::<String> {
                                    index: idx,
                                    value: lang.to_fluent_string(),
                                    class: "select-option-secondary",
                                    span { style: "color: #e0e0e0;", {lang.to_fluent_string()} }
                                    SelectItemIndicator { span { style: "color: #ff00a0;", "✓" } }
                                }
                            }
                        }
                    }

                    a {
                        href: "/koruma/",
                        class: "text-sm transition-colors link-primary",
                        "← Back to Examples"
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
