//! Terminal output: tables, single-ticket details, and JSON.

use std::collections::BTreeMap;

use ticgit_lib::Ticket;
use ticgit_lib::TicketState;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_DIM: &str = "\x1b[2m";
const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_PURPLE: &str = "\x1b[35m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_CYAN: &str = "\x1b[36m";

/// Render a list of tickets as a compact table. `current` (if any) gets a `*`.
pub fn tickets_table(tickets: &[Ticket], current: Option<&uuid::Uuid>) -> String {
    let ref_lengths = open_ticket_ref_lengths(tickets);
    tickets_table_with_refs(tickets, current, &ref_lengths)
}

/// Render a list with caller-provided open-ticket short reference lengths.
pub fn tickets_table_with_refs(
    tickets: &[Ticket],
    current: Option<&uuid::Uuid>,
    ref_lengths: &BTreeMap<uuid::Uuid, usize>,
) -> String {
    let width = crossterm::terminal::size()
        .map(|(columns, _)| columns as usize)
        .unwrap_or(100)
        .max(40);
    tickets_table_with_width(
        tickets,
        current,
        ref_lengths,
        width,
        OffsetDateTime::now_utc(),
    )
}

pub fn open_ticket_ref_lengths(tickets: &[Ticket]) -> BTreeMap<uuid::Uuid, usize> {
    let open_hexes: Vec<_> = tickets
        .iter()
        .filter(|ticket| ticket.state == TicketState::Open)
        .map(|ticket| (ticket.id, ticket.id.to_string().replace('-', "")))
        .collect();

    open_hexes
        .iter()
        .map(|(id, hex)| {
            let length = (1..=hex.len())
                .find(|length| {
                    let prefix = &hex[..*length];
                    open_hexes
                        .iter()
                        .filter(|(_, other)| other.starts_with(prefix))
                        .count()
                        == 1
                })
                .unwrap_or(hex.len());
            (*id, length)
        })
        .collect()
}

fn tickets_table_with_width(
    tickets: &[Ticket],
    current: Option<&uuid::Uuid>,
    ref_lengths: &BTreeMap<uuid::Uuid, usize>,
    width: usize,
    now: OffsetDateTime,
) -> String {
    let id_width = ref_lengths.values().copied().max().unwrap_or(6).max(6);
    const STATE_WIDTH: usize = 5;
    const DATE_WIDTH: usize = 5;
    const ASSIGNED_WIDTH: usize = 8;
    const TAGS_WIDTH: usize = 20;
    const GAPS_AND_MARKER: usize = 15;
    const MIN_TITLE_WIDTH: usize = 12;

    let fixed_width =
        id_width + STATE_WIDTH + DATE_WIDTH + ASSIGNED_WIDTH + TAGS_WIDTH + GAPS_AND_MARKER;
    let title_width = width.saturating_sub(fixed_width).max(MIN_TITLE_WIDTH);

    let mut out = String::new();
    let header = format!(
        "  {} {}  {} {} {} {}",
        fit("TicId", id_width),
        fit("Date", DATE_WIDTH),
        fit("Title", title_width),
        fit("State", STATE_WIDTH),
        fit("Assgn", ASSIGNED_WIDTH),
        fit("Tags", TAGS_WIDTH)
    );
    out.push_str(&ansi(ANSI_DIM, &header));
    out.push('\n');
    out.push_str(&ansi(ANSI_DIM, &"-".repeat(width)));
    out.push('\n');

    for t in tickets {
        let marker = if Some(&t.id) == current { "*" } else { " " };
        let assigned = t.assigned_short().unwrap_or_default();
        let tags = t.tags.iter().cloned().collect::<Vec<_>>().join(",");
        out.push_str(marker);
        out.push(' ');
        out.push_str(&styled_ticket_id(t, id_width, ref_lengths));
        out.push(' ');
        out.push_str(&ansi(
            ANSI_DIM,
            &fit(&relative_date(t.created_at, now), DATE_WIDTH),
        ));
        out.push_str("  ");
        out.push_str(&ansi(ANSI_BLUE, &fit(&flatten(&t.title), title_width)));
        out.push(' ');
        out.push_str(&ansi(
            state_color(t.state.as_str()),
            &fit(t.state.as_str(), STATE_WIDTH),
        ));
        out.push_str(&fit(&flatten(&assigned), ASSIGNED_WIDTH));
        out.push(' ');
        out.push_str(&ansi(ANSI_YELLOW, &fit(&flatten(&tags), TAGS_WIDTH)));
        out.push('\n');
    }

    out
}

/// Render a single ticket and its comments.
pub fn ticket_detail(t: &Ticket) -> String {
    let mut out = String::new();
    let title_bar = "-".repeat(t.title.chars().count().max(20));
    out.push_str(&ansi(ANSI_DIM, &title_bar));
    out.push('\n');
    out.push_str(&detail_field("Title", &ansi(ANSI_BLUE, &t.title)));
    out.push_str(&detail_field("Id", &ansi(ANSI_CYAN, &t.id.to_string())));
    out.push_str(&detail_field(
        "Created",
        &ansi(
            ANSI_DIM,
            &format!(
                "{} ({})  by {}",
                friendly_date(t.created_at),
                relative_date(t.created_at, OffsetDateTime::now_utc()),
                t.created_by
            ),
        ),
    ));
    out.push_str(&detail_field(
        "State",
        &ansi(state_color(t.state.as_str()), t.state.as_str()),
    ));
    if let Some(a) = &t.assigned {
        out.push_str(&detail_field("Assigned", a));
    }
    if let Some(p) = t.points {
        out.push_str(&detail_field("Points", &p.to_string()));
    }
    if let Some(m) = &t.milestone {
        out.push_str(&detail_field("Milestone", m));
    }
    if !t.tags.is_empty() {
        let tags: Vec<_> = t.tags.iter().cloned().collect();
        out.push_str(&detail_field("Tags", &ansi(ANSI_YELLOW, &tags.join(", "))));
    }
    out.push_str(&ansi(ANSI_YELLOW, "Description:"));
    out.push('\n');
    match t.description.as_deref().filter(|d| !d.trim().is_empty()) {
        Some(description) => {
            out.push('\n');
            out.push_str(&format!("  {}\n", description.replace('\n', "\n  ")));
        }
        None => {
            out.push_str("  ");
            out.push_str(&ansi(ANSI_DIM, "none"));
            out.push('\n');
        }
    }
    out.push_str(&ansi(ANSI_DIM, &title_bar));
    out.push('\n');

    if t.comments.is_empty() {
        out.push_str(&ansi(ANSI_DIM, "(no comments)"));
        out.push('\n');
    } else {
        for c in &t.comments {
            out.push_str(&format!(
                "\n{} {} {}\n  {}\n",
                ansi(ANSI_CYAN, &c.author),
                ansi(ANSI_DIM, "-"),
                ansi(ANSI_DIM, &c.at.format(&Rfc3339).unwrap_or_default()),
                c.body.replace('\n', "\n  "),
            ));
        }
    }
    out
}

/// Render a single ticket as JSON (for scripting).
pub fn ticket_json(t: &Ticket) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(t)
}

