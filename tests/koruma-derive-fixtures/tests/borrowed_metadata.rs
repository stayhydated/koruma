#[path = "ui-pass/borrowed_metadata.rs"]
mod fixture;

#[test]
fn borrowed_scalar_parameters_report_owned_values() {
    fixture::main();
}
