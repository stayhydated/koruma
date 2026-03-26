use clap::{Args, Parser, Subcommand};

#[derive(Clone, Debug)]
pub struct SyncOptions {
    pub check: bool,
    pub verbose: bool,
}

#[derive(Debug, Parser)]
#[command(
    name = "xtask",
    about = "Workspace maintenance tasks.",
    disable_help_subcommand = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Sync source of truth: EN FTL -> Rust std::fmt::Display messages.
    SyncDisplayFtl(SyncArgs),
    /// Build mdBook documentation to web/public/book
    BuildBook,
    /// Build llms.txt from mdBook sources to web/public/llms.txt
    BuildLlmsTxt,
}

#[derive(Args, Clone, Debug, Default)]
pub struct SyncArgs {
    /// Exit with non-zero status if files would change.
    #[arg(long)]
    pub check: bool,
    /// Print each updated Display impl.
    #[arg(long)]
    pub verbose: bool,
}

impl From<SyncArgs> for SyncOptions {
    fn from(value: SyncArgs) -> Self {
        Self {
            check: value.check,
            verbose: value.verbose,
        }
    }
}
