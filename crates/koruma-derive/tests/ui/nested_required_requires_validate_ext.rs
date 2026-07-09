use koruma_derive::Koruma;

struct Child {
    value: i32,
}

#[derive(Koruma)]
struct Parent {
    #[koruma(nested)]
    child: Child,
}

fn main() {}
