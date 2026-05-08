use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::commands::{open_store, resolve_ticket};
use crate::render;

#[derive(Debug, Parser)]
pub struct Args {
    /// Metadata field name.
    pub field: String,

    /// Metadata value. Omit when using --file.
    pub value: Option<String>,

    /// Ticket id (or prefix). Defaults to the currently checked-out ticket.
    #[arg(short = 't', long = "ticket")]
    pub ticket: Option<String>,

    /// Read the metadata value from a file.
    #[arg(short = 'F', long = "file", conflicts_with = "value")]
    pub file: Option<PathBuf>,

    /// Output the updated ticket as JSON.
    #[arg(long = "json")]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let store = open_store()?;
    let id = resolve_ticket(&store, args.ticket.as_deref())?;
    let value = if let Some(path) = args.file {
        std::fs::read_to_string(&path)
            .with_context(|| format!("reading metadata value from `{}`", path.display()))?
    } else {
        args.value.context("metadata value required")?
    };

    store.set_meta(&id, &args.field, &value)?;
    let ticket = store.load(&id)?;
    if args.json {
        println!("{}", render::ticket_json(&ticket)?);
        return Ok(());
    }

    println!(
        "{} meta {}: {}",
        ticket.short_id(),
        args.field.trim(),
        value.replace('\n', "\\n")
    );
    Ok(())
}
