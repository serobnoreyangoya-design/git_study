use anyhow::Result;
use clap::Parser;

use crate::commands::{open_store, resolve_ticket};
use crate::render;

#[derive(Debug, Parser)]
pub struct Args {
    /// Code URI in the format https://<host>/<path>:<branch>.
    pub code: Option<String>,

    /// Ticket id (or prefix). Defaults to the currently checked-out ticket.
    #[arg(short = 't', long = "ticket")]
    pub ticket: Option<String>,

    /// Clear the current code URI.
    #[arg(short = 'c', long = "clear", conflicts_with = "code")]
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
        store.set_code(&id, None)?;
    } else {
        let code = args
            .code
            .ok_or_else(|| anyhow::anyhow!("specify a code URI (or pass --clear)"))?;
        store.set_code(&id, Some(&code))?;
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

    let display = ticket.code.as_deref().unwrap_or("(none)");
    println!("{} code: {}", ticket.short_id(), display);
    Ok(())
}
