use std::process::{Command, ExitStatus};

fn run_xtask(args: &[&str]) -> ExitStatus {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .status()
        .expect("failed to run xtask binary")
}

#[test]
fn cli_flatten_and_subcommand_paths_execute() {
    // Flat args path (`None => cli.sync.into()` in main).
    let flat = run_xtask(&["--check"]);
    assert!(
        matches!(flat.code(), Some(0) | Some(1)),
        "unexpected exit status for flat args: {flat:?}"
    );

    // Explicit subcommand path (`Some(Command::SyncDisplayFtl(..))` in main).
    let subcommand = run_xtask(&["sync-display-ftl", "--check"]);
    assert!(
        matches!(subcommand.code(), Some(0) | Some(1)),
        "unexpected exit status for subcommand: {subcommand:?}"
    );
}
