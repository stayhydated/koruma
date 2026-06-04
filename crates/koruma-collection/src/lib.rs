#![doc = include_str!("../README.md")]

mod validators;
pub use validators::*;

#[doc(hidden)]
#[cfg(feature = "fluent")]
pub mod i18n;

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    #[test]
    fn fluent_i18n_assets_are_registered() {
        es_fluent_manager_embedded::EmbeddedI18n::try_new()
            .expect("koruma-collection i18n assets should register");
    }

    #[cfg(feature = "internal-showcase")]
    #[test]
    fn showcase_validators_create_dynamic_validators() {
        use std::collections::BTreeMap;

        use koruma::showcase::{InputType, ValidatorModule, validators};

        crate::__link_showcase_validators();

        struct Case {
            name: &'static str,
            module: ValidatorModule,
            input_type: InputType,
            valid_input: &'static str,
        }

        let cases = [
            Case {
                name: "Alphanumeric",
                module: ValidatorModule::String,
                input_type: InputType::Text,
                valid_input: "abc123",
            },
            Case {
                name: "ASCII",
                module: ValidatorModule::String,
                input_type: InputType::Text,
                valid_input: "hello",
            },
            Case {
                name: "Contains 'test'",
                module: ValidatorModule::String,
                input_type: InputType::Text,
                valid_input: "test-value",
            },
            Case {
                name: "Matches Value",
                module: ValidatorModule::String,
                input_type: InputType::Text,
                valid_input: "expected",
            },
            Case {
                name: "Regex Pattern",
                module: ValidatorModule::String,
                input_type: InputType::Text,
                valid_input: "abc_123",
            },
            Case {
                name: "Prefix 'hello'",
                module: ValidatorModule::String,
                input_type: InputType::Text,
                valid_input: "hello world",
            },
            Case {
                name: "Suffix '.rs'",
                module: ValidatorModule::String,
                input_type: InputType::Text,
                valid_input: "main.rs",
            },
            Case {
                name: "Credit Card",
                module: ValidatorModule::Format,
                input_type: InputType::Text,
                valid_input: "4111111111111111",
            },
            Case {
                name: "Email",
                module: ValidatorModule::Format,
                input_type: InputType::Text,
                valid_input: "user@example.com",
            },
            Case {
                name: "IP Address",
                module: ValidatorModule::Format,
                input_type: InputType::Text,
                valid_input: "127.0.0.1",
            },
            Case {
                name: "Phone Number",
                module: ValidatorModule::Format,
                input_type: InputType::Text,
                valid_input: "+14155552671",
            },
            Case {
                name: "URL",
                module: ValidatorModule::Format,
                input_type: InputType::Text,
                valid_input: "https://example.com",
            },
            Case {
                name: "Negative Number",
                module: ValidatorModule::Numeric,
                input_type: InputType::Numeric,
                valid_input: "-1",
            },
            Case {
                name: "Non-Negative Number",
                module: ValidatorModule::Numeric,
                input_type: InputType::Numeric,
                valid_input: "0",
            },
            Case {
                name: "Non-Positive Number",
                module: ValidatorModule::Numeric,
                input_type: InputType::Numeric,
                valid_input: "0",
            },
            Case {
                name: "Positive Number",
                module: ValidatorModule::Numeric,
                input_type: InputType::Numeric,
                valid_input: "1",
            },
            Case {
                name: "Range [0, 100)",
                module: ValidatorModule::Numeric,
                input_type: InputType::Numeric,
                valid_input: "42",
            },
            Case {
                name: "Length",
                module: ValidatorModule::Collection,
                input_type: InputType::Text,
                valid_input: "abc",
            },
            Case {
                name: "NonEmpty",
                module: ValidatorModule::Collection,
                input_type: InputType::Text,
                valid_input: "x",
            },
        ];

        let by_name: BTreeMap<_, _> = validators()
            .into_iter()
            .map(|showcase| (showcase.name, showcase))
            .collect();

        for case in cases {
            let showcase = by_name
                .get(case.name)
                .unwrap_or_else(|| panic!("missing showcase validator `{}`", case.name));
            assert_eq!(showcase.module, case.module);
            assert_eq!(showcase.input_type, case.input_type);
            assert!(!showcase.description.is_empty());

            let validator = (showcase.create_validator)(case.valid_input)
                .unwrap_or_else(|err| panic!("failed to create `{}`: {err}", case.name));
            assert!(
                validator.is_valid(),
                "`{}` should accept `{}`",
                case.name,
                case.valid_input
            );
            assert!(!validator.display_string().is_empty());

            #[cfg(feature = "fluent")]
            {
                assert!(!validator.fluent_string().is_empty());

                let mut localize =
                    |domain: es_fluent::registry::StaticFluentDomain,
                     id: es_fluent::registry::StaticFluentEntryId,
                     _args: Option<&es_fluent::FluentArgs<'_>>| {
                        format!("{}:{}", domain.as_str(), id.as_str())
                    };
                assert!(validator.fluent_string_with(&mut localize).contains(':'));
            }
        }
    }

    #[cfg(feature = "internal-showcase")]
    #[test]
    fn showcase_numeric_factories_report_parse_errors() {
        use koruma::showcase::validators;

        crate::__link_showcase_validators();
        let all = validators();

        for name in [
            "Negative Number",
            "Non-Negative Number",
            "Non-Positive Number",
            "Positive Number",
            "Range [0, 100)",
        ] {
            let showcase = all
                .iter()
                .find(|showcase| showcase.name == name)
                .unwrap_or_else(|| panic!("missing showcase validator `{name}`"));
            assert!((showcase.create_validator)("not-a-number").is_err());
        }
    }
}
