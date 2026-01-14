use es_fluent::ToFluentString as _;
use koruma_shared_lib::Languages;
use koruma_user_defined_example::{
    Account, AccountSettings, Address, Customer, Email, Item, SignupForm, User, i18n,
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
                println!("    (actual value was: {})", age_err.actual);
            }

            if let Some(name_err) = errors.name().string_length_validation() {
                println!("  - name: {}", name_err);
                println!("    (input was: {:?})", name_err.input);
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
            lang.to_fluent_string()
        );

        match user.validate() {
            Ok(()) => println!("User is valid!"),
            Err(errors) => {
                use es_fluent::ToFluentString;

                if let Some(id_err) = errors.id().is_even_number_validation() {
                    // This now prints in the language selected above
                    println!("  - id: {}", id_err.to_fluent_string());
                }

                if let Some(username_err) = errors.username().non_empty_string_validation() {
                    println!("  - username: {}", username_err.to_fluent_string());
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
            lang.to_fluent_string()
        );

        match account.validate() {
            Ok(()) => println!("Account is valid!"),
            Err(errors) => {
                use es_fluent::ToFluentString;

                // Access top-level field errors
                if let Some(id_err) = errors.id().is_even_number_validation() {
                    println!("  - id: {}", id_err.to_fluent_string());
                }

                if let Some(email_err) = errors.email().non_empty_string_validation() {
                    println!("  - email: {}", email_err.to_fluent_string());
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
                            attempts_err.to_fluent_string()
                        );
                    }

                    if let Some(lang_err) = settings_err
                        .default_language()
                        .non_empty_string_validation()
                    {
                        println!("      - default_language: {}", lang_err.to_fluent_string());
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
        username: "".to_string(),     // Invalid: empty
        email: Email("".to_string()), // Invalid: empty
    };

    for lang in Languages::iter() {
        i18n::change_locale(lang).expect("Failed to change locale");

        println!(
            ">> Current Language: {:?} : {}",
            lang,
            lang.to_fluent_string()
        );

        match signup.validate() {
            Ok(()) => println!("Signup form is valid!"),
            Err(errors) => {
                use es_fluent::ToFluentString;

                if let Some(username_err) = errors.username().non_empty_string_validation() {
                    println!("  - username: {}", username_err.to_fluent_string());
                }

                // Access newtype field errors
                // Note: The structure of errors for a newtype field depends on how the derive is implemented.
                // Assuming #[koruma(newtype)] flattens or provides access similarly to nested.

                // When using #[koruma(newtype)], the generated error accessor for the field
                // typically returns the error type of the inner type directly, or a wrapper that behaves like it.
                // If Email wraps String, errors.email() might return errors for String fields.
                // However, Email is a SINGLE tuple struct.
                // So errors.email() likely gives us access to validators on that single field.

                // Let's check how many levels we need to go down.
                // Email(String) -> The String has validators.
                // SignupForm has field `email: Email`.
                // errors.email() -> EmailError
                // EmailError has methods for the inner field validators?

                // With #[koruma(newtype)] on the struct Email, it should expose "all" errors or behave transparently.
                // With #[koruma(newtype)] on the field `email`, it might just expose the inner error type directly.

                // Accessing the validator error on the newtype - now friction-free, no ? needed
                // errors.email() returns &InnerError directly
                if let Some(inner_err) = errors.email().non_empty_string_validation() {
                    println!("  - email: {}", inner_err.to_fluent_string());
                }
            },
        }
        println!();
    }
}
