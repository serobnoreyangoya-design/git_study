# Creating and Browsing Tickets

Commands: `ti new`, `ti list`, `ti show`, `ti recent`, `ti tui`

## Creating tickets

### Interactive (editor)

```
$ ti new
```

Opens your `$EDITOR`. First line becomes the title, everything after a blank line becomes the description.

### Inline

```
$ ti new --title "fix the parser" --tags bug
Created: a3f29c
```

### With all the trimmings

```
$ ti new --title "refactor auth middleware" \
    --tags "refactor,security" \
    --assigned alice@example.com \
    --comment "Blocked on the new token spec, but let's get it on the board"
Created: 7b2e4f
```

### From a file

Write a file where line 1 is the title and the rest (after a blank line) is the description:

```
$ cat ticket.txt
Migrate to new token format

The old JWT tokens use HS256 which doesn't meet the new
compliance requirements. We need to switch to RS256 and
rotate all existing sessions.

$ ti new --file ticket.txt --tags "security,compliance"
Created: d91c3a
```

### Machine-friendly

```
$ ti new --title "automated ticket" --id-only
d91c3a84-7f2d-4e1b-a180-0a33fb090d59

$ ti new --title "another one" --json
{"id":"e5f7a2...","title":"another one","status":"open",...}
```

## Listing tickets

### Default view (open tickets)

```
$ ti list
  TicId  Date   Title                         Status State       Assgn    Tags
----------------------------------------------------------------------------------------------------
  3d3a5d 0d     Add spec field to tickets ... open   new                 feature
  e416bd 0d     Simplify views functionality  open   new                 feature
  9722d2 0d     Implement sub-tickets (par... open   new                 feature
  9b8039 23h    Implement `ti context <id>... open   new                 agent,context,fea...
  5d607a 23h    Link tickets to branches, ... open   new                 agent,feature,git
```

By default, `ti list` shows open tickets, newest first, capped at 20.

### Show everything

```
$ ti list --all
```

Removes the open-only filter and the 20-ticket limit.

### Filter by state

```
$ ti list --state in-progress
$ ti list --state blocked
$ ti list --status closed
```

State accepts any lifecycle value: `new`, `assigned`, `in-progress`, `blocked`, `review`, `resolved`, `wontfix`, `duplicate`, `invalid`. Status accepts `open` or `closed`.

### Filter by tag or assignee

```
$ ti list --tag bug
$ ti list --assigned alice@example.com
$ ti list --only-tagged
```

### Search

Search across title, description, and comments:

```
$ ti list --search "parser"
```

Scope to a specific field:

```
$ ti list --search "title:parser"
$ ti list --search "comments:workaround"
$ ti list --search "description:migration"
```

### Sort

```
$ ti list --order title
$ ti list --order created.desc
$ ti list --order state
$ ti list --order assigned.desc
```

### Combine filters

```
$ ti list --tag bug --state in-progress --order created.desc
```

## Showing a ticket

```
$ ti show 3d3a5d
-----------------------------------------------------------
Title   : Add spec field to tickets for implementation specifications
Id      : 3d3a5d72-109d-47fa-a91d-0c7ff5d00b4a
Created : 2026-05-09 (0d)  by schacon@gmail.com
Status  : open
State   : new
Tags    : feature
Description:
  none
-----------------------------------------------------------

schacon@gmail.com - 2026-05-09T10:46:38.649Z
  Add a top-level 'spec' field (like title and description) that holds
  the implementation specification for a ticket, kept separate from the
  description. Description is the what/why, spec is the how.
```

If you have a ticket checked out, just `ti show` with no argument.

## Recent tickets

Show the last N tickets touched (by any field change):

```
$ ti recent
$ ti recent -n 5
```

## Interactive browser

```
$ ti tui
```

Launches a full-screen terminal UI for browsing and navigating tickets with keyboard controls.
