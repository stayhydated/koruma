fn main() {
    #[cfg(any(
        feature = "facade-minimal",
        feature = "facade-default",
        feature = "facade-fluent"
    ))]
    {
        struct Nonzero;
        impl koruma::Validate<u8> for Nonzero {
            fn validate(&self, value: &u8) -> bool {
                *value != 0
            }
        }
        assert!(koruma::Validate::validate(&Nonzero, &1));
        assert!(!koruma::Validate::validate(&Nonzero, &0));
    }
    #[cfg(feature = "facade-default")]
    {
        #[derive(koruma::Koruma)]
        struct Input {
            value: u8,
        }
        assert!(Input { value: 1 }.validate().is_ok());
    }
    #[cfg(any(feature = "core-minimal", feature = "core-fluent"))]
    {
        let descriptor = koruma_core::ValidatorDescriptor::new("consumer", &[]);
        assert_eq!(descriptor.type_name(), "consumer");
        assert!(descriptor.params().is_empty());
    }
    #[cfg(any(
        feature = "collection-minimal",
        feature = "collection-default",
        feature = "collection-selected",
        feature = "collection-fluent"
    ))]
    {
        let rule = koruma_collection::numeric::RangeValidation::min(1_u8)
            .max(3_u8)
            .build();
        assert!(koruma::Validate::validate(&rule, &2));
    }
    #[cfg(feature = "collection-default")]
    {
        let rule = koruma_collection::numeric::RangeValidation::min(1_u8)
            .max(3_u8)
            .build();
        assert!(!rule.to_string().is_empty());
    }
    #[cfg(feature = "collection-selected")]
    {
        let _email =
            koruma_collection::format::EmailValidation::with_value("user@example.com".to_owned())
                .build();
        let one = rust_decimal::Decimal::ONE;
        assert_eq!(
            <rust_decimal::Decimal as koruma_collection::numeric::Numeric>::zero(),
            rust_decimal::Decimal::ZERO
        );
        let _range = koruma_collection::numeric::RangeValidation::min(one)
            .max(one)
            .build();
    }
}
