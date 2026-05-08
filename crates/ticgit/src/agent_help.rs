pub const MARKDOWN: &str = r#"---
name: ticgit
description: Use TicGit (`ti`) to track local-first, Git-native tickets in this repository. Use when creating, listing, editing, triaging, updating, syncing, or resolving tickets.
---

# TicGit Agent Guide

TicGit stores tickets as Git metadata, so ticket changes travel with the repository through `ti sync`. Prefer machine-readable output (`--json`) when reading ticket data for automation.

## Core Workflow

1. Inspect open work:

```sh
ti list
ti list --json
ti recent
```

2. Select a ticket as current when you will run several commands against it:

```sh
ti checkout <id>
ti show
ti comment "progress update"
ti checkout --clear
```

3. Update the ticket as work progresses:

```sh
ti state hold
ti state open
ti state resolved
ti close
```

## Create Tickets

Create a simple ticket:

```sh
ti new --title "Fix parser panic" --tags bug,parser
```

Create a ticket from a file. The first line is the title; remaining non-empty content is the description:

```sh
ti new -F /tmp/ticket.md --tags feature,agent
```

Use `--json` when you need the created ticket id:

```sh
ti new -F /tmp/ticket.md --json
```

## Read Tickets

List open tickets by default:

```sh
ti list
```

Useful filters:

```sh
ti list --all
ti list --state hold
ti list --tag bug
ti list --assigned alice@example.com
ti list --only-tagged
ti list --order created.desc
ti list --limit 50
```

Search syntax:

```sh
# Search title, description, and comments.
ti list --search parser

# Search one field.
ti list --search title:parser
ti list --search description:recovery
ti list --search comments:failed
```

Show one ticket, or the current ticket if one is checked out:

```sh
ti show <id>
ti show
ti show <id> --json
```

Extract a single JSON field:

```sh
ti show <id> --filter .title
ti show <id> --filter .comments[0].body
```

## Edit Tickets

Edit title and description in `$EDITOR`:

```sh
ti edit <id>
```

Edit title and description from a file:

```sh
ti edit <id> -F /tmp/ticket.md
```

File format:

```text
Updated title

Updated description.
Additional description lines are preserved.
```

## Comments And Progress Notes

Add a short comment:

```sh
ti comment -t <id> "reproduced locally"
```

Use the current ticket:

```sh
ti checkout <id>
ti comment "implemented parser guard; running tests next"
```

Open `$EDITOR` for longer comments:

```sh
ti comment -t <id> --edit
```

## State And Triage

Supported states are `open`, `resolved`, `invalid`, and `hold`.

```sh
ti state open -t <id>
ti state hold -t <id>
ti state resolved -t <id>
ti state invalid -t <id>
```

Use `hold` for blocked or paused work. Use `resolved` when the implementation is complete.
Use `ti close` as a shortcut for resolving a ticket; if the closed ticket is checked out, it also clears the checkout.

## Tags, Assignment, Estimates, Milestones

Tags are comma- or space-separated:

```sh
ti tag -t <id> bug,parser
ti tag -t <id> --remove bug
```

Set or clear ownership and planning fields:

```sh
ti assign -t <id> alice@example.com
ti assign -t <id> --clear
ti points -t <id> 3
ti points -t <id> --clear
ti milestone -t <id> v1.0
ti milestone -t <id> --clear
```

## Metadata

Store structured string metadata under a ticket. Metadata appears in `ti show` and under `.meta` in `ti show --json`.

```sh
ti meta -t <id> branch feature/parser-fix
ti meta -t <id> test-command "cargo test -p ticgit"
ti meta -t <id> notes -F /tmp/meta-value.txt
ti show <id> --filter .meta.branch
```

## Saved Views

Save filtered lists for repeatable queues:

```sh
ti save-view bugs --tag bug
ti list --view bugs
ti views
ti views bugs
```

## Sync

Ticket metadata is separate from normal Git commits. Sync it explicitly when collaborating:

```sh
ti sync
```

## Agent Practices

- Prefer `--json` for commands that support it.
- Use ticket ids or unique prefixes; ambiguous prefixes fail.
- Run `ti checkout <id>` before multi-step work so later commands can omit `-t <id>`.
- Add comments for meaningful observations, plans, blockers, and results.
- Keep tags short and queryable, such as `bug`, `feature`, `docs`, `parser`, or `agent`.
- Resolve tickets only after code changes and relevant verification are complete.
"#;

pub fn print() {
    println!("{MARKDOWN}");
}
