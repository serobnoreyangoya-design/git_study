# Sync and Setup

Commands: `ti sync`, `ti init`, `ti setup`, `ti update`

## How TicGit stores data

TicGit stores tickets as [git-meta](https://github.com/anthropics/git-meta) metadata, which lives in a separate Git ref (`refs/meta/main`). This means:

- Tickets don't clutter your working tree or commit history
- They sync with `git push`/`git pull` on the meta ref
- Multiple people can work on tickets offline and merge later
- The full ticket history is in the repo, not on a server

## ti init

Initialize TicGit on a repository for the first time:

```
$ cd my-project
$ ti init
Initialised ticgit metadata (schema v1).
```

This is idempotent -- running it again is safe.

## ti setup

If the repo has a `.git-meta` file (a one-line file with the remote URL), `ti setup` configures the git-meta remote:

```
$ cat .git-meta
git@github.com:owner/repo.git

$ ti setup
Configured git-meta remote from .git-meta: git@github.com:owner/repo.git
```

This also happens automatically the first time you run any `ti` command in a repo with a `.git-meta` file.

## ti sync

Sync pulls remote ticket changes, then pushes your local changes:

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

When there's nothing new:

```
$ ti sync
Remote: origin
Ref: refs/meta/main
URL: git@github.com:owner/repo.git
Web URL: https://github.com/owner/repo
Pull: no new tickets.
Push: 9 ticket(s) synced.
Done.
```

### Specify a remote

```
$ ti sync --remote upstream
```

By default, `ti sync` uses the first remote with `remote.<name>.meta = true` in your git config, falling back to `origin`.

## ti update

Update the `ti` binary to the latest GitHub release:

```
$ ti update
```

Check for updates without installing:

```
$ ti update --check
```

## Team workflow

A typical team setup:

1. One person initializes TicGit and adds the `.git-meta` file:

```
$ ti init
$ echo "git@github.com:team/project.git" > .git-meta
$ git add .git-meta && git commit -m "add git-meta config"
$ git push
$ ti sync
```

2. Everyone else clones and starts using tickets:

```
$ git clone git@github.com:team/project.git
$ cd project
$ ti list          # auto-setup kicks in, pulls tickets
```

3. Sync regularly:

```
$ ti sync          # before and after working on tickets
```

Since tickets are just Git data, conflicts are handled by git-meta's merge strategy. In practice, conflicts are rare because tickets use structured fields rather than free-form text files.
