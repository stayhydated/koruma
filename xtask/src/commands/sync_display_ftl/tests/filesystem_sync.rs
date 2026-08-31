    #[test]
    fn sync_args_into_sync_options() {
        let options: SyncArgs = SyncArgs {
            check: true,
            verbose: true,
        };
        assert!(options.check);
        assert!(options.verbose);
    }

    #[test]
    fn sync_display_ftl_check_mode_reports_pending_changes() {
        let (_tmp, validators_root, ftl_root, _validator_file) = create_sync_fixture();

        let err = run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: true,
                verbose: false,
            },
        )
        .expect_err("expected pending changes in check mode");

        assert!(err.to_string().contains("would be updated"));
    }

    #[test]
    fn sync_display_ftl_applies_changes_and_then_reports_clean() {
        let (_tmp, validators_root, ftl_root, validator_file) = create_sync_fixture();

        run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: false,
                verbose: true,
            },
        )
        .expect("sync should apply changes");

        let updated = fs::read_to_string(&validator_file).expect("failed to read updated file");
        assert!(compact_ws(&updated).contains("write!(f,\"Value{}and{}.\",self.min,self.actual)"));

        run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: true,
                verbose: false,
            },
        )
        .expect("check mode should pass after sync");
    }


    #[test]
    fn workspace_wrapper_and_root_paths_are_reachable() {
        let root = stayhydated_xtask::workspace_root_from_xtask_manifest().unwrap();
        assert!(root.ends_with("koruma"));
        let _ = run(SyncArgs {
            check: true,
            verbose: false,
        });
    }

    #[test]
    fn sync_display_ftl_warns_for_missing_display_and_missing_message() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let validators_root = tmp.path().join("validators");
        let ftl_root = tmp.path().join("ftl");
        let validator_file = validators_root.join("sample.rs");
        let ftl_file = ftl_root.join("sample.ftl");
        write_file(
            &validator_file,
            r#"
#[fluent(namespace = "sample")]
pub struct MissingDisplayValidation {
    #[koruma(value)]
    pub actual: String,
}

#[fluent(namespace = "sample")]
pub struct MissingMessageValidation {
    #[koruma(value)]
    pub actual: String,
}

impl std::fmt::Display for MissingMessageValidation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bad {}", self.actual)
    }
}
"#,
        );
        write_file(&ftl_file, "another_validation = Value { $actual }.");

        run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: false,
                verbose: false,
            },
        )
        .expect("sync should tolerate missing display/message with warnings");
    }

    #[test]
    fn sync_display_ftl_surfaces_template_conversion_context() {
        let tmp = tempfile::tempdir().expect("failed to create temp directory");
        let validators_root = tmp.path().join("validators");
        let ftl_root = tmp.path().join("ftl");
        let validator_file = validators_root.join("sample.rs");
        let ftl_file = ftl_root.join("sample.ftl");
        write_file(&validator_file, fixture_validator_source());
        write_file(&ftl_file, "example_validation = Unknown { $missing }.");

        let err = run_with_roots(
            &validators_root,
            &ftl_root,
            SyncArgs {
                check: false,
                verbose: false,
            },
        )
        .expect_err("placeholder resolution should fail");
        assert!(
            err.to_string()
                .contains("Failed to convert FTL template for ExampleValidation")
        );
    }
