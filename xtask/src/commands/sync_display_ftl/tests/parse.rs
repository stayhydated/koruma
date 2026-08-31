    #[test]
    fn extract_namespace_supports_fluent_and_cfg_attr() {
        use super::collect::extract_namespace;

        let direct: syn::ItemStruct = syn::parse_quote! {
            #[fluent(namespace = "string")]
            struct Direct;
        };
        assert_eq!(extract_namespace(&direct.attrs), Some("string".to_string()));

        let via_cfg_attr: syn::ItemStruct = syn::parse_quote! {
            #[cfg_attr(feature = "fluent", fluent(namespace = "numeric"))]
            struct CfgAttr;
        };
        assert_eq!(
            extract_namespace(&via_cfg_attr.attrs),
            Some("numeric".to_string())
        );

        let none: syn::ItemStruct = syn::parse_quote! {
            struct Plain;
        };
        assert_eq!(extract_namespace(&none.attrs), None);
    }

    #[test]
    fn parse_display_impl_handles_success_none_and_error_cases() {
        let success_impl: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for IncludedValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "Value {1} and {0}", self.actual, self.min)
                }
            }
        };
        let (type_name, map, _span) = parse_display_impl(&success_impl)
            .expect("parse should succeed")
            .expect("display impl should be extracted");
        assert_eq!(type_name, "IncludedValidation");
        assert_eq!(
            map.get("actual").map(|value| compact_ws(value)),
            Some("self.actual".to_string())
        );
        assert_eq!(
            map.get("min").map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );

        let local_var_impl: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for RangeValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let left_delimiter = if self.exclusive_min { "(" } else { "[" };
                    write!(f, "Value {} {}", left_delimiter, self.min)
                }
            }
        };
        let (_, map, _span) = parse_display_impl(&local_var_impl)
            .expect("parse should succeed")
            .expect("display impl should be extracted");
        assert_eq!(
            map.get("left_delimiter").map(|value| compact_ws(value)),
            Some("left_delimiter".to_string())
        );
        assert_eq!(
            map.get("min").map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );

        let non_display_impl: ItemImpl = syn::parse_quote! {
            impl IncludedValidation {
                fn fmt(&self) {}
            }
        };
        assert!(
            parse_display_impl(&non_display_impl)
                .expect("parse should succeed")
                .is_none()
        );

        let invalid_slot_impl: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for BrokenValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "Value {name}", self.actual)
                }
            }
        };
        let err = parse_display_impl(&invalid_slot_impl).expect_err("invalid slot");
        assert!(err.to_string().contains("unrecognized format slot"));
    }

    #[test]
    fn find_write_macro_helpers_cover_statement_and_expression_variants() {
        use super::parse::{
            find_write_macro_in_block, find_write_macro_in_expr, find_write_macro_in_stmt,
        };

        let block: Block = syn::parse_quote!({
            let _x = write!(f, "x");
            println!("ignore");
        });
        assert!(find_write_macro_in_block(&block).is_some());

        let item_stmt: Stmt = syn::parse_quote! { fn helper() {} };
        assert!(find_write_macro_in_stmt(&item_stmt).is_none());

        let macro_stmt: Stmt = syn::parse_quote! { println!("x"); };
        assert!(find_write_macro_in_stmt(&macro_stmt).is_none());

        let expr_try: Expr = syn::parse_quote! { (write!(f, "x"))? };
        assert!(find_write_macro_in_expr(&expr_try).is_some());

        let expr_return: Expr = syn::parse_quote! { return write!(f, "x") };
        assert!(find_write_macro_in_expr(&expr_return).is_some());
    }

    #[test]
    fn format_and_slot_parsing_helpers_cover_happy_and_error_paths() {
        use super::parse::{format_to_expr_map, parse_slot_index};

        let args: Vec<Expr> = vec![
            syn::parse_quote!(self.actual),
            syn::parse_quote!(self.min),
            syn::parse_quote!(custom_expr()),
        ];
        let map = format_to_expr_map("{} {1} {}", &args).expect("format map should parse");
        assert_eq!(
            map.get("actual").map(|value| compact_ws(value)),
            Some("self.actual".to_string())
        );
        assert_eq!(
            map.get("min").map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );

        let err = format_to_expr_map("{9}", &args).expect_err("out-of-range slot should fail");
        assert!(err.to_string().contains("references argument #9"));

        assert_eq!(parse_slot_index("", 3).expect("empty spec"), (3, false));
        assert_eq!(
            parse_slot_index(":x", 4).expect("format-only spec"),
            (4, false)
        );
        assert_eq!(parse_slot_index("2", 0).expect("explicit index"), (2, true));
        assert_eq!(
            parse_slot_index("1:?", 0).expect("explicit index with format"),
            (1, true)
        );
        assert!(parse_slot_index("name", 0).is_err());
        assert!(parse_slot_index("999999999999999999999999999999", 0).is_err());
    }

    #[test]
    fn parse_format_chunks_handles_escapes_and_errors() {
        use super::parse::parse_format_chunks;

        let chunks = parse_format_chunks("A {{brace}} and {0}").expect("valid format string");
        assert_eq!(
            chunks,
            vec![
                FormatChunk::Text("A {brace} and ".to_string()),
                FormatChunk::Slot("0".to_string()),
            ]
        );

        let unmatched = parse_format_chunks("bad }").expect_err("unmatched brace should fail");
        assert!(unmatched.to_string().contains("Unmatched"));

        let unclosed = parse_format_chunks("bad {0").expect_err("unclosed brace should fail");
        assert!(unclosed.to_string().contains("Unclosed"));
    }

    #[test]
    fn infer_variable_name_variants() {
        use super::parse::{infer_variable_name, is_self_expr, member_name};

        let field_expr: Expr = syn::parse_quote!(self.actual);
        assert_eq!(infer_variable_name(&field_expr), Some("actual".to_string()));

        let nested_expr: Expr = syn::parse_quote!((self.inner).value);
        assert_eq!(
            infer_variable_name(&nested_expr),
            Some("inner_value".to_string())
        );

        let method_expr: Expr = syn::parse_quote!((&self.actual).len());
        assert_eq!(
            infer_variable_name(&method_expr),
            Some("actual_len".to_string())
        );

        let unary_expr: Expr = syn::parse_quote!(-self.count);
        assert_eq!(infer_variable_name(&unary_expr), Some("count".to_string()));

        let plain_expr: Expr = syn::parse_quote!(some_value);
        assert_eq!(
            infer_variable_name(&plain_expr),
            Some("some_value".to_string())
        );

        let qualified_expr: Expr = syn::parse_quote!(crate::some_value);
        assert_eq!(infer_variable_name(&qualified_expr), None);

        let named_member: Member = syn::parse_quote!(field_name);
        assert_eq!(member_name(&named_member), Some("field_name".to_string()));
        let unnamed_member = Member::Unnamed(syn::Index::from(2));
        assert_eq!(member_name(&unnamed_member), Some("2".to_string()));

        let self_expr: Expr = syn::parse_quote!(self);
        let not_self_expr: Expr = syn::parse_quote!(other);
        assert!(is_self_expr(&self_expr));
        assert!(!is_self_expr(&not_self_expr));
    }


    #[test]
    fn extract_namespace_helpers_cover_none_paths() {
        use super::collect::{extract_namespace, extract_namespace_from_fluent_meta};

        let from_cfg_attr: syn::ItemStruct = syn::parse_quote! {
            #[doc = "x"]
            #[cfg_attr(feature = "fluent", derive(Clone), fluent(namespace = "cfg-space"))]
            struct CfgNamespace;
        };
        assert_eq!(
            extract_namespace(&from_cfg_attr.attrs),
            Some("cfg-space".to_string())
        );

        let meta_list: syn::MetaList = syn::parse_quote! {
            fluent(flag, module = "x", namespace = 1)
        };
        assert_eq!(extract_namespace_from_fluent_meta(&meta_list), None);

        let simple_cfg_attr: syn::ItemStruct = syn::parse_quote! {
            #[cfg_attr(test, fluent(namespace = "simple-cfg"))]
            struct SimpleCfg;
        };
        assert_eq!(
            extract_namespace(&simple_cfg_attr.attrs),
            Some("simple-cfg".to_string())
        );
    }

    #[test]
    fn parse_display_impl_and_type_helpers_cover_remaining_paths() {
        use super::parse::extract_type_ident;

        let no_fmt: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for NoFmtValidation {
                fn not_fmt(&self) {}
            }
        };
        assert!(
            parse_display_impl(&no_fmt)
                .expect("parse should succeed")
                .is_none()
        );

        let fmt_without_write: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for NoWriteValidation {
                fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    Ok(())
                }
            }
        };
        assert!(
            parse_display_impl(&fmt_without_write)
                .expect("parse should succeed")
                .is_none()
        );

        let too_few_args: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for TooFewArgsValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f)
                }
            }
        };
        assert!(
            parse_display_impl(&too_few_args)
                .expect("parse should succeed")
                .is_none()
        );

        let non_string_format: ItemImpl = syn::parse_quote! {
            impl std::fmt::Display for NonStringFormatValidation {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    let fmt_lit = "{}";
                    write!(f, fmt_lit, self.value)
                }
            }
        };
        assert!(
            parse_display_impl(&non_string_format)
                .expect("parse should succeed")
                .is_none()
        );

        let ref_ty: Type = syn::parse_quote!(&ExampleValidation);
        assert_eq!(
            extract_type_ident(&ref_ty),
            Some("ExampleValidation".to_string())
        );
        let paren_ty: Type = syn::parse_quote!((ExampleValidation));
        assert_eq!(
            extract_type_ident(&paren_ty),
            Some("ExampleValidation".to_string())
        );
        let tuple_ty: Type = syn::parse_quote!((i32, i32));
        assert_eq!(extract_type_ident(&tuple_ty), None);
    }

    #[test]
    fn write_macro_search_and_change_fallback_cover_remaining_paths() {
        use super::parse::{
            find_write_macro_in_block, find_write_macro_in_expr, find_write_macro_in_stmt,
        };

        let only_write_macro_stmt: Stmt = syn::parse_quote! { write!(f, "x"); };
        assert!(find_write_macro_in_stmt(&only_write_macro_stmt).is_some());

        let no_write_block: Block = syn::parse_quote!({
            println!("not write");
            let _ = 1;
        });
        assert!(find_write_macro_in_block(&no_write_block).is_none());

        let non_write_expr: Expr = syn::parse_quote!(println!("x"));
        assert!(find_write_macro_in_expr(&non_write_expr).is_none());

        let block_expr: Expr = syn::parse_quote!({ write!(f, "x") });
        assert!(find_write_macro_in_expr(&block_expr).is_some());

        let grouped_expr = Expr::Group(syn::ExprGroup {
            attrs: Vec::new(),
            group_token: syn::token::Group::default(),
            expr: Box::new(syn::parse_quote!(write!(f, "x"))),
        });
        assert!(find_write_macro_in_expr(&grouped_expr).is_some());

        assert!(!write_call_changed("{", "{"));
        assert!(write_call_changed("{", "}"));
    }

    #[test]
    fn span_to_byte_range_invalid_range_branch_is_covered() {
        let one_line_source = "struct Marker;fn demo(){write!(f, \"m\");}\n";
        let parsed_file: syn::File = syn::parse_file(one_line_source).expect("valid rust");
        let stmt_span = parsed_file
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(item_fn) => Some(item_fn.block.stmts[0].span()),
                _ => None,
            })
            .expect("expected function item");

        let err = span_to_byte_range(stmt_span, &[0], "x").expect_err("invalid range expected");
        assert!(err.to_string().contains("Invalid byte range"));
    }
