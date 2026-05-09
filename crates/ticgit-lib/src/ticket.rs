//! Domain types for tickets, comments, and ticket states.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{Error, Result};

/// Broad lifecycle bucket for a ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TicketStatus {
    Open,
    Closed,
}

impl TicketStatus {
    pub const ALL: &'static [TicketStatus] = &[TicketStatus::Open, TicketStatus::Closed];

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketStatus::Open => "open",
            TicketStatus::Closed => "closed",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match normalize_lifecycle_value(s).as_str() {
            "open" => Ok(TicketStatus::Open),
            "closed" => Ok(TicketStatus::Closed),
            other => Err(Error::InvalidStatus(other.to_string())),
        }
    }
}

impl fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Specific lifecycle state. Its allowed values depend on [`TicketStatus`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TicketState {
    #[serde(rename = "new")]
    New,
    #[serde(rename = "assigned")]
    Assigned,
    #[serde(rename = "in-progress")]
    InProgress,
    #[serde(rename = "blocked")]
    Blocked,
    #[serde(rename = "review")]
    Review,
    #[serde(rename = "resolved")]
    Resolved,
    #[serde(rename = "wontfix")]
    Wontfix,
    #[serde(rename = "duplicate")]
    Duplicate,
    #[serde(rename = "invalid")]
    Invalid,
}

impl TicketState {
    pub const OPEN: &'static [TicketState] = &[
        TicketState::New,
        TicketState::Assigned,
        TicketState::InProgress,
        TicketState::Blocked,
        TicketState::Review,
    ];
    pub const CLOSED: &'static [TicketState] = &[
        TicketState::Resolved,
        TicketState::Wontfix,
        TicketState::Duplicate,
        TicketState::Invalid,
    ];
    pub const ALL: &'static [TicketState] = &[
        TicketState::New,
        TicketState::Assigned,
        TicketState::InProgress,
        TicketState::Blocked,
        TicketState::Review,
        TicketState::Resolved,
        TicketState::Wontfix,
        TicketState::Duplicate,
        TicketState::Invalid,
    ];

    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketState::New => "new",
            TicketState::Assigned => "assigned",
            TicketState::InProgress => "in-progress",
            TicketState::Blocked => "blocked",
            TicketState::Review => "review",
            TicketState::Resolved => "resolved",
            TicketState::Wontfix => "wontfix",
            TicketState::Duplicate => "duplicate",
            TicketState::Invalid => "invalid",
        }
    }

    pub fn parse(s: &str) -> Result<Self> {
        match normalize_lifecycle_value(s).as_str() {
            "new" => Ok(TicketState::New),
            "assigned" => Ok(TicketState::Assigned),
            "in-progress" | "inprogress" => Ok(TicketState::InProgress),
            "blocked" | "hold" => Ok(TicketState::Blocked),
            "review" => Ok(TicketState::Review),
            "resolved" => Ok(TicketState::Resolved),
            "wontfix" | "wont-fix" | "wont_fix" => Ok(TicketState::Wontfix),
            "duplicate" => Ok(TicketState::Duplicate),
            "invalid" => Ok(TicketState::Invalid),
            other => Err(Error::InvalidState(other.to_string())),
        }
    }

    #[must_use]
    pub fn status(self) -> TicketStatus {
        if Self::OPEN.contains(&self) {
            TicketStatus::Open
        } else {
            TicketStatus::Closed
        }
    }
}

impl fmt::Display for TicketState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TicketLifecycle {
    pub status: TicketStatus,
    pub state: TicketState,
}

impl TicketLifecycle {
    #[must_use]
    pub fn new(status: TicketStatus, state: TicketState) -> Option<Self> {
        (state.status() == status).then_some(Self { status, state })
    }

    pub fn parse(spec: &str) -> Result<Self> {
        let spec = spec.trim();
        if let Some((status, state)) = spec.split_once(':') {
            let status = TicketStatus::parse(status)?;
            let state = TicketState::parse(state)?;
            return Self::new(status, state).ok_or_else(|| {
                Error::InvalidState(format!(
                    "{spec} (state `{}` does not belong to status `{}`)",
                    state.as_str(),
                    status.as_str()
                ))
            });
        }

        if let Ok(status) = TicketStatus::parse(spec) {
            let state = match status {
                TicketStatus::Open => TicketState::New,
                TicketStatus::Closed => TicketState::Resolved,
            };
            return Ok(Self { status, state });
        }

        let state = TicketState::parse(spec)?;
        Ok(Self {
            status: state.status(),
            state,
        })
    }
}

/// A single comment on a ticket.
///
/// `at` and `author` are recovered from the underlying git-meta `ListEntry`'s
/// timestamp and the JSON body we store in `ListEntry::value`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub author: String,
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub body: String,
}

