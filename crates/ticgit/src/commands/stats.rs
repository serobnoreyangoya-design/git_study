use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use time::OffsetDateTime;

use crate::commands::open_store;

#[derive(Debug, Parser)]
pub struct Args {
    /// Output stats as JSON.
    #[arg(long = "json")]
    pub json: bool,
}

pub fn run(args: Args) -> Result<()> {
    let store = open_store()?;
    let tickets = store.list()?;
    let total = tickets.len();

    if total == 0 {
        if args.json {
            println!("{}", serde_json::json!({ "total": 0 }));
        } else {
            println!("No tickets.");
        }
        return Ok(());
    }

    let now = OffsetDateTime::now_utc();
    let week_ago = now - time::Duration::days(7);

    let mut open = 0usize;
    let mut closed = 0usize;
    let mut with_comments = 0usize;
    let mut created_7d = 0usize;
    let mut closed_7d = 0usize;
    let mut states: BTreeMap<String, usize> = BTreeMap::new();
    let mut tags: BTreeMap<String, usize> = BTreeMap::new();
    let mut assignees: BTreeMap<String, usize> = BTreeMap::new();

    for t in &tickets {
        match t.status {
            ticgit_lib::TicketStatus::Open => open += 1,
            ticgit_lib::TicketStatus::Closed => closed += 1,
        }
        *states.entry(t.state.as_str().to_string()).or_default() += 1;

        if !t.comments.is_empty() {
            with_comments += 1;
        }

        if t.created_at >= week_ago {
            created_7d += 1;
        }
        if t.status == ticgit_lib::TicketStatus::Closed && t.created_at >= week_ago {
            closed_7d += 1;
        }

        for tag in &t.tags {
            *tags.entry(tag.clone()).or_default() += 1;
        }

        if let Some(ref a) = t.assigned {
            let short = a
                .split_once('@')
                .map(|(local, _)| local)
                .unwrap_or(a);
            *assignees.entry(short.to_string()).or_default() += 1;
        }
    }

    if args.json {
        let states_json: serde_json::Value = states
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::json!(v)))
            .collect();
        let tags_json: serde_json::Value = tags
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::json!(v)))
            .collect();
        let assignees_json: serde_json::Value = assignees
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::json!(v)))
            .collect();

        println!(
            "{}",
            serde_json::json!({
                "total": total,
                "open": open,
                "closed": closed,
                "with_comments": with_comments,
                "created_7d": created_7d,
                "closed_7d": closed_7d,
                "states": states_json,
                "tags": tags_json,
                "assignees": assignees_json,
            })
        );
        return Ok(());
    }

    // ── Human-readable dashboard ────────────────────────────────

    let divider = "─".repeat(48);
    println!("{divider}");
    println!("  TICGIT STATS");
    println!("{divider}");
    println!();

    // Overview
    let comment_note = if with_comments > 0 {
        format!(", {with_comments} with comments")
    } else {
        String::new()
    };
    println!("  Overview     {total} ticket(s){comment_note}");
    println!("  Open         {open:<12}Closed  {closed}");
    println!();

    // States — sorted by count descending
    let mut state_vec: Vec<_> = states.iter().collect();
    state_vec.sort_by(|a, b| b.1.cmp(a.1));
    if !state_vec.is_empty() {
        println!("  States");
        let max_count = *state_vec.iter().map(|(_, c)| *c).max().unwrap_or(&1);
        for (state, count) in &state_vec {
            let bar = bar_chart(**count, max_count, 20);
            println!("    {:<14}{:>3}  {}", state, count, bar);
        }
        println!();
    }

    // Top tags — show top 8
    let mut tag_vec: Vec<_> = tags.iter().collect();
    tag_vec.sort_by(|a, b| b.1.cmp(a.1));
    if !tag_vec.is_empty() {
        println!("  Top Tags");
        let max_count = *tag_vec.first().map(|(_, c)| *c).unwrap_or(&1);
        for (tag, count) in tag_vec.iter().take(8) {
            let bar = bar_chart(**count, max_count, 20);
            println!("    {:<14}{:>3}  {}", tag, count, bar);
        }
        if tag_vec.len() > 8 {
            println!("    ... and {} more", tag_vec.len() - 8);
        }
        println!();
    }

    // Recent activity
    if created_7d > 0 || closed_7d > 0 {
        println!("  Recent Activity (7d)");
        if created_7d > 0 {
            println!("    Created      {created_7d:>3}");
        }
        if closed_7d > 0 {
            println!("    Closed       {closed_7d:>3}");
        }
        println!();
    }

    // Assignees
    if !assignees.is_empty() {
        let mut assignee_vec: Vec<_> = assignees.iter().collect();
        assignee_vec.sort_by(|a, b| b.1.cmp(a.1));
        println!("  Assignees");
        for (name, count) in &assignee_vec {
            println!("    {:<14}{:>3}", name, count);
        }
        println!();
    }

    println!("{divider}");

    Ok(())
}

fn bar_chart(value: usize, max: usize, width: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let filled = (value * width) / max;
    "█".repeat(filled)
}
