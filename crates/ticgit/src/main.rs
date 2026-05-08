use clap::Parser;

mod agent_help;
mod cli;
mod commands;
mod editor;
mod render;
mod session_state;

fn main() -> anyhow::Result<()> {
    if requested_agent_help() {
        agent_help::print();
        return Ok(());
    }

    let args = cli::Cli::parse();
    cli::run(args)
}

fn requested_agent_help() -> bool {
    let mut args = std::env::args_os();
    let _bin = args.next();
    matches!(
        (args.next(), args.next(), args.next()),
        (Some(command), Some(flag), None) if command == "help" && flag == "--agent"
    )
}
