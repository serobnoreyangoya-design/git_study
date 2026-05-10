pub const MARKDOWN: &str = r#"---
name: ticgit
description: Use TicGit (`ti`) to track local-first, Git-native tickets in this repository. Use when creating, listing, editing, triaging, updating, syncing, or resolving tickets.
---

# TicGit Agent Guide

TicGit stores tickets as Git metadata, so ticket changes travel with the repository through `ti sync`. Prefer Markdown output (`--markdown`) when reading ticket data for agent workflows; it includes the ticket data plus suggested next commands.

## Core Workflow

1. Inspect open work:

```sh
ti list
ti list --markdown
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
ti state blocked
ti state open
ti state closed
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

Use `--markdown` when you need the created ticket details and suggested next commands:

```sh
ti new -F /tmp/ticket.md --markdown
```

## Read Tickets

List open tickets by default:

```sh
ti list
```

Useful filters:

```sh
ti list --all
ti list --status open
ti list --state blocked
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
ti show <id> --markdown
```

Extract a single JSON field:

```sh
ti show <id> --filter .title
ti show <id> --filter .comments[0].body
```

## Machine Output Schema

The stable JSON schema for `ti show --json`, JSON mutation commands, and `ti list --json` is published at:

```text
https://ticgit.dev/schema/v1.json
docs/schema/v1.json
```

`ti show --json` emits a ticket object. `ti list --json` emits an array of ticket objects. Ticket metadata appears under `.meta` as a string-to-string object.

Machine-mode guarantees for `--json`:

- successful JSON commands write parseable JSON to stdout only
- diagnostic and error text goes to stderr
- JSON output does not include ANSI color escapes
- non-zero exit status means the command failed
- ticket ids may be full UUIDs or unique UUID prefixes
- ambiguous or missing prefixes fail with a non-zero exit status and stderr diagnostic

`--porcelain` and `--format json` are not supported compatibility aliases today; use `--json` for schema-stable output.

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

Tickets have a broad `status` and a specific `state`.

```text
status=open   states: new, assigned, in-progress, blocked, review
status=closed states: resolved, wontfix, duplicate, invalid
```

New tickets start as `open:new`. `ti state` and `ti status` accept either a status, a state, or an explicit `status:state` pair.

```sh
ti state open -t <id>              # open:new
ti state blocked -t <id>           # open:blocked
ti state closed -t <id>            # closed:resolved
ti state closed:wontfix -t <id>    # closed:wontfix
ti status review -t <id>           # open:review
```

Use `blocked` for paused or blocked work. Use `review` when implementation is ready for review. Use `closed:resolved` when implementation is complete.
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

Store structured string metadata under a ticket. Metadata appears in `ti show --markdown` and under `.meta` in `ti show --json`.

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

## Planning With Specs

Use the `spec` field to write an implementation plan before starting work. The spec is a top-level ticket field separate from the description — the description says *what/why*, the spec says *how*.

```sh
# Write a spec inline
ti spec -t <id> "Use RS256 tokens with 24h expiry, rotate via cron"

# Write a spec from a file (good for multi-line plans)
ti spec -t <id> -F /tmp/plan.md

# Open $EDITOR for the spec
ti spec -t <id>

# Read the spec
ti show <id> --filter .spec

# Clear the spec
ti spec -t <id> --clear
```

When picking up a ticket, check if a spec exists. If not, write one before coding. A good spec covers: approach, files to change, edge cases, and how to verify.

## Pick Next Work

Use `ti next` to automatically select and check out the highest-priority open ticket:

```sh
ti next
ti next --tag bug
ti next --assigned alice@example.com
ti next --markdown
```

`ti next` scores tickets by state (in-progress > assigned > new), assignment, points, and age. It skips sub-issues and tickets with unresolved dependencies.

## Dependencies

Mark tickets that must be completed before another can start:

```sh
ti depends <blocker-id> -t <id>      # <id> depends on <blocker-id>
ti depends <blocker-id> -t <id> --remove
ti depends --clear -t <id>
```

Dependencies show as `Depends:` and `Blocks:` in `ti show`. Circular dependencies are rejected. `ti next` will not pick tickets with open dependencies.

## Code URIs

Link a ticket to the branch where work is happening:

```sh
ti code https://github.com/owner/repo:feature-branch -t <id>
ti code --clear -t <id>
```

## Agent Practices

- Prefer `--markdown` for commands that support it; use `--json` only when a script needs stable schema output.
- Use ticket ids or unique prefixes; ambiguous prefixes fail.
- Run `ti checkout <id>` before multi-step work so later commands can omit `-t <id>`.
- Before starting work, read the spec (`ti show <id> --filter .spec`). If none exists, write one with `ti spec`.
- Add comments for meaningful observations, plans, blockers, and results.
- Keep tags short and queryable, such as `bug`, `feature`, `docs`, `parser`, or `agent`.
- Use `ti next` to pick work rather than scanning the full list.
- Set dependencies with `ti depends` when tickets have ordering constraints.
- Resolve tickets only after code changes and relevant verification are complete.
"#;

pub fn print() {
    println!("{MARKDOWN}");
}