pub fn tickets_json(t: &[Ticket]) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(t)
}

fn fit(value: &str, width: usize) -> String {
    let truncated = truncate_display(value, width);
    let padding = width.saturating_sub(UnicodeWidthStr::width(truncated.as_str()));
    format!("{truncated}{}", " ".repeat(padding))
}

fn styled_ticket_id(
    ticket: &Ticket,
    width: usize,
    ref_lengths: &BTreeMap<uuid::Uuid, usize>,
) -> String {
    let hex = ticket.id.to_string().replace('-', "");
    let display_len = ref_lengths.get(&ticket.id).copied().unwrap_or(6).max(6);
    let visible: String = hex.chars().take(display_len).collect();
    let reference_len = ref_lengths
        .get(&ticket.id)
        .copied()
        .unwrap_or(6)
        .min(visible.len());
    let (reference, rest) = visible.split_at(reference_len);
    let padding = width.saturating_sub(visible.len());

    format!(
        "{}{}{}",
        ansi(ANSI_YELLOW, reference),
        ansi(ANSI_CYAN, rest),
        " ".repeat(padding)
    )
}

fn truncate_display(value: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(value) <= max_width {
        return value.to_string();
    }

    let ellipsis = if max_width > 3 { "..." } else { "." };
    let ellipsis_width = UnicodeWidthStr::width(ellipsis);
    let content_width = max_width.saturating_sub(ellipsis_width);
    let mut out = String::new();
    let mut width = 0;
    for ch in value.chars() {
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + char_width > content_width {
            break;
        }
        out.push(ch);
        width += char_width;
    }
    out.push_str(ellipsis);
    out
}

fn flatten(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn ansi(color: &str, value: &str) -> String {
    format!("{color}{value}{ANSI_RESET}")
}

fn detail_field(label: &str, value: &str) -> String {
    format!(
        "{}{} {value}\n",
        ansi(ANSI_YELLOW, &format!("{label:<8}")),
        ansi(ANSI_DIM, ":")
    )
}

fn state_color(state: &str) -> &'static str {
    match state {
        "open" => ANSI_GREEN,
        "hold" => ANSI_YELLOW,
        "resolved" | "invalid" => ANSI_PURPLE,
        _ => ANSI_DIM,
    }
}

fn relative_date(then: OffsetDateTime, now: OffsetDateTime) -> String {
    let seconds = (now - then).whole_seconds().max(0);
    if seconds < 60 * 60 {
        return "0d".to_string();
    }
    if seconds < 60 * 60 * 24 {
        return format!("{}h", seconds / (60 * 60));
    }
    if seconds < 60 * 60 * 24 * 30 {
        return format!("{}d", seconds / (60 * 60 * 24));
    }
    if seconds < 60 * 60 * 24 * 365 {
        return format!("{}mo", seconds / (60 * 60 * 24 * 30));
    }
    format!("{}y", seconds / (60 * 60 * 24 * 365))
}

fn friendly_date(when: OffsetDateTime) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        when.year(),
        u8::from(when.month()),
        when.day()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use uuid::Uuid;

    #[test]
    fn open_ticket_refs_ignore_closed_ticket_collisions() {
        let open = ticket(
            "d7f2d8f6-d6ec-3da1-a180-0a33fb090d59",
            "open",
            TicketState::Open,
        );
        let other_open = ticket(
            "d7a2d8f6-d6ec-3da1-a180-0a33fb090d59",
            "other",
            TicketState::Open,
        );
        let closed = ticket(
            "d7f99999-d6ec-3da1-a180-0a33fb090d59",
            "closed",
            TicketState::Resolved,
        );

        let refs = open_ticket_ref_lengths(&[open.clone(), other_open, closed]);

        assert_eq!(refs.get(&open.id), Some(&3));
    }

    fn ticket(id: &str, title: &str, state: TicketState) -> Ticket {
        Ticket {
            id: Uuid::parse_str(id).unwrap(),
            title: title.to_string(),
            description: None,
            state,
            assigned: None,
            points: None,
            milestone: None,
            tags: BTreeSet::new(),
            comments: Vec::new(),
            created_at: OffsetDateTime::UNIX_EPOCH,
            created_by: "tester@example.com".to_string(),
        }
    }
}
