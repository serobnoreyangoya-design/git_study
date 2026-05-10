//! Clap definitions and top-level dispatch.

use clap::{ArgAction, Parser, Subcommand};

use crate::commands;

#[derive(Debug, Parser)]
#[command(
    name = "ti",
    version,
    disable_version_flag = true,
    about = "Tickets in your Git repo, stored as git-meta metadata",
    subcommand_help_heading = "",
    hide_possible_values = true,
    help_template = "\
{about}

{usage-heading} {usage}

Create & Browse:
  new        Create a new ticket
  list, ls   List tickets, with optional filters
  show       Show one ticket and its comments
  recent     Show the most recently touched tickets
  tui        Browse open tickets in an interactive terminal UI

Work on Tickets:
  checkout, co  Select a ticket as \"current\"
  edit          Edit a ticket's title and description
  comment       Add a comment to a ticket
  state         Change a ticket's lifecycle status/state
  close         Close a ticket (shorthand for state resolved)

Ticket Fields:
  tag        Add or remove a tag
  assign     Set or clear assigned user
  points     Set or clear points (estimate)
  milestone  Set or clear milestone
  subissue   Make a ticket a sub-issue of another
  code       Set or clear a code URI (repo + branch)
  meta       Set a custom metadata field

Views & Import:
  save-view  Save a filtered list as a named view
  views      Show a saved view
  stats      Show a ticket stats dashboard
  import     Import tickets from external systems (e.g. GitHub)

Sync & Setup:
  sync       Sync ticket metadata with a Git remote
  pull       Pull tickets from a fork or remote URL
  init       Initialise ticgit on the current repo
  setup      Configure git-meta remote from .git-meta
  update     Update ti to the latest release

Agents:
  ti help --agent          Markdown guide for AI agents
  ti list --json           Machine-readable ticket list
  ti show <id> --json      Machine-readable ticket detail
  ti show <id> --markdown  Ticket detail with next-step suggestions

Examples:
  ti new --title \"fix the parser\" --tags bug
  ti list --tag bug --status open
  ti show a3f
  ti checkout a3f && ti comment \"on it\"
  ti state resolved --ticket a3f
  ti sync

Agent Examples:
  ti help --agent
  ti list --json | jq '.[].title'
  ti show a3f --markdown
  ti new --title \"fix bug\" --json
  ti comment --ticket a3f \"done\" --json

Options:
{options}"
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
    // -- Create & browse --------------------------------------------------

    /// Create a new ticket.
    #[command(next_help_heading = "Create & Browse")]
    New(commands::new::Args),

    /// List tickets, with optional filters.
    #[command(visible_alias = "ls")]
    List(commands::list::Args),

    /// Show one ticket and its comments.
    Show(commands::show::Args),

    /// Show the most recently touched tickets.
    Recent(commands::recent::Args),

    /// Browse open tickets in an interactive terminal UI.
    Tui(commands::tui::Args),

    // -- Work on tickets --------------------------------------------------

    /// Select a ticket as "current" for subsequent commands.
    #[command(visible_alias = "co", next_help_heading = "Work on Tickets")]
    Checkout(commands::checkout::Args),

    /// Edit a ticket's title and description in your editor.
    Edit(commands::edit::Args),

    /// Add a comment to a ticket.
    Comment(commands::comment::Args),

    /// Change a ticket's lifecycle status/state.
    State(commands::state::Args),

    /// Alias for `state`.
    Status(commands::state::Args),

    /// Close a ticket by marking it resolved.
    Close(commands::close::Args),

    // -- Ticket fields ----------------------------------------------------

    /// Add or remove a tag on a ticket.
    #[command(next_help_heading = "Ticket Fields")]
    Tag(commands::tag::Args),

    /// Set or clear a ticket's assigned user.
    Assign(commands::assign::Args),

    /// Set or clear a ticket's points (estimate).
    Points(commands::points::Args),

    /// Set or clear a ticket's milestone.
    Milestone(commands::milestone::Args),

    /// Make a ticket a sub-issue of another ticket.
    Subissue(commands::subissue::Args),

    /// Set or clear a code URI (https://host/path:branch) for associated code.
    Code(commands::code::Args),

    /// Set or clear a ticket's implementation spec.
    Spec(commands::spec::Args),

    /// Set a string metadata field on a ticket.
    Meta(commands::meta::Args),

    // -- Views & import ---------------------------------------------------

    /// Save the result of `ti list` (with filters) as a named view.
    #[command(next_help_heading = "Views & Import")]
    SaveView(commands::view::SaveArgs),

    /// Show a saved view.
    Views(commands::view::ListArgs),

    /// Show a ticket stats dashboard.
    Stats(commands::stats::Args),

    /// Import tickets from external systems.
    Import(commands::import::Args),

    // -- Sync & setup -----------------------------------------------------

    /// Sync ticket metadata with a Git remote (pull then push).
    #[command(next_help_heading = "Sync & Setup")]
    Sync(commands::sync::Args),

    /// Pull tickets from a fork or remote URL.
    Pull(commands::pull::Args),

    /// Initialise ticgit metadata on the current repo (idempotent).
    Init,

    /// Configure git-meta remote from `.git-meta` file (idempotent).
    Setup,

    /// Update ti to the latest release.
    Update(commands::update::Args),
}

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        None => commands::list::run(commands::list::Args::default()),
        Some(Command::Init) => commands::init::run(),
        Some(Command::Setup) => commands::setup::run(),
        Some(Command::New(args)) => commands::new::run(args),
        Some(Command::List(args)) => commands::list::run(args),
        Some(Command::Show(args)) => commands::show::run(args),
        Some(Command::Checkout(args)) => commands::checkout::run(args),
        Some(Command::Close(args)) => commands::close::run(args),
        Some(Command::Edit(args)) => commands::edit::run(args),
        Some(Command::Stats(args)) => commands::stats::run(args),
        Some(Command::Import(args)) => commands::import::run(args),
        Some(Command::Recent(args)) => commands::recent::run(args),
        Some(Command::Tui(args)) => commands::tui::run(args),
        Some(Command::Tag(args)) => commands::tag::run(args),
        Some(Command::State(args)) => commands::state::run(args),
        Some(Command::Status(args)) => commands::state::run(args),
        Some(Command::Assign(args)) => commands::assign::run(args),
        Some(Command::Points(args)) => commands::points::run(args),
        Some(Command::Milestone(args)) => commands::milestone::run(args),
        Some(Command::Subissue(args)) => commands::subissue::run(args),
        Some(Command::Code(args)) => commands::code::run(args),
        Some(Command::Spec(args)) => commands::spec::run(args),
        Some(Command::Meta(args)) => commands::meta::run(args),
        Some(Command::Comment(args)) => commands::comment::run(args),
        Some(Command::SaveView(args)) => commands::view::run_save(args),
        Some(Command::Views(args)) => commands::view::run_list(args),
        Some(Command::Sync(args)) => commands::sync::run_sync(args),
        Some(Command::Pull(args)) => commands::pull::run(args),
        Some(Command::Update(args)) => commands::update::run(args),
    }
}
