#[test]
fn test_koruma_expansion_struct_newtype_optional_nested_no_deref_impl() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct OptionalNestedWrapper {
            #[koruma(nested)]
            pub inner: Option<InnerValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let rendered = pretty_print(expanded);
    let compact = compact_ws(&rendered);
    assert!(compact.contains("NewtypeValidationforOptionalNestedWrapper{}"));
    assert!(!compact.contains("implcore::ops::DerefforOptionalNestedWrapperKorumaValidationError"));
}

#[test]
fn test_koruma_expansion_newtype_optional_without_field_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct OptionalNewtypeField {
            #[koruma(newtype)]
            pub wrapped: Option<WrappedValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("ifletSome(ref__newtype_value)=self.wrapped"));
    assert!(compact.contains("inner:Option<<WrappedValueas::renamed_koruma::ValidateExt>::Error>"));
    assert!(compact.contains("self.wrapped.inner.as_ref()"));
    assert!(
        !compact.contains("implstd::ops::DerefforOptionalNewtypeFieldWrappedKorumaValidationError")
    );
}

#[test]
fn test_koruma_expansion_newtype_with_full_and_unwrapped_validators() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct RichNewtypeField {
            #[koruma(newtype, RequiredValidation::<Option<_>>, GenericRange::<_>.min(0).max(10), PlainValidation.min(1))]
            pub wrapped: Option<WrappedValue>,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("::renamed_koruma::Validate<Option<WrappedValue>,>"));
    assert!(compact.contains("::validate(&validator,&self.wrapped)"));
    assert!(compact.contains("__private::CaptureValueRef::capture_value_ref("));
    assert!(compact.contains("PlainValidation::min(1)"));
    assert!(compact.contains("inner:Option<<WrappedValueas::renamed_koruma::ValidateExt>::Error>"));
    assert!(compact.contains("self.inner.as_ref()"));
    assert!(compact.contains("error.wrapped.inner=Some(newtype_err);"));
    assert!(compact.contains("inner:None"));
}

#[test]
fn test_koruma_expansion_newtype_non_optional_with_validators_uses_direct_inner_validation() {
    let input: DeriveInput = syn::parse_quote! {
        pub struct DirectNewtypeField {
            #[koruma(newtype, GenericRange::<_>.min(0).max(10), PlainValidation.min(1))]
            pub wrapped: WrappedValue,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(!compact.contains("ifletSome(ref__newtype_value)=self.wrapped"));
    assert!(compact.contains("let__newtype_value=&self.wrapped;"));
    assert!(compact.contains("ifletErr(newtype_err)=__newtype_value.validate()"));
}


#[test]
fn test_koruma_expansion_struct_newtype_nested_deref_has_no_expect() {
    let input: DeriveInput = syn::parse_quote! {
        #[koruma(newtype)]
        pub struct NestedWrapper {
            #[koruma(nested)]
            pub inner: InnerValue,
        }
    };

    let expanded = expand_koruma(input).unwrap();
    let compact = compact_ws(&pretty_print(expanded));
    assert!(compact.contains("implcore::ops::DerefforNestedWrapperKorumaValidationError"));
    assert!(compact.contains("fnderef(&self)->&Self::Target{&self.inner}"));
    assert!(!compact.contains("expect(\"newtypeerrorshouldhaveinnererror\")"));
}
