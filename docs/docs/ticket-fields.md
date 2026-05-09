# Ticket Fields

Commands: `ti tag`, `ti assign`, `ti points`, `ti milestone`, `ti meta`, `ti edit`

Every ticket has a set of built-in fields. These commands let you set them without opening an editor.

## Tags

Add tags:

```
$ ti tag bug
$ ti tag "bug,urgent"
$ ti tag --ticket a3f security compliance
```

Remove tags:

```
$ ti tag --remove bug
$ ti tag -d urgent
```

Tags are useful for filtering:

```
$ ti list --tag bug
$ ti list --only-tagged
```

## Assign

Set the assignee (typically an email):

```
$ ti assign alice@example.com
$ ti assign --ticket a3f bob@example.com
```

Clear the assignee:

```
$ ti assign --clear
```

In list output, the email domain is stripped for readability (`alice@example.com` shows as `alice`).

Filter by assignee:

```
$ ti list --assigned alice@example.com
```

## Points

Set a numeric estimate:

```
$ ti points 5
$ ti points --ticket a3f 8
```

Clear it:

```
$ ti points --clear
```

## Milestone

Group tickets by release or sprint:

```
$ ti milestone v2.0
$ ti milestone --ticket a3f "sprint-14"
```

Clear it:

```
$ ti milestone --clear
```

## Spec

The spec field holds implementation specifications -- the "how", separate from the description's "what/why". Set it via `ti edit` or programmatically.

When a spec is set, `ti show` displays its first line:

```
$ ti show a3f
-----------------------------------------------------------
Title   : Refactor auth middleware
...
Spec    : Use RS256 tokens with 24h expiry, rotate via cron
...
```

## Custom metadata

For anything that doesn't fit the built-in fields, use `ti meta` to set arbitrary key-value pairs:

```
$ ti meta branch feature/auth-refactor
$ ti meta priority P0
$ ti meta --ticket a3f source "customer-report"
```

Read metadata from a file:

```
$ ti meta spec-doc --file design.md
```

Custom metadata shows up in `ti show` output and in JSON:

```
$ ti show a3f --filter .meta.branch
feature/auth-refactor
```

## All fields in JSON

The full ticket schema is at [ticgit.dev/schema/v1.json](https://ticgit.dev/schema/v1.json). A ticket object looks like:

```json
{
  "id": "a3f29c84-...",
  "title": "fix parser panic",
  "description": "Panics on empty input...",
  "spec": null,
  "status": "open",
  "state": "in-progress",
  "assigned": "alice@example.com",
  "points": 5,
  "milestone": "v2.0",
  "tags": ["bug", "urgent"],
  "meta": {
    "branch": "feature/fix-parser"
  },
  "comments": [...],
  "created_at": "2026-05-01T10:30:00Z",
  "created_by": "bob@example.com"
}
```
