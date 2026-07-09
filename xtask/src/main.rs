mod cli;
mod commands;

use clap::Parser as _;

use cli::{BuildCommand, Cli, Command, ReleaseCommand};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::SyncDisplayFtl(sync) => commands::sync_display_ftl::run(sync),
        Command::Build { target } => match target {
            BuildCommand::Book => commands::build_book::run(),
            BuildCommand::LlmsTxt => commands::build_llms_txt::run(),
            BuildCommand::Web => commands::build_web::run(),
        },
        Command::Release { action } => match action {
            ReleaseCommand::Plan => commands::release::plan(),
            ReleaseCommand::Publish(args) => commands::release::publish(&args),
        },
    }
}
