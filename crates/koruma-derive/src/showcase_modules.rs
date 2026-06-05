#[cfg(feature = "internal-showcase")]
use heck::ToUpperCamelCase as _;
#[cfg(feature = "internal-showcase")]
use proc_macro2::{Span, TokenStream as TokenStream2};
#[cfg(feature = "internal-showcase")]
use quote::{format_ident, quote};
#[cfg(feature = "internal-showcase")]
use syn::{
    Error, Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
};

#[cfg(feature = "internal-showcase")]
struct ShowcaseModuleList {
    modules: Punctuated<Ident, Token![,]>,
}

#[cfg(feature = "internal-showcase")]
impl Parse for ShowcaseModuleList {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            modules: Punctuated::parse_terminated(input)?,
        })
    }
}

#[cfg(feature = "internal-showcase")]
fn parse_modules(input: TokenStream2) -> Result<Vec<Ident>> {
    let list: ShowcaseModuleList = syn::parse2(input)?;

    if list.modules.is_empty() {
        return Err(Error::new(
            Span::call_site(),
            "showcase_modules requires at least one module",
        ));
    }

    Ok(list.modules.into_iter().collect())
}

#[cfg(feature = "internal-showcase")]
fn module_variant_ident(module: &Ident) -> Ident {
    format_ident!("{}", module.to_string().to_upper_camel_case())
}

#[cfg(feature = "internal-showcase")]
fn module_str_literal(module: &Ident) -> LitStr {
    LitStr::new(&module.to_string(), module.span())
}

#[cfg(feature = "internal-showcase")]
fn expand_showcase_modules(modules: &[Ident]) -> Result<TokenStream2> {
    let link_calls = modules.iter().map(|module| {
        quote! { #module::__link_showcase_validators(); }
    });

    Ok(quote! {
        #[cfg(feature = "internal-showcase")]
        #[doc(hidden)]
        #[inline(never)]
        pub fn __link_showcase_validators() {
            #( #link_calls )*
        }
    })
}

#[cfg(feature = "internal-showcase")]
fn expand_showcase_module_enum(modules: &[Ident]) -> Result<TokenStream2> {
    let variants: Vec<Ident> = modules.iter().map(module_variant_ident).collect();
    let variant_count = variants.len();
    let variant_docs: Vec<syn::LitStr> = modules
        .iter()
        .map(|module| {
            LitStr::new(
                &format!("Validators in the `{}` module.", module),
                module.span(),
            )
        })
        .collect();
    let all_variants = variants.iter().map(|variant| quote! { Self::#variant });
    let as_str_arms = modules.iter().zip(&variants).map(|(module, variant)| {
        let module_str = module_str_literal(module);
        quote! { Self::#variant => #module_str }
    });

    Ok(quote! {
        /// The category/module for a showcase validator.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum ValidatorModule {
            #(
                #[doc = #variant_docs]
                #variants
            ),*
        }

        impl ValidatorModule {
            pub const ALL: [Self; #variant_count] = [ #( #all_variants ),* ];

            pub const fn as_str(self) -> &'static str {
                match self {
                    #( #as_str_arms, )*
                }
            }
        }
    })
}

#[cfg(feature = "internal-showcase")]
pub fn expand_showcase_modules_macro(input: TokenStream2) -> Result<TokenStream2> {
    expand_showcase_modules(&parse_modules(input)?)
}

#[cfg(feature = "internal-showcase")]
pub fn expand_showcase_module_enum_macro(input: TokenStream2) -> Result<TokenStream2> {
    expand_showcase_module_enum(&parse_modules(input)?)
}
