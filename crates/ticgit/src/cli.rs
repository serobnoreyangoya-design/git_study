//! Clap definitions and top-level dispatch.

use clap::{ArgAction, Parser, Subcommand};

use crate::commands;

#[derive(Debug, Parser)]
#[command(
    name = "ti",
    version,
    disable_version_flag = true,
    about = "Tickets in your Git repo, stored as git-meta metadata",
    long_about = "ti - a Git-native ticket tracker. Tickets, comments, tags, and \
                  assignments are stored as git-meta metadata on the project target \
                  and travel with the repo via sync."
)]
pub struct Cli {
    /// Print version.
    #[arg(short = 'v', long = "version", action = ArgAction::Version)]
    pub version: Option<bool>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialise ticgit metadata on the current repo (idempotent).
    Init,

    /// Create a new ticket.
    New(commands::new::Args),

    /// List tickets, with optional filters.
    #[command(visible_alias = "ls")]
    List(commands::list::Args),

    /// Show one ticket and its comments.
    Show(commands::show::Args),

    /// Select a ticket as "current" for subsequent commands.
    #[command(visible_alias = "co")]
    Checkout(commands::checkout::Args),

    /// Close a ticket by marking it resolved.
    Close(commands::close::Args),

    /// Edit a ticket's title and description in your editor.
    Edit(commands::edit::Args),

    /// Import tickets from external systems.
    Import(commands::import::Args),

    /// Show the most recently touched tickets.
    Recent(commands::recent::Args),

    /// Browse open tickets in an interactive terminal UI.
    Tui(commands::tui::Args),

    /// Add or remove a tag on a ticket.
    Tag(commands::tag::Args),

    /// Change a ticket's state (open / resolved / invalid / hold).
    State(commands::state::Args),

    /// Set or clear a ticket's assigned user.
    Assign(commands::assign::Args),

    /// Set or clear a ticket's points (estimate).
    Points(commands::points::Args),

    /// Set or clear a ticket's milestone.
    Milestone(commands::milestone::Args),

    /// Set a string metadata field on a ticket.
    Meta(commands::meta::Args),

    /// Add a comment to a ticket.
    Comment(commands::comment::Args),

    /// Save the result of `ti list` (with filters) as a named view.
    SaveView(commands::view::SaveArgs),

    /// Show a saved view.
    Views(commands::view::ListArgs),

    /// Sync ticket metadata with a Git remote (pull then push).
    Sync(commands::sync::Args),
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        None => commands::list::run(commands::list::Args::default()),
        Some(Command::Init) => commands::init::run(),
        Some(Command::New(args)) => commands::new::run(args),
        Some(Command::List(args)) => commands::list::run(args),
        Some(Command::Show(args)) => commands::show::run(args),
        Some(Command::Checkout(args)) => commands::checkout::run(args),
        Some(Command::Close(args)) => commands::close::run(args),
        Some(Command::Edit(args)) => commands::edit::run(args),
        Some(Command::Import(args)) => commands::import::run(args),
        Some(Command::Recent(args)) => commands::recent::run(args),
        Some(Command::Tui(args)) => commands::tui::run(args),
        Some(Command::Tag(args)) => commands::tag::run(args),
        Some(Command::State(args)) => commands::state::run(args),
        Some(Command::Assign(args)) => commands::assign::run(args),
        Some(Command::Points(args)) => commands::points::run(args),
        Some(Command::Milestone(args)) => commands::milestone::run(args),
        Some(Command::Meta(args)) => commands::meta::run(args),
        Some(Command::Comment(args)) => commands::comment::run(args),
        Some(Command::SaveView(args)) => commands::view::run_save(args),
        Some(Command::Views(args)) => commands::view::run_list(args),
        Some(Command::Sync(args)) => commands::sync::run_sync(args),
    }
}
