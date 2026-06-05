use syn::{
    Attribute, Error, Ident, Result, Token,
    parse::{Parse, ParseStream},
    spanned::Spanned as _,
    token,
};

use super::SpannedValue;
use super::diagnostics::{KorumaAttrContext, context_error};
use super::keywords::KorumaKeyword;

/// Struct-level options parsed from `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub struct StructOptions {
    mode: StructMode,
    constructors: ConstructorOptions,
}

impl Default for StructOptions {
    fn default() -> Self {
        Self {
            mode: StructMode::Regular,
            constructors: ConstructorOptions::default(),
        }
    }
}

/// Normalized struct-level derive mode.
#[derive(Clone, Debug)]
pub enum StructMode {
    Regular,
    Newtype { marker: SpannedValue<Ident> },
}

/// Constructor integrations requested at the struct level.
#[derive(Clone, Debug, Default)]
pub struct ConstructorOptions {
    try_new: Option<SpannedValue<Ident>>,
    try_from: Option<SpannedValue<Ident>>,
}

impl ConstructorOptions {
    pub fn try_new(&self) -> bool {
        self.try_new.is_some()
    }

    pub fn try_from(&self) -> bool {
        self.try_from.is_some()
    }

    pub fn try_new_marker(&self) -> Option<&SpannedValue<Ident>> {
        self.try_new.as_ref()
    }

    pub fn try_from_marker(&self) -> Option<&SpannedValue<Ident>> {
        self.try_from.as_ref()
    }
}

impl StructOptions {
    pub fn mode(&self) -> &StructMode {
        &self.mode
    }

    pub fn constructors(&self) -> &ConstructorOptions {
        &self.constructors
    }
}

/// A single typed item inside struct-level `#[koruma(...)]`.
#[derive(Clone, Debug)]
pub enum StructKorumaItem {
    TryNew,
    TryFrom,
    Newtype,
}

#[derive(Clone, Debug)]
struct StructKorumaItemSource {
    kind: StructKorumaItemSourceKind,
    marker: SpannedValue<Ident>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StructKorumaItemSourceKind {
    TryNew,
    TryFrom,
    Newtype,
}

#[derive(Clone, Debug, Default)]
pub struct StructKorumaAttr {
    items: Vec<StructKorumaItem>,
    item_sources: Vec<StructKorumaItemSource>,
}

impl StructKorumaAttr {
    pub fn items(&self) -> &[StructKorumaItem] {
        &self.items
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
                    });
                },
                Some(KorumaKeyword::TryFrom) => {
                    if input.peek(token::Paren) || input.peek(Token![::]) {
                        return Err(Error::new(
                            ident.span(),
                            format!(
                                "`{ident}` is only valid as a bare struct-level `#[koruma(...)]` option; expected {}",
                                KorumaAttrContext::Struct.accepted_items()
                            ),
                        ));
                    }
                    attr.items.push(StructKorumaItem::TryFrom);
                    attr.item_sources.push(StructKorumaItemSource {
                        kind: StructKorumaItemSourceKind::TryFrom,
                        marker: SpannedValue::new(ident.clone(), ident.span()),
                    });
                },
                Some(KorumaKeyword::Newtype) => {
                    if input.peek(syn::token::Paren) {
                        return Err(Error::new(
                            ident.span(),
                            "parenthesized `newtype` options are unsupported; use #[koruma(newtype, try_from)]",
                        ));
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
                    });
                    attr.items.push(StructKorumaItem::Newtype);
                },
                keyword => {
                    if matches!(
                        keyword,
                        Some(
                            KorumaKeyword::Skip
                                | KorumaKeyword::Nested
                                | KorumaKeyword::Each
                                | KorumaKeyword::Value
                                | KorumaKeyword::SkipCapture
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
                            "unknown struct-level koruma option: `{}`. Expected `try_new`, `try_from`, or `newtype`",
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
        let mut try_new_marker: Option<SpannedValue<Ident>> = None;
        let mut newtype_marker: Option<SpannedValue<Ident>> = None;
        let mut try_from_marker: Option<SpannedValue<Ident>> = None;

        for item in attr.item_sources {
            match item.kind {
                StructKorumaItemSourceKind::TryNew => {
                    if try_new_marker.is_some() {
                        return Err(Error::new(
                            item.marker.span,
                            "duplicate struct-level koruma option `try_new`",
                        ));
                    }
                    try_new_marker = Some(item.marker);
                },
                StructKorumaItemSourceKind::TryFrom => {
                    if try_from_marker.is_some() {
                        return Err(Error::new(
                            item.marker.span,
                            "duplicate struct-level koruma option `try_from`",
                        ));
                    }
                    try_from_marker = Some(item.marker);
                },
                StructKorumaItemSourceKind::Newtype => {
                    if newtype_marker.is_some() {
                        return Err(Error::new(
                            item.marker.span,
                            "duplicate struct-level koruma option `newtype`",
                        ));
                    }
                    newtype_marker = Some(item.marker);
                },
            }
        }

        let mode = if let Some(marker) = newtype_marker {
            StructMode::Newtype { marker }
        } else {
            StructMode::Regular
        };
        let constructors = ConstructorOptions {
            try_new: try_new_marker,
            try_from: try_from_marker,
        };

        Ok(StructOptions { mode, constructors })
    }
}

/// Parse struct-level `#[koruma(...)]` attributes from a list of attributes.
///
/// Returns `StructOptions::default()` if no `#[koruma(...)]` attribute is found.
pub fn parse_struct_options(attrs: &[Attribute]) -> Result<StructOptions> {
    let koruma_attrs = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("koruma"))
        .collect::<Vec<_>>();
    match koruma_attrs.as_slice() {
        [] => Ok(StructOptions::default()),
        [attr] => attr.parse_args::<StructOptions>(),
        [_, duplicate, ..] => Err(Error::new(
            duplicate.path().span(),
            "only one struct-level `#[koruma(...)]` attribute is allowed; combine options in a single attribute",
        )),
    }
}
