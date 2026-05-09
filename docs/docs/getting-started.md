# Getting Started

TicGit is a distributed issue tracker that stores tickets directly in your Git repository as metadata. No server, no database, no separate account -- just Git.

## Install

```
curl -fsSL https://ticgit.dev/install | sh
```

Or with Cargo:

```
cargo install ticgit
```

## Setup

### New project

Initialize TicGit in any Git repository:

```
$ cd my-project
$ ti init
Initialised ticgit metadata (schema v1).
```

That's it. Start creating tickets.

### Cloned project

If someone has already set up TicGit on a repo and added a `.git-meta` file, just run:

```
$ ti setup
Configured git-meta remote from .git-meta: git@github.com:owner/repo.git
```

Or if the repo already has a `.git-meta` file, `ti` will auto-configure on first use.

### Sync

Pull and push ticket data with `ti sync`:

```
$ ti sync
Remote: origin
Ref: refs/meta/main
URL: git@github.com:owner/repo.git
Web URL: https://github.com/owner/repo
Pull: 3 new ticket(s):
  a07da7  resolved tickets remain checked out after close
  9b8039  Implement `ti context <id>` for agent workflows
  5d607a  Link tickets to branches, commits, and PRs
Push: 23 ticket(s) synced.
Done.
```

## Quick workflow

Create a ticket, work on it, close it:

```
$ ti new --title "fix the parser" --tags bug
Created: a3f29c

$ ti list
  TicId  Date   Title              Status State    Assgn  Tags
------------------------------------------------------------------------
  a3f29c 0d     fix the parser     open   new             bug

$ ti checkout a3f
Checked out: a3f29c  fix the parser

$ ti comment "root cause is the empty-input path"

$ ti state in-progress

$ ti comment "fixed, added test coverage"

$ ti close
Closed: a3f29c  fix the parser

$ ti sync
```

## Ticket identity

Every ticket gets a UUID. You can refer to a ticket by any unique prefix of that UUID:

```
$ ti show a3f
$ ti show a3f29c
$ ti show a3f29c84-d6ec-3da1-a180-0a33fb090d59
```

All three refer to the same ticket. TicGit will tell you if a prefix is ambiguous.

## Checked-out ticket

Many commands default to the "currently checked-out" ticket, so you don't have to keep passing `--ticket`:

```
$ ti checkout a3f
$ ti comment "this applies to a3f"
$ ti tag bug
$ ti assign alice@example.com
$ ti state in-progress
```

Clear it with `ti checkout --clear`.

## Output formats

Every read command supports `--json` and `--markdown`:

```
$ ti list --json
$ ti show a3f --json
$ ti show a3f --markdown
```

JSON output goes to stdout with no ANSI escapes, so it's safe to pipe:

```
$ ti list --json | jq '.[].title'
"fix the parser"
"update docs"
```

You can also extract a single field:

```
$ ti show a3f --filter .title
fix the parser

$ ti show a3f --filter .comments[0].body
root cause is the empty-input path
```
