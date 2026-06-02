use koruma_derive::validator;

#[validator]
enum NotAStruct {
    Value,
}

#[validator]
fn not_a_struct() {}

fn main() {}
