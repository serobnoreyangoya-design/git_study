use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use ticgit_lib::Ticket;

use crate::commands::{open_store, resolve_ticket};
use crate::editor;
use crate::render;

#[derive(Debug, Parser)]
pub struct Args {
    /// Ticket id (or prefix). Defaults to the currently checked-out ticket.
    pub ticket: Option<String>,

    /// Read the updated title and description from a file.
    #[arg(short = 'F', long = "file")]
    pub file: Option<PathBuf>,

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
    let ticket = store.load(&id)?;

    let (title, description) = if let Some(path) = args.file {
        editor::read_ticket_edit_file(&path)?
    } else {
        let edited = editor::capture_with_initial(
            "Edit the title on the first line. Remaining non-comment lines become the description.",
            &editor_body(&ticket),
        )?
        .context("ticket title cannot be empty")?;
        editor::parse_ticket_edit(&edited)?
    };

    store.set_title(&id, &title)?;
    store.set_description(&id, description.as_deref())?;

    let ticket = store.load(&id)?;
    if args.json {
        println!("{}", render::ticket_json(&ticket)?);
        return Ok(());
    }
    if args.markdown {
        println!("{}", render::ticket_markdown(&ticket));
        return Ok(());
    }
    println!("Updated {}.", ticket.short_id());
    Ok(())
}

fn editor_body(ticket: &Ticket) -> String {
    let mut body = ticket.title.clone();
    if let Some(description) = &ticket.description {
        body.push_str("\n\n");
        body.push_str(description);
    }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ticket_edit_splits_title_and_description() {
        let (title, description) =
            editor::parse_ticket_edit("updated title\n\nfirst line\nsecond line\n").unwrap();

        assert_eq!(title, "updated title");
        assert_eq!(description.as_deref(), Some("first line\nsecond line"));
    }

    #[test]
    fn parse_ticket_edit_allows_clearing_description() {
        let (title, description) = editor::parse_ticket_edit("updated title\n\n").unwrap();

        assert_eq!(title, "updated title");
        assert_eq!(description, None);
    }

    #[test]
    fn parse_ticket_edit_rejects_empty_title() {
        assert!(editor::parse_ticket_edit("\nbody").is_err());
    }
}
