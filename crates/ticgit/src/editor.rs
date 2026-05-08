//! `$EDITOR` integration for capturing multi-line input from the user.

use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// Open `$EDITOR` (or `vi` / `notepad`) with `prompt` as the initial body
/// and return the user's saved content with the prompt comment lines
/// stripped. Returns `None` if the user saved an empty buffer.
pub fn capture(prompt: &str) -> Result<Option<String>> {
    capture_with_initial(prompt, "")
}

/// Open `$EDITOR` with editable initial content followed by comment-only
/// instructions. Lines beginning with `#` are stripped from the result.
pub fn capture_with_initial(prompt: &str, initial: &str) -> Result<Option<String>> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".to_string()
        } else {
            "vi".to_string()
        }
    });

    let mut tf = tempfile::Builder::new()
        .prefix("ticgit-")
        .suffix(".md")
        .tempfile()
        .context("creating editor tempfile")?;

    if !initial.is_empty() {
        write!(tf, "{initial}").context("writing initial content to tempfile")?;
        if !initial.ends_with('\n') {
            writeln!(tf).context("terminating initial content")?;
        }
        writeln!(tf).ok();
    }

    for line in prompt.lines() {
        writeln!(tf, "# {line}").context("writing prompt to tempfile")?;
    }
    writeln!(tf).ok();
    tf.flush().context("flushing prompt")?;

    let path = tf.path().to_path_buf();
    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("spawning editor `{editor}`"))?;
    if !status.success() {
        anyhow::bail!("editor `{editor}` exited with status {status}");
    }

    let mut contents = String::new();
    std::fs::File::open(&path)
        .context("re-opening editor tempfile")?
        .read_to_string(&mut contents)
        .context("reading editor tempfile")?;

    let cleaned: String = contents
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();

    Ok(if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    })
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
