use koruma_derive::Koruma;

#[derive(Koruma)]
#[koruma(newtype)]
struct Empty;

#[derive(Koruma)]
#[koruma(newtype, try_from)]
struct TooMany {
    first: String,
    second: String,
}

fn main() {}
