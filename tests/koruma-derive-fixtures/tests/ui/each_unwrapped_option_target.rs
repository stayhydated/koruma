use koruma_derive::Koruma;

#[derive(Koruma)]
struct Demo {
    #[koruma(each(unwrapped(GenericValidation::<Option<_>>)))]
    values: Vec<Option<i32>>,
}

fn main() {}
