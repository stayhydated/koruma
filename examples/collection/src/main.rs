use es_fluent::ToFluentString as _;
use koruma_collection_example::{BlogPost, Order, Product, UserProfile, i18n};
use shared_lib::Languages;
use strum::IntoEnumIterator as _;

fn main() {
    i18n::init();

    println!("=== Display-based Error Messages ===\n");

    // Example 1: Order validation
    let invalid_order = Order { items: vec![] };
    match invalid_order.validate() {
        Ok(()) => println!("Order is valid!"),
        Err(errors) => {
            println!("Order validation failed:");
            if let Some(err) = errors.items().len_validation() {
                // Use Display trait for simple string message
                println!("  - items: {}", err);
                // Access individual fields
                println!(
                    "    (actual: {}, min: {}, max: {})",
                    err.actual.len(),
                    err.min,
                    err.max
                );
            }
        },
    }

    println!();

    // Example 2: UserProfile with multiple field errors
    let invalid_profile = UserProfile {
        username: "ab".into(), // Too short (min 3)
        bio: "x".repeat(600),  // Too long (max 500)
    };
    match invalid_profile.validate() {
        Ok(()) => println!("Profile is valid!"),
        Err(errors) => {
            println!("UserProfile validation failed:");
            if let Some(err) = errors.username().len_validation() {
                println!("  - username: {}", err);
            }
            if let Some(err) = errors.bio().len_validation() {
                println!("  - bio: {}", err);
            }
        },
    }

    println!();

    // Example 3: BlogPost with HashSet validation
    let invalid_post = BlogPost {
        title: "Hi".into(),                     // Too short (min 5)
        tags: std::collections::HashSet::new(), // Empty (min 1)
    };
    match invalid_post.validate() {
        Ok(()) => println!("Post is valid!"),
        Err(errors) => {
            println!("BlogPost validation failed:");
            if let Some(err) = errors.title().len_validation() {
                println!("  - title: {}", err);
            }
            if let Some(err) = errors.tags().len_validation() {
                println!("  - tags: {}", err);
            }
        },
    }

    println!();

    // Example 4: Product with multiple validators - using all() method
    let invalid_product = Product {
        sku: "ABC".into(), // Too short (min 5) AND doesn't start with "SKU-"
        price: 15000,      // Too high (max 10000)
    };
    match invalid_product.validate() {
        Ok(()) => println!("Product is valid!"),
        Err(errors) => {
            println!("Product validation failed:");

            // Access individual validators
            if let Some(err) = errors.price().range_validation() {
                println!("  - price: {}", err);
            }

            // Use all() to get all failed validators for a field
            // This is useful when a field has multiple validators
            let sku_errors = errors.sku().all();
            if !sku_errors.is_empty() {
                println!("  - sku ({} errors):", sku_errors.len());
                for err in &sku_errors {
                    // Using Display trait for readable error messages
                    println!("      {}", err);
                }
            }
        },
    }

    println!();
    println!("=== EsFluent-based Error Messages ===\n");

    let order = Order { items: vec![] };

    for lang in Languages::iter() {
        i18n::change_locale(lang).expect("Failed to change locale");

        println!(
            ">> Current Language: {:?} : {}",
            lang,
            lang.to_fluent_string()
        );

        match order.validate() {
            Ok(()) => println!("   Order is valid!"),
            Err(errors) => {
                if let Some(err) = errors.items().len_validation() {
                    // This prints in the language selected above
                    println!("   - items: {}", err.to_fluent_string());
                }
            },
        }
        println!();
    }

    // Example: Using all() with ToFluentString for multiple validators
    println!("=== Multiple Validators with Fluent (all() method) ===\n");

    let invalid_product = Product {
        sku: "ABC".into(),
        price: 15000,
    };

    for lang in Languages::iter() {
        i18n::change_locale(lang).expect("Failed to change locale");

        println!(">> Language: {:?} : {}", lang, lang.to_fluent_string());

        match invalid_product.validate() {
            Ok(()) => println!("   Product is valid!"),
            Err(errors) => {
                // Use all() to get all failed validators, then use to_fluent_string()
                // The auto-generated enum implements ToFluentString when fluent feature is enabled
                for err in errors.sku().all() {
                    println!("   - sku: {}", err.to_fluent_string());
                }
            },
        }
        println!();
    }

    println!("=== Valid Data Examples ===\n");

    let valid_order = Order {
        items: vec!["Apple".into(), "Banana".into(), "Orange".into()],
    };
    match valid_order.validate() {
        Ok(()) => println!("Order with {} items is valid!", valid_order.items.len()),
        Err(_) => println!("Unexpected validation failure"),
    }

    let valid_profile = UserProfile {
        username: "alice".into(),
        bio: "Hello, I'm Alice!".into(),
    };
    match valid_profile.validate() {
        Ok(()) => println!("Profile for '{}' is valid!", valid_profile.username),
        Err(_) => println!("Unexpected validation failure"),
    }

    let valid_post = BlogPost {
        title: "Rust Validation Made Easy".into(),
        tags: ["rust", "validation", "koruma"]
            .iter()
            .map(|s| s.to_string())
            .collect(),
    };
    match valid_post.validate() {
        Ok(()) => println!(
            "Post '{}' with {} tags is valid!",
            valid_post.title,
            valid_post.tags.len()
        ),
        Err(_) => println!("Unexpected validation failure"),
    }

    let valid_product = Product {
        sku: "SKU-12345".into(),
        price: 999,
    };
    match valid_product.validate() {
        Ok(()) => println!(
            "Product '{}' at ${} is valid!",
            valid_product.sku, valid_product.price
        ),
        Err(_) => println!("Unexpected validation failure"),
    }
}
