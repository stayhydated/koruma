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
            "w-full bg-black border rounded px-4 py-3 text-green-400 border-green-400 focus:outline-none focus:ring-2 focus:ring-cyan-400/50"
        },
        Some(Ok(_)) => {
            "w-full bg-black border rounded px-4 py-3 text-red-400 border-red-400 focus:outline-none focus:ring-2 focus:ring-cyan-400/50"
        },
        Some(Err(_)) => {
            "w-full bg-black border rounded px-4 py-3 text-yellow-400 border-yellow-400 focus:outline-none focus:ring-2 focus:ring-cyan-400/50"
        },
        None => {
            "w-full bg-black border rounded px-4 py-3 text-gray-400 border-gray-400 focus:outline-none focus:ring-2 focus:ring-cyan-400/50"
        },
    };

    let status_emoji = match &current_validator {
        Some(Ok(v)) if v.is_valid() => "✅",
        Some(Ok(_)) => "❌",
        Some(Err(_)) => "⚠️",
        None => "",
    };

    let result_class = match &current_validator {
        Some(Ok(v)) if v.is_valid() => "border rounded p-4 text-green-400 border-green-400",
        Some(Ok(_)) => "border rounded p-4 text-pink-400 border-pink-400",
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
        div { class: "min-h-screen bg-black text-gray-200 p-8 font-sans",
            div { class: "max-w-4xl mx-auto space-y-6",
                h1 { class: "text-3xl font-bold text-cyan-400 mb-8 text-center",
                    "Koruma Collection Showcase"
                }

                div { class: "grid grid-cols-2 gap-4",
                    div { class: "space-y-2",
                        label { class: "text-sm text-cyan-400", "Module" }
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
                                class: "w-full bg-black border border-cyan-400/50 text-cyan-400 rounded px-3 py-2",
                                SelectValue {}
                            }
                            SelectList {
                                aria_label: "Module list",
                                class: "bg-black border border-cyan-400/50 rounded shadow-lg",
                                for (idx, module) in available_modules.iter().enumerate() {
                                    SelectOption::<String> {
                                        index: idx,
                                        value: module.name().to_string(),
                                        class: "px-3 py-2 hover:bg-cyan-400/10 cursor-pointer text-gray-200",
                                        {module.name()}
                                        SelectItemIndicator { "✓" }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "space-y-2",
                        label { class: "text-sm text-cyan-400", "Validator" }
                        Select::<String> {
                            placeholder: "Select validator...",
                            default_value: selected_validator_name().unwrap_or_default(),
                            on_value_change: move |v: Option<String>| {
                                selected_validator_name.set(v);
                                input.set(String::new());
                            },
                            SelectTrigger {
                                aria_label: "Validator selector",
                                class: "w-full bg-black border border-yellow-400/50 text-yellow-400 rounded px-3 py-2",
                                SelectValue {}
                            }
                            SelectList {
                                aria_label: "Validator list",
                                class: "bg-black border border-yellow-400/50 rounded shadow-lg max-h-60 overflow-y-auto",
                                for (idx, validator) in current_validators.iter().enumerate() {
                                    SelectOption::<String> {
                                        index: idx,
                                        value: validator.name.to_string(),
                                        class: "px-3 py-2 hover:bg-yellow-400/10 cursor-pointer text-gray-200",
                                        div { class: "flex flex-col",
                                            span { class: "font-medium", {validator.name} }
                                            span { class: "text-xs text-gray-500", {validator.description} }
                                        }
                                        SelectItemIndicator { "✓" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "space-y-2",
                    label { class: "text-sm text-cyan-400", "Input" }
                    div { class: "relative",
                        input {
                            r#type: "text",
                            class: status_class,
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
                            div { class: result_class,
                                label { class: "text-sm text-cyan-400 block mb-2", "Display Output" }
                                p { class: "text-lg", {display_msg} }
                            }

                            div { class: "border rounded p-4 border-blue-400 text-blue-400",
                                label { class: "text-sm text-cyan-400 block mb-2", "Fluent Output" }
                                p { class: "text-lg", {fluent_msg} }
                            }
                        }
                    } else if matches!(&current_validator, Some(Err(_))) {
                        div { class: "border border-yellow-400 rounded p-4 text-yellow-400",
                            label { class: "text-sm text-cyan-400 block mb-2", "Parse Error" }
                            p { class: "text-lg", {error_msg} }
                        }
                    }
                } else {
                    div { class: "border border-gray-600 rounded p-4 text-gray-500 text-center",
                        "No validator selected"
                    }
                }

                div { class: "flex justify-between items-center pt-6 border-t border-gray-800",
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
                            class: "bg-black border border-purple-400/50 text-purple-400 rounded px-3 py-1 text-sm",
                            SelectValue {}
                        }
                        SelectList {
                            aria_label: "Language list",
                            class: "bg-black border border-purple-400/50 rounded shadow-lg",
                            for (idx, lang) in Languages::iter().enumerate() {
                                SelectOption::<String> {
                                    index: idx,
                                    value: lang.to_fluent_string(),
                                    class: "px-3 py-2 hover:bg-purple-400/10 cursor-pointer text-gray-200",
                                    {lang.to_fluent_string()}
                                    SelectItemIndicator { "✓" }
                                }
                            }
                        }
                    }

                    a {
                        href: "/koruma/",
                        class: "text-cyan-400 hover:text-cyan-300 text-sm transition-colors",
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
