#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod bon {
    #[doc(hidden)]
    pub use bon::__::IsUnset;
    #[doc(hidden)]
    pub use bon::*;
}

pub use koruma_core::{BuilderWithValue, KorumaResult, Validate, ValidationError};

#[cfg(feature = "derive")]
pub use koruma_derive::{Koruma, KorumaAllDisplay, validator};

#[cfg(all(feature = "derive", feature = "fluent"))]
pub use koruma_derive::KorumaAllFluent;

#[cfg(feature = "fluent")]
pub use es_fluent;
