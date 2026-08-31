//! Snapshot tests for the expand module.
//!
//! These tests verify the generated TokenStream output using insta snapshots.

use crate::expand::*;

use proc_macro2::TokenStream as TokenStream2;
use syn::{DeriveInput, ItemStruct};

macro_rules! assert_snapshot {
    ($($argument:tt)*) => {
        insta::with_settings!({ snapshot_path => "../snapshots" }, {
            insta::assert_snapshot!($($argument)*);
        })
    };
}

/// Helper to format TokenStream as pretty-printed Rust code
fn pretty_print(tokens: TokenStream2) -> String {
    let file = syn::parse_file(&tokens.to_string()).unwrap();
    prettyplease::unparse(&file)
}

fn compact_ws(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

include!("snapshot_tests/validator.rs");
include!("snapshot_tests/koruma.rs");
include!("snapshot_tests/display_fluent.rs");
include!("snapshot_tests/optional_collection.rs");
include!("snapshot_tests/borrowed_generic.rs");
include!("snapshot_tests/newtype.rs");
include!("snapshot_tests/constructors.rs");
