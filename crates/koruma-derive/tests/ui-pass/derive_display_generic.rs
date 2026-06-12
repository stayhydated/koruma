use koruma_derive::{validator, Koruma, KorumaAllDisplay};
use renamed_koruma::Validate;
use std::fmt;

#[validator]
#[derive(Debug)]
pub struct MinValidation<T> {
    min: T,
    actual: T,
}

impl<T> Validate<T> for MinValidation<T>
where
    T: Clone + PartialOrd,
{
    fn validate(&self, value: &T) -> bool {
        *value >= self.min
    }
}

impl<T> fmt::Display for MinValidation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("too small")
    }
}

#[derive(Koruma, KorumaAllDisplay)]
struct Item<T>
where
    T: Clone + Default + PartialOrd,
{
    #[koruma(MinValidation::<_>::min(T::default()))]
    value: T,
}

fn main() {
    let _ = Item { value: 3 }.validate();
}
