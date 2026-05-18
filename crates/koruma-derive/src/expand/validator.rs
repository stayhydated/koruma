#[cfg(feature = "internal-showcase")]
use super::{ShowcaseInputType, ShowcaseModule};
use heck::{ToSnakeCase, ToUpperCamelCase};
#[cfg(feature = "internal-showcase")]
use koruma_derive_core::find_showcase_attr;
use koruma_derive_core::{ValueFieldCapture, find_value_field_info_strict, option_inner_type};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Field, Fields, GenericParam, Ident, ItemStruct, Token, Type, Visibility,
    parenthesized, parse_quote,
};

/// Core expansion logic for the `#[validator]` attribute macro.
///
/// Takes a parsed struct and returns the expanded TokenStream.
pub fn expand_validator(mut input: ItemStruct) -> Result<TokenStream2, syn::Error> {
    let struct_name = &input.ident;
    let builder_name = format_ident!("{}Builder", struct_name);

    if !matches!(input.fields, Fields::Named(_)) {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "koruma::validator only supports structs with named fields",
        ));
    }

    // Parse showcase attribute if present (only when feature enabled)
    #[cfg(feature = "internal-showcase")]
    let showcase_attr = find_showcase_attr(&input)?;

    // Find the field marked with #[koruma(value)]
    let value_field = find_value_field_info_strict(&input)?.ok_or_else(|| {
        syn::Error::new_spanned(
            &input,
            "koruma::validator requires a field marked with #[koruma(value)].\n\
             Example:\n\
             #[koruma(value)]\n\
             actual: Option<i32>",
        )
    })?;
    let value_field_name = value_field.name;
    let value_field_type = value_field.ty;
    let value_field_capture = value_field.capture;

    // Extract the inner type from Option<T>
    let inner_type = option_inner_type(&value_field_type).unwrap_or(&value_field_type);

    if value_field_capture == ValueFieldCapture::Skip
        && option_inner_type(&value_field_type).is_none()
    {
        return Err(syn::Error::new_spanned(
            &value_field_type,
            "`#[koruma(value, skip_capture)]` currently requires an `Option<T>` field",
        ));
    }

    // Add #[derive(bon::Builder)] to the existing attributes
    let builder_attr: syn::Attribute = parse_quote!(#[derive(koruma::bon::Builder)]);
    input.attrs.insert(0, builder_attr);

    // Tell bon to use koruma's re-exported bon path, so downstream crates don't
    // need a direct `bon` dependency. Keep Bon's start function private; koruma
    // exposes direct validator entrypoints instead.
    let bon_crate_attr: syn::Attribute = parse_quote!(
        #[builder(crate = ::koruma::bon, start_fn(name = __koruma_bon_builder, vis = ""))]
    );
    input.attrs.insert(1, bon_crate_attr);

    // Remove #[koruma(value)] and #[showcase(...)] from attributes
    input.attrs.retain(|attr| !attr.path().is_ident("showcase"));

    // Remove #[koruma(value)] from the field so bon doesn't see it
    let Fields::Named(ref mut fields) = input.fields else {
        return Err(syn::Error::new_spanned(
            &input.fields,
            "koruma::validator only supports structs with named fields",
        ));
    };
    for field in &mut fields.named {
        if field.ident.as_ref() == Some(&value_field_name)
            && !matches!(field.vis, Visibility::Inherited)
        {
            return Err(syn::Error::new_spanned(
                &field.vis,
                format!(
                    "`#[koruma(value)]` field `{}` must be private; use the generated getter instead",
                    value_field_name
                ),
            ));
        }

        if field.ident.as_ref() == Some(&value_field_name) {
            field.attrs.retain(|attr| !attr.path().is_ident("koruma"));
        }
    }

    // Generate the module name that bon creates (snake_case of struct name + _builder)
    let module_name = format_ident!("{}_builder", struct_name.to_string().to_snake_case());

    // Generate the associated type name (PascalCase of field name) and Set wrapper
    let value_pascal = value_field_name.to_string().to_upper_camel_case();
    let value_assoc_type = format_ident!("{}", value_pascal);
    let set_value_type = format_ident!("Set{}", value_pascal);
    let value_field_name_str = value_field_name.to_string();
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let builder_generic_args: Vec<_> = input
        .generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                quote! { #lifetime }
            },
            GenericParam::Type(param) => {
                let ident = &param.ident;
                quote! { #ident }
            },
            GenericParam::Const(param) => {
                let ident = &param.ident;
                quote! { #ident }
            },
        })
        .collect();

    let initial_builder_ty =
        quote! { #builder_name<#(#builder_generic_args,)* #module_name::Empty> };
    let output_builder_ty =
        quote! { #builder_name<#(#builder_generic_args,)* #module_name::#set_value_type<S>> };
    let initial_value_builder_ty = quote! { #builder_name<#(#builder_generic_args,)* #module_name::#set_value_type<#module_name::Empty>> };

    let direct_builder_methods = direct_builder_methods(
        &input,
        &value_field_name,
        &builder_name,
        &module_name,
        &builder_generic_args,
    )?;

    let value_getter_impl = quote! {
        impl #impl_generics #struct_name #type_generics #where_clause {
            #[doc(hidden)]
            pub fn __koruma_builder() -> #initial_builder_ty {
                Self::__koruma_bon_builder()
            }

            #[doc = concat!(
                "Starts building [`",
                stringify!(#struct_name),
                "`] with the `",
                #value_field_name_str,
                "` value set."
            )]
            pub fn with_value(value: #inner_type) -> #initial_value_builder_ty {
                Self::__koruma_builder().with_value(value)
            }

            #[doc = concat!(
                "Returns the stored `",
                #value_field_name_str,
                "` value captured by `#[koruma(value)]`."
            )]
            pub fn #value_field_name(&self) -> &#value_field_type {
                &self.#value_field_name
            }

            #(#direct_builder_methods)*
        }
    };

    let mut with_value_generics = input.generics.clone();
    with_value_generics
        .params
        .push(parse_quote!(S: #module_name::State));
    with_value_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(S::#value_assoc_type: koruma::bon::IsUnset));
    let (with_value_impl_generics, with_value_ty_generics, with_value_where_clause) =
        with_value_generics.split_for_impl();

    let with_value_impl = quote! {
        impl #with_value_impl_generics #builder_name #with_value_ty_generics #with_value_where_clause {
            /// Sets the value field. This is auto-generated by `#[koruma::validator]`.
            pub fn with_value(self, value: #inner_type) -> #output_builder_ty {
                self.#value_field_name(value)
            }
        }
    };

    let mut with_value_ref_generics = input.generics.clone();
    with_value_ref_generics
        .params
        .push(parse_quote!(S: #module_name::State));
    with_value_ref_generics
        .make_where_clause()
        .predicates
        .push(parse_quote!(S::#value_assoc_type: koruma::bon::IsUnset));
    if value_field_capture == ValueFieldCapture::Capture {
        with_value_ref_generics
            .make_where_clause()
            .predicates
            .push(parse_quote!(#inner_type: ::std::clone::Clone));
    }
    let (with_value_ref_impl_generics, with_value_ref_ty_generics, with_value_ref_where_clause) =
        with_value_ref_generics.split_for_impl();
    let builder_ty = quote! { #builder_name #with_value_ref_ty_generics };

    let with_value_ref_impl = match value_field_capture {
        ValueFieldCapture::Capture => quote! {
            impl #with_value_ref_impl_generics koruma::BuilderWithValueRef<#inner_type>
                for #builder_ty #with_value_ref_where_clause
            {
                type Output = #output_builder_ty;

                fn with_value_ref(self, value: &#inner_type) -> Self::Output {
                    self.with_value(value.clone())
                }
            }
        },
        ValueFieldCapture::Skip => quote! {
            impl #with_value_ref_impl_generics koruma::BuilderWithValueRef<#inner_type>
                for #builder_ty #with_value_ref_where_clause
            {
                type Output = Self;

                fn with_value_ref(self, _value: &#inner_type) -> Self::Output {
                    self
                }
            }
        },
    };

    // Generate showcase registration if the attribute is present
    #[cfg(feature = "internal-showcase")]
    let showcase_registration = if let Some(showcase) = showcase_attr {
        let name = &showcase.name;
        let description = &showcase.description;
        let create_closure = &showcase.create;
        let anchor_fn = format_ident!(
            "__koruma_showcase_anchor_{}",
            struct_name.to_string().to_snake_case()
        );
        let input_type = &showcase.input_type;
        let input_type_tokens = match input_type {
            ShowcaseInputType::Text => quote! { ::koruma::showcase::InputType::Text },
            ShowcaseInputType::Numeric => quote! { ::koruma::showcase::InputType::Numeric },
        };
        let module_tokens = if let Some(module) = showcase.module {
            match module {
                ShowcaseModule::String => quote! { ::koruma::showcase::ValidatorModule::String },
                ShowcaseModule::Format => quote! { ::koruma::showcase::ValidatorModule::Format },
                ShowcaseModule::Numeric => quote! { ::koruma::showcase::ValidatorModule::Numeric },
                ShowcaseModule::Collection => {
                    quote! { ::koruma::showcase::ValidatorModule::Collection }
                },
                ShowcaseModule::General => quote! { ::koruma::showcase::ValidatorModule::General },
            }
        } else {
            quote! { ::koruma::showcase::ValidatorModule::General }
        };

        let mut showcase_generics = input.generics.clone();
        let showcase_where_clause = showcase_generics.make_where_clause();
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: ::std::marker::Send + ::std::marker::Sync));
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: ::koruma::Validate<#value_field_type>));
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: ::std::fmt::Display));
        #[cfg(feature = "fluent")]
        showcase_where_clause
            .predicates
            .push(parse_quote!(Self: ::es_fluent::FluentMessage));

        let (impl_generics, type_generics, where_clause) = showcase_generics.split_for_impl();

        #[cfg(feature = "fluent")]
        let fluent_methods = quote! {
            fn fluent_string_with(
                &self,
                localize: &mut ::koruma::showcase::FluentLocalizer<'_>,
            ) -> String {
                use ::es_fluent::FluentMessage;
                self.to_fluent_string_with(localize)
            }
        };

        #[cfg(not(feature = "fluent"))]
        let fluent_methods = quote! {
            fn fluent_string(&self) -> String {
                "(fluent feature required)".to_string()
            }
        };

        quote! {
            impl #impl_generics ::koruma::showcase::DynValidator for #struct_name #type_generics #where_clause {
                fn is_valid(&self) -> bool {
                    ::koruma::Validate::validate(self, &self.#value_field_name)
                }

                fn display_string(&self) -> String {
                    ::std::string::ToString::to_string(self)
                }

                #fluent_methods
            }

            ::koruma::inventory::submit! {
                ::koruma::showcase::ValidatorShowcase {
                    name: #name,
                    description: #description,
                    input_type: #input_type_tokens,
                    module: #module_tokens,
                    create_validator: |input: &str| -> ::anyhow::Result<Box<dyn ::koruma::showcase::DynValidator>> {
                        (#create_closure)(input).map(|v| Box::new(v) as Box<dyn ::koruma::showcase::DynValidator>)
                    },
                }
            }

            #[doc(hidden)]
            pub fn #anchor_fn() {}
        }
    } else {
        quote! {}
    };

    #[cfg(not(feature = "internal-showcase"))]
    let showcase_registration = quote! {};

    Ok(quote! {
        #input

        #value_getter_impl

        #with_value_impl
        #with_value_ref_impl

        #showcase_registration
    })
}

struct DirectBuilderConfig {
    method: Ident,
    ty: Type,
    set_type: Ident,
    into: bool,
    optional_inner_ty: Option<Type>,
}

fn direct_builder_methods(
    input: &ItemStruct,
    value_field_name: &Ident,
    builder_name: &Ident,
    module_name: &Ident,
    builder_generic_args: &[TokenStream2],
) -> Result<Vec<TokenStream2>, syn::Error> {
    let Fields::Named(fields) = &input.fields else {
        return Ok(Vec::new());
    };

    fields
        .named
        .iter()
        .filter(|field| field.ident.as_ref() != Some(value_field_name))
        .filter_map(|field| match direct_builder_config(field) {
            Ok(Some(config)) => Some(Ok(render_direct_builder_method(
                &config,
                builder_name,
                module_name,
                builder_generic_args,
            ))),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        })
        .collect()
}

fn direct_builder_config(field: &Field) -> Result<Option<DirectBuilderConfig>, syn::Error> {
    let Some(field_name) = &field.ident else {
        return Ok(None);
    };

    let mut method = field_name.clone();
    let mut into = false;
    let mut required = false;
    let mut skip_direct_method = false;

    for attr in field.attrs.iter().filter(is_builder_attr) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("into") {
                into = true;
                return Ok(());
            }

            if meta.path.is_ident("required") {
                required = true;
                return Ok(());
            }

            if meta.path.is_ident("skip")
                || meta.path.is_ident("field")
                || meta.path.is_ident("start_fn")
            {
                skip_direct_method = true;
                consume_meta_value_or_group(&meta)?;
                return Ok(());
            }

            if meta.path.is_ident("name") {
                let value = meta.value()?;
                method = value.parse()?;
                return Ok(());
            }

            consume_meta_value_or_group(&meta)
        })?;
    }

    if skip_direct_method {
        return Ok(None);
    }

    let set_type = format_ident!("Set{}", field_name.to_string().to_upper_camel_case());
    let optional_inner_ty = if required {
        None
    } else {
        option_inner_type(&field.ty).cloned()
    };

    Ok(Some(DirectBuilderConfig {
        method,
        ty: field.ty.clone(),
        set_type,
        into,
        optional_inner_ty,
    }))
}

fn is_builder_attr(attr: &&Attribute) -> bool {
    attr.path().is_ident("builder")
}

fn consume_meta_value_or_group(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<(), syn::Error> {
    if meta.input.peek(Token![=]) {
        let value = meta.value()?;
        let _: syn::Expr = value.parse()?;
    } else if meta.input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in meta.input);
        let _: TokenStream2 = content.parse()?;
    }

    Ok(())
}

fn render_direct_builder_method(
    config: &DirectBuilderConfig,
    builder_name: &Ident,
    module_name: &Ident,
    builder_generic_args: &[TokenStream2],
) -> TokenStream2 {
    let method = &config.method;
    let ty = &config.ty;
    let set_type = &config.set_type;
    let output_builder_ty = quote! { #builder_name<#(#builder_generic_args,)* #module_name::#set_type<#module_name::Empty>> };
    let arg_ty = if config.into {
        quote! { impl ::std::convert::Into<#ty> }
    } else {
        quote! { #ty }
    };
    let method_name_str = method.to_string();

    let maybe_method = config.optional_inner_ty.as_ref().map(|inner_ty| {
        let maybe_method = format_ident!("maybe_{}", method);
        let maybe_method_name_str = maybe_method.to_string();
        quote! {
            #[doc = concat!(
                "Starts building this validator with `",
                #maybe_method_name_str,
                "` set."
            )]
            pub fn #maybe_method(value: ::std::option::Option<#inner_ty>) -> #output_builder_ty {
                Self::__koruma_builder().#maybe_method(value)
            }
        }
    });

    quote! {
        #[doc = concat!(
            "Starts building this validator with `",
            #method_name_str,
            "` set."
        )]
        pub fn #method(value: #arg_ty) -> #output_builder_ty {
            Self::__koruma_builder().#method(value)
        }

        #maybe_method
    }
}
