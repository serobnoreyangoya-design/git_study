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
    let editor = resolve_editor();

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
    let status = spawn_editor(&editor, &path)
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

/// Spawn the editor, running through the shell so that editor values
/// like `"code --wait"` or `"emacsclient -t"` work correctly (matching
/// Git's behavior).
fn spawn_editor(editor: &str, path: &Path) -> Result<std::process::ExitStatus> {
    let path_str = path.display().to_string();
    if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", &format!("{editor} \"{path_str}\"")])
            .status()
            .map_err(Into::into)
    } else {
        Command::new("sh")
            .args(["-c", &format!("{editor} \"$@\""), "--", &path_str])
            .status()
            .map_err(Into::into)
    }
}

/// Resolve the editor using Git's precedence:
///   1. `$GIT_EDITOR`
///   2. `git config core.editor`
///   3. `$VISUAL`
///   4. `$EDITOR`
///   5. `vi` (or `notepad` on Windows)
fn resolve_editor() -> String {
    if let Ok(e) = std::env::var("GIT_EDITOR") {
        if !e.is_empty() {
            return e;
        }
    }

    if let Ok(output) = Command::new("git")
        .args(["config", "core.editor"])
        .output()
    {
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !val.is_empty() {
                return val;
            }
        }
    }

    if let Ok(e) = std::env::var("VISUAL") {
        if !e.is_empty() {
            return e;
        }
    }

    if let Ok(e) = std::env::var("EDITOR") {
        if !e.is_empty() {
            return e;
        }
    }

    if cfg!(windows) {
        "notepad".to_string()
    } else {
        "vi".to_string()
    }
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
