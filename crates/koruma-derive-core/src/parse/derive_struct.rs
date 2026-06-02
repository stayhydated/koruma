use syn::{
    Attribute, Error, Ident, Result, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token,
};
use syn_cfg_attr::AttributeHelpers;

use super::keywords::KorumaKeyword;
use super::{KorumaAttrContext, SpannedValue, context_error};

/// Struct-level options parsed from `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub struct StructOptions {
    /// Constructor integrations requested at the struct level.
    constructor: StructConstructor,
    /// Whether struct-level newtype behavior is enabled.
    newtype: bool,
    try_new_marker: Option<SpannedValue<Ident>>,
    newtype_marker: Option<SpannedValue<Ident>>,
    try_from_marker: Option<SpannedValue<Ident>>,
}

impl Default for StructOptions {
    fn default() -> Self {
        Self {
            constructor: StructConstructor::None,
            newtype: false,
            try_new_marker: None,
            newtype_marker: None,
            try_from_marker: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructConstructor {
    None,
    TryNew,
    TryFrom,
    TryNewAndTryFrom,
}

impl StructConstructor {
    fn with_try_new(self) -> Self {
        match self {
            Self::None => Self::TryNew,
            Self::TryNew | Self::TryNewAndTryFrom => self,
            Self::TryFrom => Self::TryNewAndTryFrom,
        }
    }

    fn with_try_from(self) -> Self {
        match self {
            Self::None => Self::TryFrom,
            Self::TryFrom | Self::TryNewAndTryFrom => self,
            Self::TryNew => Self::TryNewAndTryFrom,
        }
    }

    pub fn try_new(self) -> bool {
        matches!(self, Self::TryNew | Self::TryNewAndTryFrom)
    }

    pub fn try_from(self) -> bool {
        matches!(self, Self::TryFrom | Self::TryNewAndTryFrom)
    }
}

impl StructOptions {
    pub fn constructor(&self) -> StructConstructor {
        self.constructor
    }

    pub fn is_newtype(&self) -> bool {
        self.newtype
    }

    pub fn try_new(&self) -> bool {
        self.constructor.try_new()
    }

    pub fn try_from(&self) -> bool {
        self.constructor.try_from()
    }
}

/// A single typed item inside struct-level `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub enum StructKorumaItem {
    TryNew,
    Newtype(StructNewtypeOptions),
}

#[derive(Clone, Debug)]
struct StructKorumaItemSource {
    kind: StructKorumaItemSourceKind,
    marker: SpannedValue<Ident>,
    try_from_marker: Option<SpannedValue<Ident>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StructKorumaItemSourceKind {
    TryNew,
    Newtype,
}

/// Options for the struct-level `newtype(...)` attribute.
#[derive(Clone, Debug, Default)]
pub struct StructNewtypeOptions {
    pub try_from: bool,
    try_from_marker: Option<SpannedValue<Ident>>,
}

#[derive(Clone, Debug, Default)]
pub struct StructKorumaAttr {
    pub items: Vec<StructKorumaItem>,
    item_sources: Vec<StructKorumaItemSource>,
}

impl Parse for StructNewtypeOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut options = StructNewtypeOptions::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match KorumaKeyword::from_ident(&ident) {
                Some(KorumaKeyword::TryFrom) => {
                    if options.try_from_marker.is_some() {
                        return Err(Error::new(
                            ident.span(),
                            "duplicate `newtype(try_from)` option",
                        ));
                    }
                    options.try_from = true;
                    options.try_from_marker = Some(SpannedValue::new(ident.clone(), ident.span()));
                },
                _ => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unknown newtype option: `{}`. Expected `try_from`", ident),
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(options)
    }
}

impl Parse for StructKorumaAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attr = StructKorumaAttr::default();

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            match KorumaKeyword::from_ident(&ident) {
                Some(KorumaKeyword::TryNew) => {
                    if input.peek(token::Paren) || input.peek(Token![::]) {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "`{ident}` is only valid as a bare struct-level `#[koruma(...)]` option; expected {}",
                                KorumaAttrContext::Struct.accepted_items()
                            ),
                        ));
                    }
                    attr.items.push(StructKorumaItem::TryNew);
                    attr.item_sources.push(StructKorumaItemSource {
                        kind: StructKorumaItemSourceKind::TryNew,
                        marker: SpannedValue::new(ident.clone(), ident.span()),
                        try_from_marker: None,
                    });
                },
                Some(KorumaKeyword::Newtype) => {
                    let mut newtype_options = StructNewtypeOptions::default();
                    if input.peek(syn::token::Paren) {
                        let content;
                        syn::parenthesized!(content in input);
                        newtype_options = content.parse()?;
                    } else if input.peek(Token![::]) {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "`{ident}` is a reserved koruma struct option; use a different validator path in a data-field `#[koruma(...)]` attribute"
                            ),
                        ));
                    }
                    attr.item_sources.push(StructKorumaItemSource {
                        kind: StructKorumaItemSourceKind::Newtype,
                        marker: SpannedValue::new(ident.clone(), ident.span()),
                        try_from_marker: newtype_options.try_from_marker.clone(),
                    });
                    attr.items.push(StructKorumaItem::Newtype(newtype_options));
                },
                keyword => {
                    if matches!(
                        keyword,
                        Some(
                            KorumaKeyword::Skip
                                | KorumaKeyword::Nested
                                | KorumaKeyword::Each
                                | KorumaKeyword::Value
                                | KorumaKeyword::Setter
                                | KorumaKeyword::Full
                                | KorumaKeyword::Unwrapped
                        )
                    ) {
                        return Err(context_error(&ident, KorumaAttrContext::Struct));
                    }
                    return Err(Error::new(
                        ident.span(),
                        format!(
                            "unknown struct-level koruma option: `{}`. Expected `try_new` or `newtype`",
                            ident
                        ),
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(attr)
    }
}

impl Parse for StructOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let attr: StructKorumaAttr = input.parse()?;
        let mut options = StructOptions::default();

        for item in attr.item_sources {
            match item.kind {
                StructKorumaItemSourceKind::TryNew => {
                    if options.try_new_marker.is_some() {
                        return Err(Error::new(
                            item.marker.span,
                            "duplicate struct-level koruma option `try_new`",
                        ));
                    }
                    options.constructor = options.constructor.with_try_new();
                    options.try_new_marker = Some(item.marker);
                },
                StructKorumaItemSourceKind::Newtype => {
                    if options.newtype_marker.is_some() {
                        return Err(Error::new(
                            item.marker.span,
                            "duplicate struct-level koruma option `newtype`",
                        ));
                    }
                    options.newtype = true;
                    options.newtype_marker = Some(item.marker);
                    if let Some(try_from_marker) = item.try_from_marker {
                        options.constructor = options.constructor.with_try_from();
                        options.try_from_marker = Some(try_from_marker);
                    }
                },
            }
        }

        Ok(options)
    }
}

/// Parse struct-level `#[koruma(...)]` attributes from a list of attributes.
///
/// Returns `StructOptions::default()` if no `#[koruma(...)]` attribute is found.
pub fn parse_struct_options(attrs: &[Attribute]) -> Result<StructOptions> {
    let attrs = attrs.to_vec();
    let koruma_attrs = attrs.find_attribute("koruma");
    match koruma_attrs.as_slice() {
        [] => Ok(StructOptions::default()),
        [attr] => attr.parse_args::<StructOptions>(),
        [_, duplicate, ..] => Err(Error::new(
            duplicate.path().span(),
            "only one struct-level `#[koruma(...)]` attribute is allowed; combine options in a single attribute",
        )),
    }
}
