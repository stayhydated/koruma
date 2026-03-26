mod cli;
mod commands;
mod util;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::SyncDisplayFtl(sync) => commands::sync_display_ftl::run(sync),
        Command::BuildBook => commands::build_book::run(),
        Command::BuildLlmsTxt => commands::build_llms_txt::run(),
    }
}
