# Agent Integration

TicGit is designed to work with AI coding agents. Every command that reads or writes data supports `--json` and `--markdown` output.

## For agents: getting started

Run `ti help --agent` for a Markdown guide written for agents, covering the full command set with structured output examples.

## JSON workflow

Agents should use `--json` for all reads and writes:

```
$ ti list --json
[
  {
    "id": "3d3a5d72-109d-47fa-a91d-0c7ff5d00b4a",
    "title": "Add spec field to tickets",
    "status": "open",
    "state": "new",
    "tags": ["feature"],
    ...
  }
]

$ ti show 3d3 --json
{
  "id": "3d3a5d72-...",
  "title": "Add spec field to tickets",
  "description": null,
  "status": "open",
  "state": "new",
  "comments": [
    {
      "author": "schacon@gmail.com",
      "at": "2026-05-09T10:46:38.649Z",
      "body": "Add a top-level 'spec' field..."
    }
  ],
  ...
}
```

## Markdown workflow

`--markdown` output includes next-step command suggestions, useful for agents that need guidance on what to do next:

```
$ ti show 3d3 --markdown
```

## Extract specific fields

Use `--filter` with jq-like paths:

```
$ ti show 3d3 --filter .title
Add spec field to tickets

$ ti show 3d3 --filter .comments[0].body
Add a top-level 'spec' field...

$ ti show 3d3 --filter .tags
["feature"]
```

## Create and update tickets

All write commands return the updated ticket as JSON when you pass `--json`:

```
$ ti new --title "fix the bug" --tags bug --json
{"id":"a3f29c84-...","title":"fix the bug","status":"open",...}

$ ti comment --ticket a3f "root cause identified" --json
{"id":"a3f29c84-...","comments":[...],...}

$ ti state --ticket a3f in-progress --json
{"id":"a3f29c84-...","state":"in-progress",...}

$ ti tag --ticket a3f urgent --json
{"id":"a3f29c84-...","tags":["bug","urgent"],...}

$ ti close a3f --json
{"id":"a3f29c84-...","status":"closed","state":"resolved",...}
```

## ID-only creation

When an agent just needs the ticket ID:

```
$ ti new --title "automated finding" --id-only
a3f29c84-d6ec-3da1-a180-0a33fb090d59
```

## Piping and scripting

```
# Get all open bug titles
$ ti list --tag bug --json | jq -r '.[].title'

# Count tickets by state
$ ti list --all --json | jq 'group_by(.state) | map({state: .[0].state, count: length})'

# Bulk close tickets matching a pattern
$ ti list --json | jq -r '.[] | select(.title | test("cleanup")) | .id' | \
    xargs -I{} ti close {}
```

## Schema

The full JSON schema is published at [ticgit.dev/schema/v1.json](https://ticgit.dev/schema/v1.json). All JSON output conforms to this schema.
