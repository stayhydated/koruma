use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

pub(crate) fn koruma_crate_path() -> TokenStream2 {
    match crate_name("koruma") {
        Ok(FoundCrate::Itself) => quote! { crate },
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{name}");
            quote! { ::#ident }
        },
        Err(_) => quote! { ::koruma },
    }
}
