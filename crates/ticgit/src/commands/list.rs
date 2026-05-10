use anyhow::Result;
use clap::Parser;
use ticgit_lib::{Filter, SearchFilter, SortOrder, TicketLifecycle, TicketStatus};

use crate::commands::{open_store, SessionGitDir};
use crate::render;
use crate::session_state::State;

#[derive(Debug, Default, Parser)]
pub struct Args {
    /// Show only tickets in this status/state. Defaults to status open.
    #[arg(short = 's', long = "state")]
    pub state: Option<String>,

    /// Show only tickets in this broad status.
    #[arg(long = "status")]
    pub status: Option<String>,

    /// Show all tickets, without the default open-only filter or limit.
    #[arg(long = "all")]
    pub all: bool,

    /// Show only tickets with this tag.
    #[arg(short = 'g', long = "tag")]
    pub tag: Option<String>,

    /// Show only tickets assigned to this user.
    #[arg(short = 'a', long = "assigned")]
    pub assigned: Option<String>,

    /// Show only tickets that have at least one tag.
    #[arg(short = 'T', long = "only-tagged")]
    pub only_tagged: bool,

    /// Search title, description, and comments. Use `title:term`, `description:term`, or `comments:term` to scope.
    #[arg(long = "search")]
    pub search: Option<String>,

    /// Sort order. e.g. `state`, `title.desc`, `created`, `assigned`.
    #[arg(short = 'o', long = "order")]
    pub order: Option<String>,

    /// Show tickets that belong to a saved view.
    #[arg(short = 'V', long = "view")]
    pub view: Option<String>,

    /// Include sub-issues (hidden by default).
    #[arg(long = "subissues")]
    pub subissues: bool,

    /// Maximum number of tickets to show.
    #[arg(short = 'n', long = "limit", default_value_t = 20)]
    pub limit: usize,

    /// Output as JSON.
    #[arg(long = "json")]
    pub json: bool,

    /// Output as Markdown.
    #[arg(long = "markdown", conflicts_with = "json")]
    pub markdown: bool,
}

pub fn run(args: Args) -> Result<()> {
    let store = open_store()?;
    let mut tickets = store.list()?;
    let open_ref_lengths = render::open_ticket_ref_lengths(&tickets);

    if let Some(view_name) = &args.view {
        let ids = store.load_view(view_name)?;
        tickets.retain(|t| ids.contains(&t.id));
    }

    let mut status = match args.status.as_deref() {
        Some(s) => Some(TicketStatus::parse(s)?),
        None if args.all || args.state.is_some() => None,
        None => Some(TicketStatus::Open),
    };
    let mut state = None;
    if let Some(spec) = args.state.as_deref() {
        let lifecycle = TicketLifecycle::parse(spec)?;
        status = Some(lifecycle.status);
        if TicketStatus::parse(spec).is_err() {
            state = Some(lifecycle.state);
        }
    }
    let order = match args.order.as_deref() {
        Some(spec) => Some(
            SortOrder::parse(spec).ok_or_else(|| anyhow::anyhow!("unknown sort order `{spec}`"))?,
        ),
        None => None,
    };
    let search = match args.search.as_deref() {
        Some(spec) => Some(SearchFilter::parse(spec).map_err(|e| anyhow::anyhow!(e))?),
        None => None,
    };

    let filter = Filter {
        status,
        state,
        tag: args.tag,
        assigned: args.assigned,
        only_tagged: args.only_tagged,
        search,
        order,
        hide_subissues: !args.subissues,
    };
    let mut tickets = ticgit_lib::query::apply(tickets, &filter);
    if !args.all && args.limit > 0 {
        tickets.truncate(args.limit);
    }

    if args.json {
        println!("{}", render::tickets_json(&tickets)?);
        return Ok(());
    }

    if args.markdown {
        println!("{}", render::tickets_markdown(&tickets));
        return Ok(());
    }

    if tickets.is_empty() {
        println!("(no tickets)");
        return Ok(());
    }

    let session_state = State::load().unwrap_or_default();
    let current = session_state.current_for(&store.session().repo_git_dir());
    println!(
        "{}",
        render::tickets_table_with_refs(&tickets, current.as_ref(), &open_ref_lengths)
    );
    Ok(())
}
