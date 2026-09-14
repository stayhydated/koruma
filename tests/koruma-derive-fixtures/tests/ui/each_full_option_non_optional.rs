use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(each(GenericValidation::<Option<_>>))]
    values: Vec<i32>,
}

fn main() {}
