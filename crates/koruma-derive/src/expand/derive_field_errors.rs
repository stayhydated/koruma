use crate::expand::codegen::{helper_generics_for_usages, ref_enum_generics_for_usages};
use crate::expand::plan::ValidationPlan;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Generics, Ident, Type};

pub(crate) fn render_field_error_structs(
    plan: &ValidationPlan,
    struct_name: &Ident,
    generics: &Generics,
    koruma: &TokenStream2,
) -> Vec<TokenStream2> {
    plan.fields
        .iter()
        .filter(|field_plan| !field_plan.is_nested())
        .map(|field_plan| {
            let field_name = &field_plan.name;
            let field_error_struct_name = &field_plan.generated_names.field_error_struct;

            if field_plan.is_newtype() {
                let inner_ty = field_plan.inner_type();
                let has_field_validators = field_plan.has_field_validators();
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();

                if !has_field_validators {
                    let field_is_optional = field_plan.field_optional();
                    let helper_usages: Vec<Type> =
                        vec![syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error }];
                    let helper_generics = helper_generics_for_usages(generics, &helper_usages);
                    let helper_definition = &helper_generics.definition;
                    let helper_impl_generics = &helper_generics.impl_generics;
                    let helper_ty_generics = &helper_generics.ty_generics;
                    let helper_where_clause = &helper_generics.where_clause;
                    let inner_field_ty = if field_is_optional {
                        quote! { Option<<#inner_ty as #koruma::ValidateExt>::Error> }
                    } else {
                        quote! { <#inner_ty as #koruma::ValidateExt>::Error }
                    };
                    let inner_getter = if field_is_optional {
                        quote! {
                            #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`, if any.")]
                            pub fn inner(&self) -> Option<&<#inner_ty as #koruma::ValidateExt>::Error> {
                                self.inner.as_ref()
                            }
                        }
                    } else {
                        quote! {
                            #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`.")]
                            pub fn inner(&self) -> &<#inner_ty as #koruma::ValidateExt>::Error {
                                &self.inner
                            }
                        }
                    };
                    let is_empty_check = if field_is_optional {
                        quote! {
                            self.inner.as_ref().map_or(true, |inner| inner.is_empty())
                        }
                    } else {
                        quote! { self.inner.is_empty() }
                    };
                    let deref_impl = if field_is_optional {
                        quote! {}
                    } else {
                        quote! {
                            impl #helper_impl_generics std::ops::Deref for #field_error_struct_name #helper_ty_generics #helper_where_clause {
                                type Target = <#inner_ty as #koruma::ValidateExt>::Error;

                                fn deref(&self) -> &Self::Target {
                                    &self.inner
                                }
                            }
                        }
                    };

                    return quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` newtype field of [`", #struct_name_str, "`].")]
                        #[derive(Debug, Default)]
                        pub struct #field_error_struct_name #helper_definition {
                            inner: #inner_field_ty,
                        }

                        impl #helper_impl_generics #field_error_struct_name #helper_ty_generics #helper_where_clause {
                            #inner_getter

                            pub fn is_empty(&self) -> bool {
                                #is_empty_check
                            }

                            pub fn has_errors(&self) -> bool {
                                !self.is_empty()
                            }
                        }

                        #deref_impl
                    };
                }

                let field_validator_names: Vec<String> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| v.doc_name())
                    .collect();
                let validators_list = field_validator_names.join("`], `[");
                let mut helper_usages: Vec<Type> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| v.validator_type.as_type())
                    .collect();
                helper_usages
                    .push(syn::parse_quote! { <#inner_ty as #koruma::ValidateExt>::Error });
                let helper_generics = helper_generics_for_usages(generics, &helper_usages);
                let helper_definition = &helper_generics.definition;
                let helper_impl_generics = &helper_generics.impl_generics;
                let helper_ty_generics = &helper_generics.ty_generics;
                let helper_where_clause = &helper_generics.where_clause;
                let field_is_optional = field_plan.field_optional();

                let field_validator_fields: Vec<TokenStream2> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        let vtype = &v.validator_type;
                        quote! { #validator_snake: Option<#vtype> }
                    })
                    .collect();

                let field_validator_getters: Vec<TokenStream2> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        let validator_name = v.doc_name();
                        let vtype = &v.validator_type;
                        quote! {
                            #[doc = concat!("Returns the failed `", #validator_name, "` validator, if any.")]
                            pub fn #validator_snake(&self) -> Option<&#vtype> {
                                self.#validator_snake.as_ref()
                            }
                        }
                    })
                    .collect();

                let field_is_empty_checks: Vec<TokenStream2> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        quote! { self.#validator_snake.is_none() }
                    })
                    .collect();

                let enum_name = &field_plan.generated_names.field_validator_ref_enum;
                let enum_generics = ref_enum_generics_for_usages(generics, &helper_usages);
                let enum_definition = &enum_generics.definition;
                let enum_path = enum_generics.return_type_path(enum_name);

                let enum_variants: Vec<TokenStream2> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| {
                        let variant_name = &v.variant_ident;
                        let vtype = &v.validator_type;
                        quote! { #variant_name(&'koruma #vtype) }
                    })
                    .collect();

                let inner_error_ty = quote! { <#inner_ty as #koruma::ValidateExt>::Error };
                let inner_variant = quote! { Inner(&'koruma #inner_error_ty) };

                let all_pushes: Vec<TokenStream2> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        let variant_name = &v.variant_ident;
                        quote! {
                            self.#validator_snake.as_ref().map(#enum_name::#variant_name)
                        }
                    })
                    .collect();

                let inner_push = quote! {
                    self.inner
                        .as_ref()
                        .filter(|inner| !inner.is_empty())
                        .map(#enum_name::Inner)
                };
                let inner_push = if field_is_optional {
                    inner_push
                } else {
                    quote! {
                        (!self.inner.is_empty()).then_some(#enum_name::Inner(&self.inner))
                    }
                };
                let inner_field_ty = if field_is_optional {
                    quote! { Option<<#inner_ty as #koruma::ValidateExt>::Error> }
                } else {
                    quote! { <#inner_ty as #koruma::ValidateExt>::Error }
                };
                let inner_getter = if field_is_optional {
                    quote! {
                        #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`, if any.")]
                        pub fn inner(&self) -> Option<&<#inner_ty as #koruma::ValidateExt>::Error> {
                            self.inner.as_ref()
                        }
                    }
                } else {
                    quote! {
                        #[doc = concat!("Returns the inner validation error for `", #field_name_str, "`.")]
                        pub fn inner(&self) -> &<#inner_ty as #koruma::ValidateExt>::Error {
                            &self.inner
                        }
                    }
                };
                let inner_is_empty_check = if field_is_optional {
                    quote! { self.inner.as_ref().map_or(true, |inner| inner.is_empty()) }
                } else {
                    quote! { self.inner.is_empty() }
                };

                return quote! {
                    #[doc = concat!("Validators for the `", #field_name_str, "` field of [`", #struct_name_str, "`]: [`", #validators_list, "`] (plus inner validation).")]
                    #[derive(Clone, Copy, Debug)]
                    #[allow(dead_code)]
                    pub enum #enum_name #enum_definition {
                        #(#enum_variants,)*
                        #inner_variant
                    }

                    #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].\n\nValidators: [`", #validators_list, "`].")]
                    #[derive(Debug, Default)]
                    pub struct #field_error_struct_name #helper_definition {
                        #(#field_validator_fields,)*
                        inner: #inner_field_ty,
                    }

                    impl #helper_impl_generics #field_error_struct_name #helper_ty_generics #helper_where_clause {
                        #(#field_validator_getters)*

                        #inner_getter

                        #[doc = concat!("Returns all failed validators for `", #field_name_str, "` including inner newtype validation errors.")]
                        pub fn all(&self) -> impl Iterator<Item = #enum_path> + '_ {
                            [
                                #(#all_pushes,)*
                                #inner_push,
                            ]
                                .into_iter()
                                .flatten()
                        }

                        pub fn is_empty(&self) -> bool {
                            #(#field_is_empty_checks)&&* && #inner_is_empty_check
                        }

                        pub fn has_errors(&self) -> bool {
                            !self.is_empty()
                        }
                    }
                };
            }

            let has_element_validators = field_plan.has_element_validators();

            let field_validator_fields: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|v| {
                    let validator_snake = &v.field_ident;
                    let vtype = &v.validator_type;
                    quote! { #validator_snake: Option<#vtype> }
                })
                .collect();

            let field_validator_getters: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|v| {
                    let validator_snake = &v.field_ident;
                    let validator_name = v.doc_name();
                    let vtype = &v.validator_type;
                    quote! {
                        #[doc = concat!("Returns the failed `", #validator_name, "` validator, if any.")]
                        pub fn #validator_snake(&self) -> Option<&#vtype> {
                            self.#validator_snake.as_ref()
                        }
                    }
                })
                .collect();

            let field_is_empty_checks: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|v| {
                    let validator_snake = &v.field_ident;
                    quote! { self.#validator_snake.is_none() }
                })
                .collect();
            let field_validator_usages: Vec<Type> = field_plan
                .field_validators()
                .iter()
                .map(|v| v.validator_type.as_type())
                .collect();
            let element_error_struct_name = &field_plan.generated_names.element_error_struct;
            let element_enum_name = &field_plan.generated_names.element_validator_ref_enum;
            let element_validator_usages: Vec<Type> = field_plan
                .element_validators()
                .iter()
                .map(|v| v.validator_type.as_type())
                .collect();
            let element_helper_generics = has_element_validators
                .then(|| helper_generics_for_usages(generics, &element_validator_usages));
            let element_error_path = element_helper_generics
                .as_ref()
                .map(|helper| helper.type_path(element_error_struct_name));
            let mut field_error_usages = field_validator_usages.clone();
            if let Some(element_error_path) = &element_error_path {
                field_error_usages.push(syn::parse_quote! { Vec<(usize, #element_error_path)> });
            }
            let field_error_helper_generics =
                helper_generics_for_usages(generics, &field_error_usages);
            let field_error_definition = &field_error_helper_generics.definition;
            let field_error_impl_generics = &field_error_helper_generics.impl_generics;
            let field_error_ty_generics = &field_error_helper_generics.ty_generics;
            let field_error_where_clause = &field_error_helper_generics.where_clause;

            let element_error_struct = if has_element_validators {
                let element_helper_generics =
                    element_helper_generics.as_ref().expect("element validators checked");
                let element_definition = &element_helper_generics.definition;
                let element_impl_generics = &element_helper_generics.impl_generics;
                let element_ty_generics = &element_helper_generics.ty_generics;
                let element_where_clause = &element_helper_generics.where_clause;
                let element_enum_generics =
                    ref_enum_generics_for_usages(generics, &element_validator_usages);
                let element_enum_definition = &element_enum_generics.definition;
                let element_enum_path = element_enum_generics.return_type_path(element_enum_name);

                let element_validator_names: Vec<String> = field_plan
                    .element_validators()
                    .iter()
                    .map(|v| v.doc_name())
                    .collect();
                let element_validators_list = element_validator_names.join("`], `[");
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();

                let element_validator_fields: Vec<TokenStream2> = field_plan
                    .element_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        let vtype = &v.validator_type;
                        quote! { #validator_snake: Option<#vtype> }
                    })
                    .collect();

                let element_validator_getters: Vec<TokenStream2> = field_plan
                    .element_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        let validator_name = v.doc_name();
                        let vtype = &v.validator_type;
                        quote! {
                            #[doc = concat!("Returns the failed `", #validator_name, "` validator, if any.")]
                            pub fn #validator_snake(&self) -> Option<&#vtype> {
                                self.#validator_snake.as_ref()
                            }
                        }
                    })
                    .collect();

                let element_is_empty_checks: Vec<TokenStream2> = field_plan
                    .element_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        quote! { self.#validator_snake.is_none() }
                    })
                    .collect();

                let element_enum_variants: Vec<TokenStream2> = field_plan
                    .element_validators()
                    .iter()
                    .map(|v| {
                        let variant_name = &v.variant_ident;
                        let vtype = &v.validator_type;
                        quote! { #variant_name(&'koruma #vtype) }
                    })
                    .collect();

                let element_all_pushes: Vec<TokenStream2> = field_plan
                    .element_validators()
                    .iter()
                    .map(|v| {
                        let validator_snake = &v.field_ident;
                        let variant_name = &v.variant_ident;
                        quote! {
                            self.#validator_snake.as_ref().map(#element_enum_name::#variant_name)
                        }
                    })
                    .collect();

                quote! {
                    #[doc = concat!("Element validators for the `", #field_name_str, "` field of [`", #struct_name_str, "`]: [`", #element_validators_list, "`].")]
                    #[derive(Clone, Copy, Debug)]
                    #[allow(dead_code)]
                    pub enum #element_enum_name #element_enum_definition {
                        #(#element_enum_variants),*
                    }

                    #[doc = concat!("Per-element validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].")]
                    #[derive(Debug, Default)]
                    pub struct #element_error_struct_name #element_definition {
                        #(#element_validator_fields),*
                    }

                    impl #element_impl_generics #element_error_struct_name #element_ty_generics #element_where_clause {
                        #(#element_validator_getters)*

                        #[doc = concat!("Returns all failed element validators for `", #field_name_str, "`.")]
                        pub fn all(&self) -> impl Iterator<Item = #element_enum_path> + '_ {
                            [
                                #(#element_all_pushes),*
                            ]
                                .into_iter()
                                .flatten()
                        }

                        pub fn is_empty(&self) -> bool {
                            #(#element_is_empty_checks)&&*
                        }

                        pub fn has_errors(&self) -> bool {
                            !self.is_empty()
                        }
                    }
                }
            } else {
                quote! {}
            };

            let element_errors_getter = if has_element_validators {
                let element_error_path = element_error_path
                    .as_ref()
                    .expect("element validators should have an error path");
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();
                quote! {
                    #[doc = concat!("Returns all element validation errors for `", #field_name_str, "` of [`", #struct_name_str, "`] with their indices.")]
                    pub fn element_errors(&self) -> &[(usize, #element_error_path)] {
                        &self.element_errors
                    }
                }
            } else {
                quote! {}
            };

            let element_is_empty_check = if has_element_validators {
                quote! { && self.element_errors.is_empty() }
            } else {
                quote! {}
            };

            let enum_name = &field_plan.generated_names.field_validator_ref_enum;
            let field_enum_helper_generics =
                ref_enum_generics_for_usages(generics, &field_validator_usages);
            let field_enum_definition = &field_enum_helper_generics.definition;
            let field_enum_path = field_enum_helper_generics.return_type_path(enum_name);

            let enum_variants: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|v| {
                    let variant_name = &v.variant_ident;
                    let vtype = &v.validator_type;
                    quote! { #variant_name(&'koruma #vtype) }
                })
                .collect();

            let all_pushes: Vec<TokenStream2> = field_plan
                .field_validators()
                .iter()
                .map(|v| {
                    let validator_snake = &v.field_ident;
                    let variant_name = &v.variant_ident;
                    quote! {
                        self.#validator_snake.as_ref().map(#enum_name::#variant_name)
                    }
                })
                .collect();

            let enum_and_all = if !field_plan.has_field_validators() {
                quote! {}
            } else {
                let validator_names: Vec<String> = field_plan
                    .field_validators()
                    .iter()
                    .map(|v| v.doc_name())
                    .collect();
                let validators_list = validator_names.join("`], `[");
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();
                quote! {
                    #[doc = concat!("Validators for the `", #field_name_str, "` field of [`", #struct_name_str, "`]: [`", #validators_list, "`].")]
                    #[derive(Clone, Copy, Debug)]
                    #[allow(dead_code)]
                    pub enum #enum_name #field_enum_definition {
                        #(#enum_variants),*
                    }
                }
            };

            let all_method = if !field_plan.has_field_validators() {
                quote! {}
            } else {
                let field_name_str = field_name.to_string();
                quote! {
                    #[doc = concat!("Returns all failed validators for `", #field_name_str, "`.")]
                    pub fn all(&self) -> impl Iterator<Item = #field_enum_path> + '_ {
                        [
                            #(#all_pushes),*
                        ]
                            .into_iter()
                            .flatten()
                    }
                }
            };

            let is_empty_body = if !field_plan.has_field_validators() {
                quote! { self.element_errors.is_empty() }
            } else {
                quote! { #(#field_is_empty_checks)&&* #element_is_empty_check }
            };

            let field_error_struct_doc = {
                let field_name_str = field_name.to_string();
                let struct_name_str = struct_name.to_string();
                if has_element_validators && field_plan.has_field_validators() {
                    let field_validator_names: Vec<String> = field_plan
                        .field_validators()
                        .iter()
                        .map(|v| v.doc_name())
                        .collect();
                    let field_validators = field_validator_names.join("`], `[");
                    let element_validator_names: Vec<String> = field_plan
                        .element_validators()
                        .iter()
                        .map(|v| v.doc_name())
                        .collect();
                    let element_validators = element_validator_names.join("`], `[");
                    quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].\n\nField validators: [`", #field_validators, "`]. Element validators: [`", #element_validators, "`].")]
                    }
                } else if has_element_validators {
                    quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`] (element validators only).")]
                    }
                } else if field_plan.has_field_validators() {
                    let field_validator_names: Vec<String> = field_plan
                        .field_validators()
                        .iter()
                        .map(|v| v.doc_name())
                        .collect();
                    let field_validators = field_validator_names.join("`], `[");
                    quote! {
                        #[doc = concat!("Validation errors for the `", #field_name_str, "` field of [`", #struct_name_str, "`].\n\nValidators: [`", #field_validators, "`].")]
                    }
                } else {
                    quote! {}
                }
            };

            let struct_fields = if field_plan.has_field_validators()
                && field_plan.has_element_validators()
            {
                let element_error_path = element_error_path
                    .as_ref()
                    .expect("element validators should have an error path");
                quote! {
                    #(#field_validator_fields,)*
                    element_errors: Vec<(usize, #element_error_path)>
                }
            } else if field_plan.has_element_validators() {
                let element_error_path = element_error_path
                    .as_ref()
                    .expect("element validators should have an error path");
                quote! {
                    element_errors: Vec<(usize, #element_error_path)>
                }
            } else {
                quote! {
                    #(#field_validator_fields),*
                }
            };

            quote! {
                #element_error_struct

                #enum_and_all

                #field_error_struct_doc
                #[derive(Debug, Default)]
                pub struct #field_error_struct_name #field_error_definition {
                    #struct_fields
                }

                impl #field_error_impl_generics #field_error_struct_name #field_error_ty_generics #field_error_where_clause {
                    #(#field_validator_getters)*

                    #element_errors_getter

                    #all_method

                    pub fn is_empty(&self) -> bool {
                        #is_empty_body
                    }

                    pub fn has_errors(&self) -> bool {
                        !self.is_empty()
                    }
                }
            }
        })
        .collect()
}
