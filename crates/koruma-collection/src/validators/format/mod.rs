//! Format validation validators.
//!
//! This module contains validators for specific format validation scenarios
//! such as emails, URLs, phone numbers, credit cards, and IP addresses.

#[cfg(feature = "credit-card")]
mod credit_card;
#[cfg(feature = "email")]
mod email;
mod ip;
#[cfg(feature = "phone-number")]
mod phone_number;
#[cfg(feature = "url")]
mod url;

#[cfg(feature = "credit-card")]
pub use credit_card::CreditCardValidation;
#[cfg(feature = "email")]
pub use email::EmailValidation;
pub use ip::{IpKind, IpValidation};
#[cfg(feature = "phone-number")]
pub use phone_number::PhoneNumberValidation;
#[cfg(feature = "url")]
pub use url::UrlValidation;

#[cfg(feature = "internal-showcase")]
#[doc(hidden)]
#[inline(never)]
pub fn __link_showcase_validators() {
    #[cfg(feature = "credit-card")]
    credit_card::__koruma_showcase_anchor_credit_card_validation();

    #[cfg(feature = "email")]
    email::__koruma_showcase_anchor_email_validation();

    ip::__koruma_showcase_anchor_ip_validation();

    #[cfg(feature = "phone-number")]
    phone_number::__koruma_showcase_anchor_phone_number_validation();

    #[cfg(feature = "url")]
    url::__koruma_showcase_anchor_url_validation();
}
