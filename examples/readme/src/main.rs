use koruma_shared_lib::Languages;
use readme::{
    Account, AccountSettings, Address, Customer, Email, Item, LoginForm, Only67u8,
    OptionalSignupForm, SignupForm, SignupInput, User, Username, i18n,
};
use strum::IntoEnumIterator as _;

pub fn main() {
    i18n::init();

    println!("Display-based Error Messages \n");

    let item = Item {
        age: 150,             // Invalid: out of range
        name: "".to_string(), // Invalid: too short
        internal_id: 1,
    };

    match item.validate() {
        Ok(()) => println!("Item is valid!"),
        Err(errors) => {
            println!("Item validation failed:");

            // Access errors by field, then by validator type
            if let Some(age_err) = errors.age().number_range_validation() {
                // Use Display trait for simple string message
                println!("  - age: {}", age_err);
                // Or access the actual value directly
                println!("    (actual value was: {})", age_err.actual());
            }

            if let Some(name_err) = errors.name().string_length_validation() {
                println!("  - name: {}", name_err);
                println!("    (input was: {:?})", name_err.input());
            }

            println!("  - all failed validators (via all()):");
            for err in errors.age().all() {
                println!("    - age: {}", err);
            }
            for err in errors.name().all() {
                println!("    - name: {}", err);
            }
        },
    }

    println!();

    // =========================================================================
    // Nested Validation with Display-based validators
    // =========================================================================
    println!("Nested Validation (Display-based) \n");

    let customer = Customer {
        name: "".to_string(), // Invalid: empty name
        age: 15,              // Invalid: too young (min 18)
        address: Address {
            street: "123 Main St".to_string(),
            city: "".to_string(),        // Invalid: empty city
            zip_code: "ABC".to_string(), // Invalid: not 5 digits
        },
    };

    match customer.validate() {
        Ok(()) => println!("Customer is valid!"),
        Err(errors) => {
            println!("Customer validation failed:");

            // Access top-level field errors
            if let Some(name_err) = errors.name().string_length_validation() {
                println!("  - name: {}", name_err);
            }

            if let Some(age_err) = errors.age().number_range_validation() {
                println!("  - age: {}", age_err);
            }

            // Access nested struct errors
            if let Some(address_err) = errors.address() {
                println!("  - address (nested errors):");

                if let Some(street_err) = address_err.street().string_length_validation() {
                    println!("      - street: {}", street_err);
                }

                if let Some(city_err) = address_err.city().string_length_validation() {
                    println!("      - city: {}", city_err);
                }

                if let Some(zip_err) = address_err.zip_code().zip_code_validation() {
                    println!("      - zip_code: {}", zip_err);
                }
            }
        },
    }

    println!();

    println!("EsFluent-based Error Messages \n");

    let user = User {
        id: 1,                    // Invalid: not even
        username: "".to_string(), // Invalid: empty
    };

    for lang in Languages::iter() {
        i18n::change_locale(lang).expect("Failed to change locale");

        println!(
            ">> Current Language: {:?} : {}",
            lang,
            i18n::localize(&lang)
        );

        match user.validate() {
            Ok(()) => println!("User is valid!"),
            Err(errors) => {
                if let Some(id_err) = errors.id().is_even_number_validation() {
                    // This now prints in the language selected above
                    println!("  - id: {}", i18n::localize(id_err));
                }

                if let Some(username_err) = errors.username().non_empty_string_validation() {
                    println!("  - username: {}", i18n::localize(username_err));
                }
            },
        }
        println!();
    }

    // =========================================================================
    // Nested Validation with EsFluent-based validators
    // =========================================================================
    println!("Nested Validation (EsFluent-based) \n");

    let account = Account {
        id: 3,                 // Invalid: not even
        email: "".to_string(), // Invalid: empty
        settings: AccountSettings {
            max_login_attempts: -5,           // Invalid: not positive
            default_language: "".to_string(), // Invalid: empty
        },
    };

    for lang in Languages::iter() {
        i18n::change_locale(lang).expect("Failed to change locale");

        println!(
            ">> Current Language: {:?} : {}",
            lang,
            i18n::localize(&lang)
        );

        match account.validate() {
            Ok(()) => println!("Account is valid!"),
            Err(errors) => {
                // Access top-level field errors
                if let Some(id_err) = errors.id().is_even_number_validation() {
                    println!("  - id: {}", i18n::localize(id_err));
                }

                if let Some(email_err) = errors.email().non_empty_string_validation() {
                    println!("  - email: {}", i18n::localize(email_err));
                }

                // Access nested struct errors with i18n
                if let Some(settings_err) = errors.settings() {
                    println!("  - settings (nested errors):");

                    if let Some(attempts_err) = settings_err
                        .max_login_attempts()
                        .positive_number_validation()
                    {
                        println!(
                            "      - max_login_attempts: {}",
                            i18n::localize(attempts_err)
                        );
                    }

                    if let Some(lang_err) = settings_err
                        .default_language()
                        .non_empty_string_validation()
                    {
                        println!("      - default_language: {}", i18n::localize(lang_err));
                    }
                }
            },
        }
        println!();
    }

    println!("Valid Data Example \n");

    let valid_item = Item {
        age: 25,
        name: "Alice".to_string(),
        internal_id: 42,
    };

    match valid_item.validate() {
        Ok(()) => println!(
            "Item with age={} and name={:?} is valid!",
            valid_item.age, valid_item.name
        ),
        Err(_) => println!("Unexpected validation failure"),
    }

    let valid_user = User {
        id: 2,
        username: "alice".to_string(),
    };

    match valid_user.validate() {
        Ok(()) => println!(
            "User with id={} and username={:?} is valid!",
            valid_user.id, valid_user.username
        ),
        Err(_) => println!("Unexpected validation failure"),
    }

    // Valid nested structs
    let valid_customer = Customer {
        name: "Bob".to_string(),
        age: 30,
        address: Address {
            street: "456 Oak Ave".to_string(),
            city: "Springfield".to_string(),
            zip_code: "12345".to_string(),
        },
    };

    match valid_customer.validate() {
        Ok(()) => println!(
            "Customer {:?} with address in {:?} is valid!",
            valid_customer.name, valid_customer.address.city
        ),
        Err(_) => println!("Unexpected validation failure"),
    }

    // =========================================================================
    // Newtype Validation with EsFluent-based validators
    // =========================================================================
    println!();
    println!("Newtype Validation (EsFluent-based) \n");

    let signup = SignupForm {
        username: "".to_string(), // Invalid: empty
        // Intentionally bypass try_new to demonstrate validate() still catches it
        email: Email {
            value: "".to_string(),
        }, // Invalid: empty
    };

    for lang in Languages::iter() {
        i18n::change_locale(lang).expect("Failed to change locale");

        println!(
            ">> Current Language: {:?} : {}",
            lang,
            i18n::localize(&lang)
        );

        // Constructor-time validation for the newtype itself
        match Email::try_new("".to_string()) {
            Ok(_) => println!("  - Email::try_new unexpectedly passed"),
            Err(email_errs) => {
                if let Some(err) = email_errs.non_empty_string_validation() {
                    println!("  - email::try_new: {}", i18n::localize(err));
                }

                println!("  - all failed validators from Email::try_new:");
                for err in email_errs.all() {
                    println!("    - {}", i18n::localize(&err));
                }
            },
        }

        match signup.validate() {
            Ok(()) => println!("Signup form is valid!"),
            Err(errors) => {
                if let Some(username_err) = errors.username().non_empty_string_validation() {
                    println!("  - username: {}", i18n::localize(username_err));
                }

                if let Some(inner_err) = errors.email().non_empty_string_validation() {
                    println!("  - email: {}", i18n::localize(inner_err));
                }

                println!("  - all failed validators (via all()):");
                for err in errors.username().all() {
                    println!("    - username: {}", i18n::localize(&err));
                }
                for err in errors.email().all() {
                    println!("    - email: {}", i18n::localize(&err));
                }
            },
        }

        let optional_signup = OptionalSignupForm { email: None };
        println!(
            "Optional signup with no email is valid: {}",
            optional_signup.validate().is_ok()
        );

        let invalid_optional_signup = OptionalSignupForm {
            email: Some(Email {
                value: "".to_string(),
            }),
        };

        if let Err(errors) = invalid_optional_signup.validate()
            && let Some(email_errors) = errors.email()
            && let Some(email_err) = email_errors.non_empty_string_validation()
        {
            println!("  - optional email: {}", i18n::localize(email_err));
        }

        // Unnamed (tuple struct) newtype test
        println!("\n--- Unnamed (Tuple Struct) Newtype ---\n");

        let login = LoginForm {
            username: Username("".to_string()),
        };

        match login.validate() {
            Ok(()) => println!("Login form is valid!"),
            Err(errors) => {
                if let Some(username_err) = errors.username().non_empty_string_validation() {
                    println!(
                        "  - username (unnamed newtype): {}",
                        i18n::localize(username_err)
                    );
                }

                println!("  - all failed validators (via all()):");
                for err in errors.username().all() {
                    println!("    - username: {}", i18n::localize(&err));
                }
            },
        }

        match Username::try_new("".to_string()) {
            Ok(_) => println!("  - Username::try_new unexpectedly passed"),
            Err(username_errs) => {
                if let Some(err) = username_errs.non_empty_string_validation() {
                    println!("  - Username::try_new: {}", i18n::localize(err));
                }

                println!("  - all failed validators from Username::try_new:");
                for err in username_errs.all() {
                    println!("    - {}", i18n::localize(&err));
                }
            },
        }

        // Successful Username creation
        match Username::try_new("alice".to_string()) {
            Ok(username) => println!(
                "  - Username::try_new succeeded: username.0 = {}",
                username.0
            ),
            Err(_) => println!("  - Username::try_new unexpectedly failed"),
        }

        println!();
    }

    // =========================================================================
    // TryFrom integration (newtype(try_from))
    // =========================================================================
    println!("TryFrom Integration (#[koruma(newtype(try_from))]) \n");

    match Only67u8::try_from(69) {
        Ok(n) => println!("  - Only67u8::try_from(69) unexpectedly passed: {}!", n.0),
        Err(errors) => {
            println!("  - Only67u8::try_from(69) failed:");
            for failed in errors.all() {
                println!("    - {}", i18n::localize(&failed));
            }
        },
    }

    match Only67u8::try_from(67) {
        Ok(n) => println!("  - Only67u8::try_from(67) succeeded: {}!", n.0),
        Err(errors) => {
            println!("  - Only67u8::try_from(67) unexpectedly failed:");
            for failed in errors.all() {
                println!("    - {}", i18n::localize(&failed));
            }
        },
    }

    let input = SignupInput {
        username: "".to_string(),
        handle: "bad-handle".to_string(),
        age: 8,
        display_name: None,
    };

    if let Err(errors) = input.validate() {
        if let Some(err) = errors.username().non_empty_validation() {
            println!("username: {err}");
        }

        if let Some(err) = errors.handle().handle_ascii() {
            println!("handle(ascii): {err}");
        }

        for err in errors.handle().all() {
            println!("handle(any): {err}");
        }
    }
}
