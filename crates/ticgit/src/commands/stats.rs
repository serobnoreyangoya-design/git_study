use std::collections::BTreeMap;

use anyhow::Result;
use clap::Parser;
use crossterm::terminal;
use time::OffsetDateTime;

use crate::commands::open_store;

#[derive(Debug, Parser)]
pub struct Args {
    /// Output stats as JSON.
    #[arg(long = "json")]
    pub json: bool,
}

// ANSI colour helpers
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RED: &str = "\x1b[31m";
const MAGENTA: &str = "\x1b[35m";
const WHITE: &str = "\x1b[97m";
const BG_GREEN: &str = "\x1b[42m";
const BG_CYAN: &str = "\x1b[46m";
const _BG_YELLOW: &str = "\x1b[43m";
const BG_MAGENTA: &str = "\x1b[45m";

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

    // ── Terminal-aware dashboard ────────────────────────────────

    let (term_width, term_height) = terminal::size()
        .map(|(w, h)| (w as usize, h as usize))
        .unwrap_or((80, 24));

    let width = term_width.min(120);
    let two_col = width >= 70;

    // Pre-sort data.
    let mut state_vec: Vec<_> = states.into_iter().collect();
    state_vec.sort_by(|a, b| b.1.cmp(&a.1));

    let mut tag_vec: Vec<_> = tags.into_iter().collect();
    tag_vec.sort_by(|a, b| b.1.cmp(&a.1));

    let mut assignee_vec: Vec<_> = assignees.into_iter().collect();
    assignee_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // Limit tags/assignees to fit terminal height.
    // Reserve lines: header(3) + overview(3) + spacing(3) + footer(1) = 10 baseline.
    let avail_lines = term_height.saturating_sub(10);
    let max_section_rows = if two_col {
        avail_lines.max(4)
    } else {
        // Single column: sections are stacked. Budget lines across sections.
        let sections = 1 + (!tag_vec.is_empty() as usize)
            + (!assignee_vec.is_empty() as usize)
            + ((created_7d > 0 || closed_7d > 0) as usize);
        if sections > 0 { avail_lines / sections } else { avail_lines }
    }.max(3);

    let tag_limit = max_section_rows.min(tag_vec.len());
    let assignee_limit = max_section_rows.min(assignee_vec.len());

    // ── Build output ──

    let divider = format!("{DIM}{}{RESET}", "─".repeat(width));
    println!();
    println!("{divider}");
    println!("  {BOLD}{WHITE}TICGIT STATS{RESET}");
    println!("{divider}");
    println!();

    // Overview block
    let comment_note = if with_comments > 0 {
        format!("{DIM}, {with_comments} with comments{RESET}")
    } else {
        String::new()
    };
    println!(
        "  {BOLD}{WHITE}{total}{RESET} ticket(s){comment_note}    \
         {GREEN}{BOLD}{open}{RESET}{DIM} open{RESET}  \
         {RED}{BOLD}{closed}{RESET}{DIM} closed{RESET}"
    );
    println!();

    if two_col {
        // ── Two-column layout ──
        let col_width = (width - 4) / 2; // 2 padding on each side
        let bar_width = col_width.saturating_sub(22);

        // Left column: States + Recent Activity
        // Right column: Tags + Assignees
        let mut left_lines: Vec<String> = Vec::new();
        let mut right_lines: Vec<String> = Vec::new();

        // States
        if !state_vec.is_empty() {
            left_lines.push(format!("{CYAN}{BOLD}  States{RESET}"));
            let max_count = state_vec.iter().map(|(_, c)| *c).max().unwrap_or(1);
            for (state, count) in &state_vec {
                let bar = colored_bar(*count, max_count, bar_width, BG_CYAN);
                let color = state_color(state);
                left_lines.push(format!(
                    "    {color}{:<12}{RESET} {:>3}  {bar}",
                    state, count
                ));
            }
            left_lines.push(String::new());
        }

        // Recent activity
        if created_7d > 0 || closed_7d > 0 {
            left_lines.push(format!("{YELLOW}{BOLD}  Recent (7d){RESET}"));
            if created_7d > 0 {
                left_lines.push(format!(
                    "    {GREEN}+{created_7d}{RESET}{DIM} created{RESET}"
                ));
            }
            if closed_7d > 0 {
                left_lines.push(format!(
                    "    {RED}-{closed_7d}{RESET}{DIM} closed{RESET}"
                ));
            }
            left_lines.push(String::new());
        }

        // Tags
        if !tag_vec.is_empty() {
            right_lines.push(format!("{MAGENTA}{BOLD}  Tags{RESET}"));
            let max_count = tag_vec.first().map(|(_, c)| *c).unwrap_or(1);
            for (tag, count) in tag_vec.iter().take(tag_limit) {
                let bar = colored_bar(*count, max_count, bar_width, BG_MAGENTA);
                right_lines.push(format!(
                    "    {MAGENTA}{:<12}{RESET} {:>3}  {bar}",
                    tag, count
                ));
            }
            let remaining = tag_vec.len().saturating_sub(tag_limit);
            if remaining > 0 {
                right_lines.push(format!("    {DIM}... and {remaining} more{RESET}"));
            }
            right_lines.push(String::new());
        }

        // Assignees
        if !assignee_vec.is_empty() {
            right_lines.push(format!("{GREEN}{BOLD}  Assignees{RESET}"));
            for (name, count) in assignee_vec.iter().take(assignee_limit) {
                let bar = colored_bar(*count, state_vec.iter().map(|(_, c)| *c).max().unwrap_or(1).max(*count), bar_width, BG_GREEN);
                right_lines.push(format!(
                    "    {GREEN}{:<12}{RESET} {:>3}  {bar}",
                    name, count
                ));
            }
            right_lines.push(String::new());
        }

        // Merge columns side by side.
        let max_rows = left_lines.len().max(right_lines.len());
        for i in 0..max_rows {
            let left = left_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            let right = right_lines.get(i).map(|s| s.as_str()).unwrap_or("");
            let left_visible = visible_len(left);
            let pad = col_width.saturating_sub(left_visible);
            println!("{left}{}{right}", " ".repeat(pad));
        }
    } else {
        // ── Single-column layout ──
        let bar_width = width.saturating_sub(24);

        // States
        if !state_vec.is_empty() {
            println!("  {CYAN}{BOLD}States{RESET}");
            let max_count = state_vec.iter().map(|(_, c)| *c).max().unwrap_or(1);
            for (state, count) in &state_vec {
                let bar = colored_bar(*count, max_count, bar_width, BG_CYAN);
                let color = state_color(state);
                println!(
                    "    {color}{:<14}{RESET}{:>3}  {bar}",
                    state, count
                );
            }
            println!();
        }

        // Tags
        if !tag_vec.is_empty() {
            println!("  {MAGENTA}{BOLD}Top Tags{RESET}");
            let max_count = tag_vec.first().map(|(_, c)| *c).unwrap_or(1);
            for (tag, count) in tag_vec.iter().take(tag_limit) {
                let bar = colored_bar(*count, max_count, bar_width, BG_MAGENTA);
                println!(
                    "    {MAGENTA}{:<14}{RESET}{:>3}  {bar}",
                    tag, count
                );
            }
            let remaining = tag_vec.len().saturating_sub(tag_limit);
            if remaining > 0 {
                println!("    {DIM}... and {remaining} more{RESET}");
            }
            println!();
        }

        // Recent activity
        if created_7d > 0 || closed_7d > 0 {
            println!("  {YELLOW}{BOLD}Recent Activity (7d){RESET}");
            if created_7d > 0 {
                println!(
                    "    {GREEN}+{created_7d}{RESET}{DIM} created{RESET}"
                );
            }
            if closed_7d > 0 {
                println!(
                    "    {RED}-{closed_7d}{RESET}{DIM} closed{RESET}"
                );
            }
            println!();
        }

        // Assignees
        if !assignee_vec.is_empty() {
            println!("  {GREEN}{BOLD}Assignees{RESET}");
            for (name, count) in assignee_vec.iter().take(assignee_limit) {
                println!(
                    "    {GREEN}{:<14}{RESET}{:>3}",
                    name, count
                );
            }
            println!();
        }
    }

    println!("{divider}");

    Ok(())
}

fn state_color(state: &str) -> &'static str {
    match state {
        "new" => GREEN,
        "open" => GREEN,
        "resolved" => CYAN,
        "invalid" | "wontfix" => DIM,
        _ => WHITE,
    }
}

fn colored_bar(value: usize, max: usize, width: usize, bg: &str) -> String {
    if max == 0 || width == 0 {
        return String::new();
    }
    let filled = ((value as f64 / max as f64) * width as f64).round() as usize;
    let filled = filled.max(if value > 0 { 1 } else { 0 }).min(width);
    format!("{bg}{}{RESET}", " ".repeat(filled))
}

/// Compute the visible length of a string (ignoring ANSI escape codes).
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}
