use clap::Parser;

mod agent_help;
mod cli;
mod commands;
mod editor;
mod render;
mod session_state;
mod timefmt;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    cli::run(args)
}