/// On-the-wire shape of a comment list entry. We JSON-encode this as
/// the `value` of a git-meta `ListEntry`; the timestamp lives on the
/// `ListEntry` itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CommentBody {
    pub author: String,
    pub body: String,
}

/// A ticket, fully hydrated from project-target metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticket {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub spec: Option<String>,
    pub status: TicketStatus,
    pub state: TicketState,
    pub assigned: Option<String>,
    pub points: Option<i64>,
    pub milestone: Option<String>,
    pub tags: BTreeSet<String>,
    pub meta: BTreeMap<String, String>,
    pub comments: Vec<Comment>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    pub created_by: String,
}

impl Ticket {
    /// Short 6-char form of the UUID used in table output and as a
    /// human-friendly handle (e.g. `d7f2d8`).
    #[must_use]
    pub fn short_id(&self) -> String {
        let s = self.id.to_string();
        s.chars().take(6).collect()
    }

    /// The "@user" portion of an email-style assigned handle, or the
    /// raw value if it doesn't look like an email.
    #[must_use]
    pub fn assigned_short(&self) -> Option<String> {
        self.assigned.as_ref().map(|a| {
            a.split_once('@')
                .map(|(local, _)| local.to_string())
                .unwrap_or_else(|| a.clone())
        })
    }
}

fn normalize_lifecycle_value(value: &str) -> String {
    value.trim().to_ascii_lowercase().replace('_', "-")
}

/// Options accepted by [`crate::store::TicketStore::create`].
#[derive(Debug, Clone, Default)]
pub struct NewTicketOpts {
    pub comment: Option<String>,
    pub tags: Vec<String>,
    pub assigned: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_state_parse_round_trip() {
        for state in TicketState::ALL {
            assert_eq!(TicketState::parse(state.as_str()).unwrap(), *state);
        }
    }

    #[test]
    fn ticket_status_parse_round_trip() {
        for status in TicketStatus::ALL {
            assert_eq!(TicketStatus::parse(status.as_str()).unwrap(), *status);
        }
    }

    #[test]
    fn lifecycle_parse_accepts_status_state_and_combined_specs() {
        assert_eq!(
            TicketLifecycle::parse("closed").unwrap(),
            TicketLifecycle {
                status: TicketStatus::Closed,
                state: TicketState::Resolved
            }
        );
        assert_eq!(
            TicketLifecycle::parse("closed:wontfix").unwrap(),
            TicketLifecycle {
                status: TicketStatus::Closed,
                state: TicketState::Wontfix
            }
        );
        assert_eq!(
            TicketLifecycle::parse("in_progress").unwrap(),
            TicketLifecycle {
                status: TicketStatus::Open,
                state: TicketState::InProgress
            }
        );
        assert!(TicketLifecycle::parse("open:wontfix").is_err());
    }

    #[test]
    fn ticket_state_parse_rejects_garbage() {
        assert!(TicketState::parse("frob").is_err());
        assert!(TicketState::parse("").is_err());
    }

    #[test]
    fn short_id_is_six_chars() {
        let t = Ticket {
            id: Uuid::parse_str("d7f2d8f6-d6ec-3da1-a180-0a33fb090d59").unwrap(),
            title: "x".into(),
            description: None,
            spec: None,
            status: TicketStatus::Open,
            state: TicketState::New,
            assigned: None,
            points: None,
            milestone: None,
            tags: BTreeSet::new(),
            meta: BTreeMap::new(),
            comments: vec![],
            created_at: OffsetDateTime::UNIX_EPOCH,
            created_by: "x".into(),
        };
        assert_eq!(t.short_id(), "d7f2d8");
    }

    #[test]
    fn assigned_short_strips_email_domain() {
        let t = Ticket {
            id: Uuid::nil(),
            title: "x".into(),
            description: None,
            spec: None,
            status: TicketStatus::Open,
            state: TicketState::New,
            assigned: Some("jeff.welling@gmail.com".into()),
            points: None,
            milestone: None,
            tags: BTreeSet::new(),
            meta: BTreeMap::new(),
            comments: vec![],
            created_at: OffsetDateTime::UNIX_EPOCH,
            created_by: "x".into(),
        };
        assert_eq!(t.assigned_short().as_deref(), Some("jeff.welling"));
    }

    #[test]
    fn assigned_short_passes_through_non_email() {
        let t = Ticket {
            id: Uuid::nil(),
            title: "x".into(),
            description: None,
            spec: None,
            status: TicketStatus::Open,
            state: TicketState::New,
            assigned: Some("jdoe".into()),
            points: None,
            milestone: None,
            tags: BTreeSet::new(),
            meta: BTreeMap::new(),
            comments: vec![],
            created_at: OffsetDateTime::UNIX_EPOCH,
            created_by: "x".into(),
        };
        assert_eq!(t.assigned_short().as_deref(), Some("jdoe"));
    }
}
