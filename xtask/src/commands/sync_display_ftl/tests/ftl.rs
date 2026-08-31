    #[test]
    fn template_and_expression_helpers_cover_remaining_branches() {
        use super::ftl::{push_text_part, root_variable_for_expression, template_from_pattern};

        let ftl = r#"
-term = skip
simple = Value { $actual }.
unrecognized = { "literal" }
"#;
        let resource = parser::parse(ftl.to_string()).expect("valid ftl");

        let simple_pattern = nth_message_pattern(&resource, 0);
        let simple_template = template_from_pattern(simple_pattern).expect("simple template");
        assert_eq!(
            simple_template,
            vec![
                TemplatePart::Text("Value ".to_string()),
                TemplatePart::Placeholder("actual".to_string()),
                TemplatePart::Text(".".to_string()),
            ]
        );

        let unrecognized_pattern = nth_message_pattern(&resource, 1);
        assert!(template_from_pattern(unrecognized_pattern).is_err());

        let inline_expr: ast::Expression<String> =
            ast::Expression::Inline(ast::InlineExpression::VariableReference {
                id: ast::Identifier {
                    name: "actual".to_string(),
                },
            });
        assert_eq!(
            root_variable_for_expression(&inline_expr),
            Some("actual".to_string())
        );

        let no_var_expr: ast::Expression<String> =
            ast::Expression::Inline(ast::InlineExpression::StringLiteral {
                value: "x".to_string(),
            });
        assert_eq!(root_variable_for_expression(&no_var_expr), None);

        let mut template = Vec::new();
        push_text_part(&mut template, String::new());
        push_text_part(&mut template, "A".to_string());
        push_text_part(&mut template, "B".to_string());
        assert_eq!(template, vec![TemplatePart::Text("AB".to_string())]);
    }


    #[test]
    fn extracts_placeholder_from_select_expression() {
        use super::ftl::template_from_pattern;

        let ftl = r#"
ip_validation =
    Not a valid { $kind ->
        [v4] IPv4
       *[other] IP
    } address.
"#;
        let resource = parser::parse(ftl.to_string()).expect("valid ftl");
        let pattern = nth_message_pattern(&resource, 0);

        let template = template_from_pattern(pattern).expect("template conversion works");
        assert_eq!(
            template,
            vec![
                TemplatePart::Text("Not a valid ".to_string()),
                TemplatePart::Placeholder("kind".to_string()),
                TemplatePart::Text(" address.".to_string()),
            ]
        );
    }
    #[test]
    fn collect_ftl_templates_covers_additional_branches() {
        let parse_err_tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(
            &parse_err_tmp.path().join("broken.ftl"),
            "broken = { $value",
        );
        let parse_err = collect_ftl_templates(parse_err_tmp.path()).expect_err("invalid ftl");
        assert!(parse_err.to_string().contains("Failed to parse FTL AST"));

        let unrecognized_tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(
            &unrecognized_tmp.path().join("sample.ftl"),
            r#"
-term = keep
no_value =
    .attr = still ignored
unrecognized = { "literal" }
"#,
        );
        let unrecognized =
            collect_ftl_templates(unrecognized_tmp.path()).expect_err("unrecognized pattern");
        assert!(
            unrecognized
                .to_string()
                .contains("Unrecognized message pattern")
        );

        let skip_tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(&skip_tmp.path().join("ignore.txt"), "ignore");
        write_file(
            &skip_tmp.path().join("sample.ftl"),
            "sample_validation = ok",
        );
        let skipped = collect_ftl_templates(skip_tmp.path()).expect("skip paths should succeed");
        assert_eq!(skipped.len(), 1);
    }

    #[test]
    fn namespace_from_ftl_path_filters_non_ftl_paths() {
        assert_eq!(
            namespace_from_ftl_path(Path::new("sample.ftl")).expect("UTF-8 stem should parse"),
            Some("sample".to_string())
        );
        assert_eq!(
            namespace_from_ftl_path(Path::new("ignore.txt")).expect("non-FTL path should skip"),
            None
        );
    }
