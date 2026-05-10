use anyhow::Result;
use clap::Parser;

use crate::commands::{open_store, resolve_ticket};
use crate::render;

#[derive(Debug, Parser)]
pub struct Args {
    /// Priority value (lower = more important). Omit with --clear to remove.
    pub priority: Option<i64>,

    /// Ticket id (or prefix). Defaults to the currently checked-out ticket.
    #[arg(short = 't', long = "ticket")]
    pub ticket: Option<String>,

    /// Clear the current priority value.
    #[arg(short = 'c', long = "clear", conflicts_with = "priority")]
    pub clear: bool,

    /// Output the updated ticket as JSON.
    #[arg(long = "json")]
    pub json: bool,

    /// Output the updated ticket as Markdown.
    #[arg(long = "markdown", conflicts_with = "json")]
    pub markdown: bool,
}

pub fn run(args: Args) -> Result<()> {
    let store = open_store()?;
    let id = resolve_ticket(&store, args.ticket.as_deref())?;

    if args.clear {
        store.set_priority(&id, None)?;
    } else {
        let priority = args
            .priority
            .ok_or_else(|| anyhow::anyhow!("specify priority (or pass --clear)"))?;
        store.set_priority(&id, Some(priority))?;
    }

    let ticket = store.load(&id)?;
    if args.json {
        println!("{}", render::ticket_json(&ticket)?);
        return Ok(());
    }
    if args.markdown {
        println!("{}", render::ticket_markdown(&ticket));
        return Ok(());
    }

    let display = ticket
        .priority
        .map(|p| p.to_string())
        .unwrap_or_else(|| "(none)".to_string());
    println!("{} priority: {}", ticket.short_id(), display);
    Ok(())
}
