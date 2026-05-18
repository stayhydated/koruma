use clap::{Args, Parser, Subcommand};

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
    /// Build generated workspace artifacts
    Build {
        #[command(subcommand)]
        target: BuildCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum BuildCommand {
    /// Build mdBook documentation to web/public/book
    Book,
    /// Build llms.txt from mdBook sources to web/public/llms.txt
    LlmsTxt,
    /// Build the Dioxus site into web/dist for GitHub Pages
    Web,
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
