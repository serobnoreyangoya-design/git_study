//! Editor integration — delegates to `soe` for editor resolution and capture.

use std::path::Path;

use anyhow::{Context, Result};

/// Open the best available editor with comment-prompt instructions.
/// Returns `Some(content)` on save, `None` if cancelled or empty.
pub fn capture(prompt: &str) -> Result<Option<String>> {
    soe::capture(prompt)
}

/// Like [`capture`], but pre-fills the buffer with `initial` content.
pub fn capture_with_initial(prompt: &str, initial: &str) -> Result<Option<String>> {
    soe::capture_with_initial(prompt, initial)
}

/// Read a ticket title/description body from disk. The first line is the
/// title; remaining lines become the optional description.
pub fn read_ticket_edit_file(path: &Path) -> Result<(String, Option<String>)> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading ticket content from `{}`", path.display()))?;
    parse_ticket_edit(&raw)
}

pub fn parse_ticket_edit(raw: &str) -> Result<(String, Option<String>)> {
    let mut lines = raw.lines();
    let title = lines.next().unwrap_or_default().trim().to_string();
    if title.is_empty() {
        anyhow::bail!("ticket title cannot be empty");
    }

    let description = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    let description = if description.is_empty() {
        None
    } else {
        Some(description)
    };

    Ok((title, description))
}
