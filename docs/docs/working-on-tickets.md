# Working on Tickets

Commands: `ti checkout`, `ti edit`, `ti comment`, `ti state`, `ti close`

## Checkout

Select a ticket as your "current" ticket. Most commands default to the checked-out ticket when you don't pass `--ticket`.

```
$ ti checkout a3f
Checked out: a3f29c  fix the parser

$ ti show
(shows a3f29c without needing to specify it)

$ ti comment "working on this now"
(comment added to a3f29c)
```

Clear the checkout:

```
$ ti checkout --clear
```

The checked-out ticket is marked with `*` in `ti list` output.

## Edit title and description

Open the ticket in your editor to change the title and description:

```
$ ti edit a3f
```

Or from a file:

```
$ ti edit --file updated.txt
```

The file format is the same as `ti new --file`: first line is the title, blank line, then description.

## Comments

### Inline

```
$ ti comment "found the root cause, it's in the tokenizer"
```

```
$ ti comment --ticket a3f "this is specific to ticket a3f"
```

### Editor

```
$ ti comment
```

Opens `$EDITOR` for longer comments. You can also force the editor even when passing text:

```
$ ti comment --edit
```

## Lifecycle: status and state

Tickets have two lifecycle dimensions:

**Status** -- broad bucket:
- `open` -- active work
- `closed` -- done

**State** -- specific position in the lifecycle:

| Open states      | Closed states |
|------------------|---------------|
| `new`            | `resolved`    |
| `assigned`       | `wontfix`     |
| `in-progress`    | `duplicate`   |
| `blocked`        | `invalid`     |
| `review`         |               |

### Change state

```
$ ti state in-progress
$ ti state blocked
$ ti state review
```

State automatically sets the correct status (e.g., `resolved` sets status to `closed`).

### Be explicit about both

```
$ ti state closed:wontfix
$ ti state open:blocked
```

### Close a ticket

`ti close` is shorthand for `ti state resolved`:

```
$ ti close a3f
Closed: a3f29c  fix the parser
```

### Common workflow

```
$ ti new --title "fix parser panic on empty input" --tags bug
Created: a3f29c

$ ti checkout a3f

$ ti state assigned
$ ti assign alice@example.com

$ ti state in-progress
$ ti comment "reproducing locally"

$ ti comment "fix is in commit e7a2b1f"
$ ti state review

$ ti close
$ ti sync
```
