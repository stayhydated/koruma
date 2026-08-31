    #[test]
    fn template_conversion_covers_resolver_paths() {
        let validator = ValidatorInfo {
            name: "DemoValidation".to_string(),
            namespace: "demo".to_string(),
            message_id: "demo_validation".to_string(),
            source: PathBuf::from("demo.rs"),
            fields: ["actual", "kind", "other"]
                .into_iter()
                .map(str::to_string)
                .collect(),
        };
        let display = DisplayInfo {
            expr_by_placeholder: std::iter::once(("kind".to_string(), "self.kind".to_string()))
                .collect(),
            source: PathBuf::from("demo.rs"),
            write_span: Span::call_site(),
        };
        let template = vec![
            TemplatePart::Text("A ".to_string()),
            TemplatePart::Placeholder("kind".to_string()),
            TemplatePart::Text(" ".to_string()),
            TemplatePart::Placeholder("other".to_string()),
            TemplatePart::Text(" ".to_string()),
            TemplatePart::Placeholder("actual".to_string()),
        ];

        let (format_literal, args) =
            template_to_write_parts(&template, &validator, &display).expect("template conversion");
        assert_eq!(format_literal, "A {} {} {}");
        assert_eq!(
            args,
            vec![
                "self.kind".to_string(),
                "self.other".to_string(),
                "self.actual".to_string()
            ]
        );

        let unresolved = template_to_write_parts(
            &[TemplatePart::Placeholder("missing".to_string())],
            &validator,
            &display,
        )
        .expect_err("missing placeholder should fail");
        assert!(
            unresolved
                .to_string()
                .contains("Cannot resolve placeholder '$missing'")
        );
    }

    #[test]
    fn string_and_range_helpers_cover_error_paths() {
        use super::template::line_col_to_offset;

        assert_eq!(
            escape_format_literal("\\\"\n\r\t{}"),
            "\\\\\\\"\\n\\r\\t{{}}"
        );
        assert_eq!(
            build_write_call("Value {}", &["self.actual".to_string()]),
            "write!(f, \"Value {}\", self.actual)"
        );

        let source = "ab\ncd\nef";
        let starts = line_start_offsets(source);
        assert_eq!(starts, vec![0, 3, 6]);
        assert_eq!(
            line_col_to_offset(LineColumn { line: 2, column: 1 }, &starts).expect("valid line/col"),
            4
        );
        assert!(line_col_to_offset(LineColumn { line: 0, column: 0 }, &starts).is_err());
        assert!(line_col_to_offset(LineColumn { line: 9, column: 0 }, &starts).is_err());

        let source_with_span = "struct Marker;\nfn demo() {\n    write!(f, \"message\");\n}\n";
        let parsed_file: syn::File = syn::parse_file(source_with_span).expect("valid rust file");
        let stmt_span = parsed_file
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(item_fn) => Some(item_fn.block.stmts[0].span()),
                _ => None,
            })
            .expect("expected function item");
        let valid_range = span_to_byte_range(
            stmt_span,
            &line_start_offsets(source_with_span),
            source_with_span,
        )
        .expect("valid span");
        assert!(valid_range.1 > valid_range.0);

        let invalid_range =
            span_to_byte_range(stmt_span, &[0], "x").expect_err("invalid byte range");
        let message = invalid_range.to_string();
        assert!(
            message.contains("Invalid byte range") || message.contains("out of bounds"),
            "unexpected span error: {message}"
        );
    }

    #[test]
    fn replacement_and_change_detection_helpers() {
        let source = "abc123xyz";
        let replaced = apply_replacements(
            source,
            &[
                Replacement {
                    start: 3,
                    end: 6,
                    replacement: "###".to_string(),
                    type_name: "middle".to_string(),
                },
                Replacement {
                    start: 0,
                    end: 3,
                    replacement: "ABC".to_string(),
                    type_name: "start".to_string(),
                },
            ],
        );
        assert_eq!(replaced, "ABC###xyz");

        assert!(!write_call_changed(
            "write!(f, \"x\", value)",
            "write!(f,\"x\",value)"
        ));
        assert!(write_call_changed(
            "write!(f, \"x\", value)",
            "write!(f, \"y\", value)"
        ));
        assert!(!write_call_changed("not-an-expr", "not-an-expr"));
        assert!(write_call_changed("not-an-expr", "different"));
    }


    #[test]
    fn resolves_actual_from_struct_fields() {
        let validator = ValidatorInfo {
            name: "ExampleValidation".to_string(),
            namespace: "example".to_string(),
            message_id: "example_validation".to_string(),
            source: PathBuf::from("example.rs"),
            fields: std::iter::once("actual".to_string()).collect(),
        };

        let display = DisplayInfo {
            expr_by_placeholder: HashMap::new(),
            source: PathBuf::from("example.rs"),
            write_span: Span::call_site(),
        };

        let template = vec![
            TemplatePart::Text("Value was ".to_string()),
            TemplatePart::Placeholder("actual".to_string()),
            TemplatePart::Text(".".to_string()),
        ];

        let (format_literal, args) =
            template_to_write_parts(&template, &validator, &display).expect("conversion works");

        assert_eq!(format_literal, "Value was {}.");
        assert_eq!(args, vec!["self.actual".to_string()]);
    }

    #[test]
    fn escapes_rust_format_literal_text() {
        assert_eq!(
            escape_format_literal("a { brace } and \"quote\""),
            "a {{ brace }} and \\\"quote\\\""
        );
    }
