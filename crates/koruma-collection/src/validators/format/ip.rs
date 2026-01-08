//! IP address validation for koruma.
//!
//! This module provides:
//! - `IpValidation` validator to check if a string is a valid IP address
//!
//! # Example
//! ```ignore
//! use koruma::Koruma;
//! use koruma_collection::validators::ip::IpValidation;
//!
//! #[derive(Koruma)]
//! struct NetworkConfig {
//!     #[koruma(IpValidation<_>(kind = IpKind::V4))]
//!     ip_address: String,
//! }
//! ```

use koruma::{Validate, validator};

/// The type of IP address to validate
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub enum IpKind {
    Any,
    V4,
    V6,
}

impl std::fmt::Display for IpKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpKind::Any => write!(f, "IP"),
            IpKind::V4 => write!(f, "IPv4"),
            IpKind::V6 => write!(f, "IPv6"),
        }
    }
}

/// Validates that a string is a valid IP address.
#[validator]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "fluent", derive(es_fluent::EsFluent))]
pub struct IpValidation<T: AsRef<str>> {
    /// The type of IP address to validate
    pub kind: IpKind,
    /// The string being validated (stored for error context)
    #[koruma(value)]
    #[cfg_attr(feature = "fluent", fluent(value(|x: &T| x.as_ref().to_string())))]
    pub actual: T,
}

impl<T: AsRef<str>> Validate<T> for IpValidation<T> {
    fn validate(&self, value: &T) -> bool {
        let s = value.as_ref();
        match self.kind {
            IpKind::Any => s.parse::<std::net::IpAddr>().is_ok(),
            IpKind::V4 => s.parse::<std::net::Ipv4Addr>().is_ok(),
            IpKind::V6 => s.parse::<std::net::Ipv6Addr>().is_ok(),
        }
    }
}

#[cfg(feature = "fmt")]
impl<T: AsRef<str>> std::fmt::Display for IpValidation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a valid {} address", self.kind)
    }
}
