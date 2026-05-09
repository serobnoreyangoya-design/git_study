use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::commands::{open_store, resolve_ticket};
use crate::render;

#[derive(Debug, Parser)]
pub struct Args {
    /// Spec text. Omit to open $EDITOR, or pass --clear to remove.
    pub spec: Option<String>,

    /// Ticket id (or prefix). Defaults to the currently checked-out ticket.
    #[arg(short = 't', long = "ticket")]
    pub ticket: Option<String>,

    /// Read spec from a file.
    #[arg(short = 'F', long = "file", conflicts_with = "spec")]
    pub file: Option<PathBuf>,

    /// Clear the current spec.
    #[arg(short = 'c', long = "clear", conflicts_with_all = ["spec", "file"])]
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
        store.set_spec(&id, None)?;
    } else if let Some(path) = args.file {
        let content = std::fs::read_to_string(&path)?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            store.set_spec(&id, None)?;
        } else {
            store.set_spec(&id, Some(trimmed))?;
        }
    } else if let Some(text) = args.spec {
        store.set_spec(&id, Some(&text))?;
    } else {
        let ticket = store.load(&id)?;
        let initial = ticket.spec.as_deref().unwrap_or("");
        let edited = crate::editor::capture_with_initial(
            "Write the implementation spec below. Lines starting with # are ignored.",
            initial,
        )?;
        match edited {
            Some(text) if !text.trim().is_empty() => store.set_spec(&id, Some(text.trim()))?,
            _ => store.set_spec(&id, None)?,
        }
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
        .spec
        .as_deref()
        .map(|s| s.lines().next().unwrap_or(""))
        .unwrap_or("(none)");
    println!("{} spec: {}", ticket.short_id(), display);
    Ok(())
}
