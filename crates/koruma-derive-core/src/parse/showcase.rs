use syn::{
    Error, Ident, ItemStruct, Result, Token,
    parse::{Parse, ParseStream},
};

/// Parsed showcase attribute:
/// `#[showcase(name = "...", description = "...", create = |input| { ... }, input_type = Text)]`
///
/// The `create` closure takes a `&str` and returns the validator instance.
/// Required `input_type` must be `Text` or `Numeric`.
/// Optional `module` can be "string", "format", "numeric", "collection", or "general".
/// `module` defaults to `general` when omitted.

#[cfg(feature = "internal-showcase")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShowcaseInputType {
    /// Showcases that expect text input.
    Text,
    /// Showcases that expect numeric input.
    Numeric,
}

/// Parsed and validated `module` selector for showcase validators.
#[cfg(feature = "internal-showcase")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShowcaseModule {
    String,
    Format,
    Numeric,
    Collection,
    General,
}

#[cfg(feature = "internal-showcase")]
#[derive(Clone, Debug)]
pub struct ShowcaseAttr {
    pub name: syn::LitStr,
    pub description: syn::LitStr,
    pub create: syn::ExprClosure,
    pub input_type: ShowcaseInputType,
    pub module: Option<ShowcaseModule>,
}

#[cfg(feature = "internal-showcase")]
impl Parse for ShowcaseAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<syn::LitStr> = None;
        let mut description: Option<syn::LitStr> = None;
        let mut create: Option<syn::ExprClosure> = None;
        let mut input_type: Option<Ident> = None;
        let mut module: Option<ShowcaseModule> = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "name" => {
                    name = Some(input.parse()?);
                },
                "description" => {
                    description = Some(input.parse()?);
                },
                "create" => {
                    create = Some(input.parse()?);
                },
                "input_type" => {
                    input_type = Some(input.parse()?);
                },
                "module" => {
                    let parsed_module: syn::LitStr = input.parse()?;
                    module = Some(parse_showcase_module(parsed_module)?);
                },
                other => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unknown showcase attribute: {}", other),
                    ));
                },
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(ShowcaseAttr {
            name: name
                .ok_or_else(|| Error::new(input.span(), "showcase requires `name` attribute"))?,
            description: description.ok_or_else(|| {
                Error::new(input.span(), "showcase requires `description` attribute")
            })?,
            create: create
                .ok_or_else(|| Error::new(input.span(), "showcase requires `create` attribute"))?,
            input_type: match input_type {
                Some(input_type) => parse_showcase_input_type(input_type)?,
                None => {
                    return Err(Error::new(
                        input.span(),
                        "showcase requires `input_type` attribute",
                    ));
                },
            },
            module,
        })
    }
}

#[cfg(feature = "internal-showcase")]
fn parse_showcase_input_type(input_type: Ident) -> Result<ShowcaseInputType> {
    match input_type.to_string().as_str() {
        "Text" => Ok(ShowcaseInputType::Text),
        "Numeric" => Ok(ShowcaseInputType::Numeric),
        _ => Err(Error::new(
            input_type.span(),
            "showcase `input_type` must be `Text` or `Numeric`",
        )),
    }
}

#[cfg(feature = "internal-showcase")]
fn parse_showcase_module(module: syn::LitStr) -> Result<ShowcaseModule> {
    match module.value().as_str() {
        "string" => Ok(ShowcaseModule::String),
        "format" => Ok(ShowcaseModule::Format),
        "numeric" => Ok(ShowcaseModule::Numeric),
        "collection" => Ok(ShowcaseModule::Collection),
        "general" => Ok(ShowcaseModule::General),
        _ => Err(Error::new(
            module.span(),
            "showcase `module` must be one of: \"string\", \"format\", \"numeric\", \"collection\", or \"general\"",
        )),
    }
}

/// Find and parse showcase attribute from struct
#[cfg(feature = "internal-showcase")]
pub fn find_showcase_attr(input: &ItemStruct) -> Result<Option<ShowcaseAttr>> {
    for attr in &input.attrs {
        if attr.path().is_ident("showcase") {
            return Ok(Some(attr.parse_args::<ShowcaseAttr>()?));
        }
    }
    Ok(None)
}
