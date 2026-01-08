use es_fluent::ToFluentString as _;
use koruma_custom_example::{Item, User, i18n};
use koruma_shared_lib::Languages;
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
}
