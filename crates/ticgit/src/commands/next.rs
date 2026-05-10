use anyhow::Result;
use clap::Parser;
use ticgit_lib::{Ticket, TicketState, TicketStatus};

use crate::commands::{open_store, SessionGitDir};
use crate::render;
use crate::session_state::State;

#[derive(Debug, Parser)]
pub struct Args {
    /// Only consider tickets with this tag.
    #[arg(short = 'g', long = "tag")]
    pub tag: Option<String>,

    /// Only consider tickets assigned to this user.
    #[arg(short = 'a', long = "assigned")]
    pub assigned: Option<String>,

    /// Output as JSON.
    #[arg(long = "json")]
    pub json: bool,

    /// Output as Markdown.
    #[arg(long = "markdown", conflicts_with = "json")]
    pub markdown: bool,
}

pub fn run(args: Args) -> Result<()> {
    let store = open_store()?;
    let git_dir = store.session().repo_git_dir();
    let tickets = store.list()?;

    let mut candidates: Vec<Ticket> = tickets
        .into_iter()
        .filter(|t| t.status == TicketStatus::Open)
        .filter(|t| t.parent.is_none())
        .filter(|t| {
            if let Some(tag) = &args.tag {
                t.tags.contains(tag)
            } else {
                true
            }
        })
        .filter(|t| {
            if let Some(assigned) = &args.assigned {
                t.assigned.as_deref() == Some(assigned.as_str())
            } else {
                true
            }
        })
        .collect();

    // Rank candidates: higher score = better candidate
    candidates.sort_by(|a, b| score(b).cmp(&score(a)));

    let ticket = match candidates.into_iter().next() {
        Some(t) => t,
        None => {
            if args.json {
                println!("{}", serde_json::json!({ "next": null }));
            } else if args.markdown {
                println!("# Next Ticket\n\nNo open tickets match the criteria.");
            } else {
                println!("No open tickets to work on.");
            }
            return Ok(());
        }
    };

    // Check it out
    let mut state = State::load().unwrap_or_default();
    state.set_current(&git_dir, ticket.id);
    state.save()?;

    if args.json {
        println!("{}", render::ticket_json(&ticket)?);
        return Ok(());
    }
    if args.markdown {
        println!("{}", render::ticket_markdown(&ticket));
        return Ok(());
    }

    println!("Next: {} - {}", ticket.short_id(), ticket.title);
    println!("  State: {}  Points: {}", ticket.state.as_str(), ticket.points.map(|p| p.to_string()).unwrap_or_else(|| "-".into()));
    if let Some(a) = &ticket.assigned {
        println!("  Assigned: {a}");
    }
    if !ticket.tags.is_empty() {
        let tags: Vec<_> = ticket.tags.iter().cloned().collect();
        println!("  Tags: {}", tags.join(", "));
    }
    println!("Checked out.");
    Ok(())
}

/// Score a ticket for work priority. Higher = should be worked on first.
fn score(t: &Ticket) -> i64 {
    let mut s: i64 = 0;

    // Prefer tickets already in progress
    match t.state {
        TicketState::InProgress => s += 100,
        TicketState::Assigned => s += 80,
        TicketState::Review => s += 60,
        TicketState::Blocked => s -= 200,  // skip blocked tickets
        TicketState::New => s += 40,
        _ => {}
    }

    // Prefer assigned tickets (someone decided this matters)
    if t.assigned.is_some() {
        s += 20;
    }

    // Prefer higher priority (points)
    if let Some(p) = t.points {
        s += p * 10;
    }

    // Prefer older tickets (days since creation, capped)
    let age_days = (time::OffsetDateTime::now_utc() - t.created_at).whole_days();
    s += age_days.min(30);

    s
}
