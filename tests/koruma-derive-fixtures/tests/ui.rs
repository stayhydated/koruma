#[test]
fn compile_fail_diagnostics() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
    #[cfg(feature = "fluent")]
    t.compile_fail("tests/ui-fluent/*.rs");
    t.pass("tests/ui-pass/*.rs");
    #[cfg(feature = "fluent")]
    t.pass("tests/ui-pass-fluent/*.rs");
}
