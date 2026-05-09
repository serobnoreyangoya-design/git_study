# Views and Import

Commands: `ti save-view`, `ti views`, `ti import`

## Saved views

Views are named collections of tickets based on filter criteria. They let you save a query you run often.

### Create a view

```
$ ti save-view bugs --tag bug --status open
Saved view: bugs (3 tickets)

$ ti save-view my-work --assigned alice@example.com --status open
Saved view: my-work (7 tickets)

$ ti save-view blocked --state blocked
Saved view: blocked (2 tickets)
```

### List saved views

```
$ ti views
bugs
my-work
blocked
```

### Show a view's tickets

```
$ ti views bugs
a3f29c84-...
d91c3a12-...
7b2e4f9a-...
```

### Use a view in list

```
$ ti list --view bugs
  TicId  Date   Title                    Status State       Tags
----------------------------------------------------------------------
  a3f29c 2d     fix parser panic         open   in-progress bug
  d91c3a 5d     null pointer in export   open   new         bug
  7b2e4f 8d     crash on large input     open   assigned    bug
```

## Import from GitHub

Pull open issues from a GitHub repository into TicGit. Requires the [GitHub CLI](https://cli.github.com/) (`gh`).

```
$ ti import gh --repo owner/repo
Imported 12 ticket(s) from owner/repo
  a3f29c  fix parser panic on empty input
  d91c3a  null pointer in CSV export
  7b2e4f  crash on large input files
  ... and 9 more
```

What gets imported:
- Issue title becomes the ticket title
- Issue body, GitHub URL, author, and assignees go into the description
- Issue labels become tags (plus `github` and `github-issue`)
- First GitHub assignee becomes the ticket assignee
- Issue milestone becomes the ticket milestone

### Limit the import

```
$ ti import gh --repo owner/repo --limit 50
```

### Machine-readable output

```
$ ti import gh --repo owner/repo --json
```
