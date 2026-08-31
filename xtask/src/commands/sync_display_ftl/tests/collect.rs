    #[test]
    fn collect_rs_files_walks_directories() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        write_file(&tmp.path().join("root.rs"), "fn root() {}");
        write_file(&tmp.path().join("nested/inner.rs"), "fn inner() {}");
        write_file(&tmp.path().join("nested/ignore.txt"), "ignore");

        let mut files = Vec::new();
        collect_rs_files(tmp.path(), &mut files).expect("collection should succeed");
        files.sort();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|path| path.ends_with("root.rs")));
        assert!(files.iter().any(|path| path.ends_with("inner.rs")));
    }

    #[test]
    fn collect_validator_info_filters_and_extracts_namespace() {
        let source = r#"
#[fluent(namespace = "demo")]
pub struct IncludedValidation {
    pub min: usize,
    pub actual: String,
}

pub struct NoNamespaceValidation {
    pub actual: String,
}

#[fluent(namespace = "demo")]
pub struct IgnoredType {
    pub actual: String,
}
"#;

        let parsed = syn::parse_file(source).expect("valid rust");
        let mut validators = BTreeMap::new();
        collect_validator_info(Path::new("demo.rs"), &parsed, &mut validators);

        assert_eq!(validators.len(), 1);
        let info = validators
            .get("IncludedValidation")
            .expect("included validator should exist");
        assert_eq!(info.namespace, "demo");
        assert_eq!(info.message_id, "included_validation");
        assert!(info.fields.contains("min"));
        assert!(info.fields.contains("actual"));
    }

    #[test]
    fn collect_display_info_reads_display_impls() {
        let source = r#"
pub struct IncludedValidation;

impl std::fmt::Display for IncludedValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value {1} and {0}", self.actual, self.min)
    }
}
"#;
        let parsed = syn::parse_file(source).expect("valid rust");
        let mut displays = BTreeMap::new();
        collect_display_info(Path::new("demo.rs"), &parsed, &mut displays).expect("parse display");

        let info = displays
            .get("IncludedValidation")
            .expect("display info should exist");
        assert_eq!(
            info.expr_by_placeholder
                .get("actual")
                .map(|value| compact_ws(value)),
            Some("self.actual".to_string())
        );
        assert_eq!(
            info.expr_by_placeholder
                .get("min")
                .map(|value| compact_ws(value)),
            Some("self.min".to_string())
        );
    }


    #[test]
    fn collect_validator_and_display_info_cover_additional_paths() {
        let source = r#"
#[fluent(namespace = "demo")]
pub struct TupleValidation(i32);

#[fluent(namespace = "demo")]
pub struct DisplayedValidation {
    pub actual: i32,
}

impl DisplayedValidation {
    fn helper(&self) {}
}

impl std::fmt::Display for DisplayedValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.actual)
    }
}
"#;
        let parsed = syn::parse_file(source).expect("valid rust");

        let mut validators = BTreeMap::new();
        collect_validator_info(Path::new("demo.rs"), &parsed, &mut validators);
        let tuple = validators
            .get("TupleValidation")
            .expect("tuple validator should be collected");
        assert!(tuple.fields.is_empty());

        let mut displays = BTreeMap::new();
        collect_display_info(Path::new("demo.rs"), &parsed, &mut displays).expect("display parse");
        assert!(displays.contains_key("DisplayedValidation"));
    }
