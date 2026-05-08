use anyhow::Result;
use clap::Parser;
use ticgit_lib::TicketState;

use crate::commands::{open_store, resolve_ticket};
use crate::render;

#[derive(Debug, Parser)]
pub struct Args {
    /// New state: open, resolved, invalid, or hold.
    pub state: String,

    /// Ticket id (or prefix). Defaults to the currently checked-out ticket.
    #[arg(short = 't', long = "ticket")]
    pub ticket: Option<String>,

    /// Output the updated ticket as JSON.
    #[arg(long = "json")]
    pub json: bool,

    /// Output the updated ticket as Markdown.
    #[arg(long = "markdown", conflicts_with = "json")]
    pub markdown: bool,
}

fn interactive_lifecycle_spec(ticket: &Ticket) -> Result<Option<String>> {
    if !io::stdin().is_terminal() {
        bail!(
            "missing STATE (e.g. `blocked` or `closed:wontfix`); \
             interactive mode needs a terminal, or pass the state on the command line"
        );
    }

    let mut items: Vec<String> = Vec::new();
    let mut specs: Vec<String> = Vec::new();
    for &st in TicketState::ALL {
        let spec = format!("{}:{}", st.status().as_str(), st.as_str());
        let label = if ticket.state == st {
            format!("{spec} (current)")
        } else {
            spec.clone()
        };
        items.push(label);
        specs.push(spec);
    }

    let prompt = format!("Choose new lifecycle for {}", ticket.short_id());
    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .default(
            TicketState::ALL
                .iter()
                .position(|&s| s == ticket.state)
                .unwrap_or(0),
        )
        .interact_opt()
        .context("interactive state selection")?;

    Ok(idx.map(|i| specs[i].clone()))
}

pub fn run(args: Args) -> Result<()> {
    let store = open_store()?;
    let id = resolve_ticket(&store, args.ticket.as_deref())?;

    let lifecycle_spec = match args.lifecycle.as_deref() {
        Some(s) => s.to_string(),
        None => {
            if args.json || args.markdown {
                bail!(
                    "`--json` / `--markdown` require an explicit STATE argument \
                     (e.g. `ti state blocked -t <id> --json`)"
                );
            }
            let ticket = store.load(&id)?;
            match interactive_lifecycle_spec(&ticket)? {
                Some(spec) => spec,
                None => return Ok(()),
            }
        }
    };

    let lifecycle = TicketLifecycle::parse(&lifecycle_spec)?;
    store.set_lifecycle(&id, lifecycle.status, lifecycle.state)?;
    let ticket = store.load(&id)?;
    if args.json {
        println!("{}", render::ticket_json(&ticket)?);
        return Ok(());
    }
    if args.markdown {
        println!("{}", render::ticket_markdown(&ticket));
        return Ok(());
    }
    println!(
        "{} -> {}:{}",
        ticket.short_id(),
        ticket.status,
        ticket.state
    );
    Ok(())
}
