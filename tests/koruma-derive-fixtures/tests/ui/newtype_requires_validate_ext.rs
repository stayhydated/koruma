use koruma_derive::Koruma;

struct Child(i32);

#[derive(Koruma)]
struct Parent {
    #[koruma(newtype)]
    child: Child,
}

fn main() {}
