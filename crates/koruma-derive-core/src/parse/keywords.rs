use syn::Ident;

syn::custom_keyword!(capture);
syn::custom_keyword!(default);
syn::custom_keyword!(each);
syn::custom_keyword!(full);
syn::custom_keyword!(into);
syn::custom_keyword!(name);
syn::custom_keyword!(nested);
syn::custom_keyword!(newtype);
syn::custom_keyword!(required);
syn::custom_keyword!(setter);
syn::custom_keyword!(skip);
syn::custom_keyword!(try_from);
syn::custom_keyword!(try_new);
syn::custom_keyword!(unwrapped);
syn::custom_keyword!(value);

/// Reserved Koruma attribute words shared across context-specific parsers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KorumaKeyword {
    Capture,
    Default,
    Each,
    Full,
    Into,
    Name,
    Nested,
    Newtype,
    Required,
    Setter,
    Skip,
    TryFrom,
    TryNew,
    Unwrapped,
    Value,
}

impl KorumaKeyword {
    pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "capture" => Some(Self::Capture),
            "default" => Some(Self::Default),
            "each" => Some(Self::Each),
            "full" => Some(Self::Full),
            "into" => Some(Self::Into),
            "name" => Some(Self::Name),
            "nested" => Some(Self::Nested),
            "newtype" => Some(Self::Newtype),
            "required" => Some(Self::Required),
            "setter" => Some(Self::Setter),
            "skip" => Some(Self::Skip),
            "try_from" => Some(Self::TryFrom),
            "try_new" => Some(Self::TryNew),
            "unwrapped" => Some(Self::Unwrapped),
            "value" => Some(Self::Value),
            _ => None,
        }
    }
}

/// Methods generated or called by Koruma internally when rendering direct
/// validator chains. User-authored chains must stop before these methods.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReservedBuilderMethod {
    Builder,
    Build,
    WithValue,
    CaptureValueRef,
}

impl ReservedBuilderMethod {
    pub(super) fn from_ident(ident: &Ident) -> Option<Self> {
        match ident.to_string().as_str() {
            "builder" => Some(Self::Builder),
            "build" => Some(Self::Build),
            "with_value" => Some(Self::WithValue),
            "capture_value_ref" => Some(Self::CaptureValueRef),
            _ => None,
        }
    }

    pub(super) fn is_builder(self) -> bool {
        matches!(self, Self::Builder)
    }
}
