#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub mod bon {
    #[doc(hidden)]
    pub use bon::__::IsUnset;
    #[doc(hidden)]
    pub use bon::*;
}

pub use koruma_core::{BuilderWithValue, Validate, ValidateExt, ValidationError};

#[cfg(feature = "derive")]
pub use koruma_derive::{Koruma, KorumaAllDisplay, validator};

#[cfg(all(feature = "derive", feature = "fluent"))]
pub use koruma_derive::KorumaAllFluent;

#[cfg(feature = "fluent")]
pub use es_fluent;

#[cfg(feature = "showcase")]
pub use koruma_core::showcase;

#[cfg(feature = "showcase")]
#[doc(hidden)]
pub use inventory;
