use std::io::{self, IsTerminal};

use anyhow::{bail, Context, Result};
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use ticgit_lib::{Ticket, TicketLifecycle, TicketState};

use crate::commands::{open_store, resolve_ticket};
use crate::render;

#[derive(Debug, Parser)]
pub struct Args {
    /// New lifecycle value: status, state, or status:state.
    /// When omitted, choose interactively from a list (requires a terminal).
    pub lifecycle: Option<String>,

    /// Ticket id (or prefix). Defaults to the currently checked-out ticket.
    #[arg(short = 't', long = "ticket")]
    pub ticket: Option<String>,

    /// Output the updated ticket as JSON.
    #[arg(long = "json")]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let store = open_store()?;
    let id = resolve_ticket(&store, args.ticket.as_deref())?;
    let new_state = TicketState::parse(&args.state)?;
    store.set_state(&id, new_state)?;
    let ticket = store.load(&id)?;
    if args.json {
        println!("{}", render::ticket_json(&ticket)?);
        return Ok(());
    }
    println!("{} -> {}", ticket.short_id(), ticket.state);
    Ok(())
}
