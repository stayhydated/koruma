use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

pub(crate) fn koruma_crate_path() -> TokenStream2 {
    koruma_crate_path_from_found(crate_name("koruma").ok())
}

fn koruma_crate_path_from_found(found: Option<FoundCrate>) -> TokenStream2 {
    match found {
        Some(FoundCrate::Itself) => quote! { crate },
        Some(FoundCrate::Name(name)) => {
            let ident = format_ident!("{name}");
            quote! { ::#ident }
        },
        None => quote! { ::koruma },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_path_rendering_covers_all_lookup_outcomes() {
        assert_eq!(
            koruma_crate_path_from_found(Some(FoundCrate::Itself)).to_string(),
            "crate"
        );
        assert_eq!(
            koruma_crate_path_from_found(Some(FoundCrate::Name("renamed_koruma".to_owned())))
                .to_string(),
            ":: renamed_koruma"
        );
        assert_eq!(koruma_crate_path_from_found(None).to_string(), ":: koruma");
    }
}
