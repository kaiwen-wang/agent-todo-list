# agent-todo-list Architecture

A local-first, CRDT-backed todo/project management tool for developers and AI agents.
CLI for agents and power users. Web dashboard for visual management. Automerge for
conflict-free sync.

## Design Principles

1. **Local-first** — Works offline. Data lives in the repo. No cloud dependency.
2. **Agent-native** — CLI is the primary interface. Agents call it directly via bash.
3. **CRDT-backed** — Automerge handles merge conflicts. Multi-user sync is a transport
   problem, not a data problem.
4. **Per-repo** — One project per git repo. The `.todo/` directory makes a repo "tracked."
5. **Simple until proven otherwise** — No frameworks, ORMs, or abstractions until needed.

## Tech Stack

| Layer       | Technology                                        |
| ----------- | ------------------------------------------------- |
| Runtime     | Bun                                               |
| Language    | TypeScript                                        |
| CRDT        | Automerge (`@automerge/automerge`)                |
| CLI         | `commander` (or raw `process.argv`)               |
| Server      | `Bun.serve()` (native HTTP + WebSocket)           |
| Frontend    | Vue 3 + Vite + Pinia                              |
| Linting     | oxlint                                            |
| Formatting  | oxfmt                                             |
| Testing     | `bun test` (lib/cli) + Vitest (web)               |

### Why two test runners?

Vue Single File Components (`.vue` files) combine template, script, and style in one
file. To test them, the runner must compile `.vue` into JavaScript. Vitest shares Vite's
plugin pipeline (`@vitejs/plugin-vue`), so SFCs work automatically. `bun test` doesn't
understand `.vue` files without a custom preload plugin.

For `lib/` and `cli/` (pure TypeScript, no `.vue` files), `bun test` is simpler, faster,
and has zero config.

## Data Model

Every project is a single Automerge document containing all todos, members, and metadata.

```typescript
import { Counter } from '@automerge/automerge'

interface Project {
  _version: number          // Schema version for migrations
  id: string                // UUID
  prefix: string            // e.g. "ABC" — for issue IDs like ABC-1
  name: string
  description: string
  counter: Counter          // CRDT-safe auto-incrementing counter for issue numbers
  createdAt: string         // ISO 8601

  members: Member[]
  todos: Todo[]
}

interface Todo {
  id: string                // UUID
  number: number            // Sequential — displayed as PREFIX-N (e.g. ABC-1)
  title: string
  description: string       // Markdown body
  status: Status
  priority: Priority
  assignee: string | null   // Member ID
  tags: string[]
  createdAt: string         // ISO 8601
  updatedAt: string
  createdBy: string         // Member ID
}

type Status = 'backlog' | 'todo' | 'in_progress' | 'done' | 'archived'
type Priority = 'low' | 'medium' | 'high' | 'urgent'

interface Member {
  id: string
  name: string
  email: string | null
  role: 'owner' | 'member' | 'agent'
}
```

### Automerge Counter

The `counter` field uses `Automerge.Counter` instead of a plain `number`. This is a
CRDT-native type — concurrent increments from different peers merge correctly via
commutative addition. With a plain number, two agents adding todos simultaneously would
both read the same value and produce duplicate todo numbers.

```typescript
// Inside an Automerge.change() callback:
d.counter.increment(1)        // CRDT-safe increment
const num = d.counter.value   // read the current value
```

### Schema Migrations

Automerge is schema-less. Migrations are handled at read-time:

- A `_version` field tracks the schema version.
- On load, the document is checked against the current version.
- Migration functions transform old shapes to new ones inside an `Automerge.change()` call.
- Adding fields is free (read with defaults). Renaming fields requires explicit migration.

## On-Disk Layout

### Per-Repo Data

```
my-project/                     # Any git repository
  .git/
  .gitattributes                # Merge driver for .automerge files
  .todo/                        # Created by `agt init`
    config.toml                 # Project config (prefix, name)
    data.automerge              # CRDT binary document — committed to git
  src/
  ...
```

- One project per git repo. `.todo/` is created at the git root (same level as `.git/`).
- `agt init` warns if no `.git/` is found but still allows creation (for quick experiments).
- **Both files are committed to git.** `config.toml` is human-readable. `data.automerge`
  is a binary blob, but Automerge's merge semantics make it safe to commit — a custom
  git merge driver handles binary conflicts using `Automerge.merge()`.
- Sync between collaborators happens via `git push`/`git pull`. No server needed.
- The CLI finds the project by walking up directories looking for `.todo/` (like git
  finds `.git/`).

### Config File

```toml
# .todo/config.toml
id = "550e8400-e29b-41d4-a716-446655440000"
prefix = "ABC"
name = "My Project"
```

## Project Structure

```
agent-todo-list/
  package.json              # Bun workspace root
  bunfig.toml
  tsconfig.json             # Base TypeScript config
  oxlint.config.json
  ARCHITECTURE.md           # This file

  src/
    lib/                    # Shared library (not a separate package)
      schema.ts             # TypeScript types (Project, Todo, Member)
      operations.ts         # Automerge mutation functions
      queries.ts            # Read/filter functions
      storage.ts            # Load/save .automerge files
      project.ts            # .todo/ directory management, config I/O
      merge-driver.ts       # Git merge driver for .automerge files
      migrate.ts            # Schema migration logic
      export.ts             # Markdown export renderer

    cli/                    # CLI application
      index.ts              # Entry point, command router
      commands/
        init.ts             # `agt init` — create .todo/ in current repo
        add.ts              # `agt add "title"`
        list.ts             # `agt list`
        update.ts           # `agt update ABC-1 --status done`
        show.ts             # `agt show ABC-1`
        assign.ts           # `agt assign ABC-1 kaiwen`
        browser.ts          # `agt browser` — open web dashboard in browser
        export.ts           # `agt export --format md`
      output.ts             # Terminal formatting (tables, colors)

    server/                 # Bun.serve() — local web dashboard server
      index.ts              # Server entry point (serves Vue app, reads doc from disk)

    web/                    # Vue 3 + Vite frontend (scaffolded by create-vue)
      src/
        App.vue
        main.ts
        router/index.ts
        stores/
          project.ts        # Pinia store wrapping Automerge doc
        views/
          Board.vue         # Kanban board (default view)
          ListView.vue      # Table/list view
          TodoDetail.vue    # Single todo detail
        components/
          TodoCard.vue
          StatusColumn.vue
          CreateTodoModal.vue
        composables/
          useAutomerge.ts   # Vue composable for Automerge doc reactivity
      vite.config.ts
      package.json
```

All source code lives under `src/`. `lib/` is NOT a workspace package — it's a shared
directory imported via TypeScript path aliases. This avoids a third `package.json` and
build step.

Bun workspaces are used for `src/server` and `src/web` (they have different dependency
profiles — web needs Vue/Vite, server doesn't).

## CLI Interface

The CLI command is **`agt`** (short for agent-todo). Run via `bun run agt` during
development, or install globally via `bun link`.

```bash
# Project setup
agt init --name "My Project" --prefix ABC
  # Warns if no .git/ found (but still allows creation)
  # Creates .todo/ directory with config.toml + data.automerge
  # Sets up .gitattributes with Automerge merge driver
  # Configures git merge driver in local repo config

# Creating todos
agt add "Fix authentication bug" --priority high --tags auth,security
agt add "Write tests" --status todo --assignee "Kaiwen"

# Listing and filtering
agt list                              # All non-archived todos
agt list --status in_progress         # Filter by status
agt list --assignee kaiwen            # Filter by assignee
agt list --tag auth                   # Filter by tag
agt list --json                       # JSON output (for agents)

# Viewing details
agt show ABC-1                        # Full detail for a single todo

# Updating
agt update ABC-1 --status done
agt update ABC-1 --title "New title" --priority urgent
agt assign ABC-1 kaiwen              # Shorthand
agt archive ABC-1                    # Shorthand for --status archived

# Bulk operations (agent-friendly)
agt batch --file changes.json         # Apply multiple ops from JSON

# Web dashboard
agt browser                            # Open web dashboard in browser
agt browser --port 8080                # Custom port

# Export
agt export --format md                # Print markdown to stdout
agt export --format json              # Print JSON to stdout
```

### Agent-Friendly Design

- All commands support `--json` flag for structured output
- Exit codes: 0 = success, 1 = error, 2 = not found
- No interactive prompts — all parameters are flags/arguments
- `agt batch` supports bulk operations from a JSON file (agents can write the file
  and apply many changes in one call)

## Sync Architecture

Sync happens through **git**, not a custom server. The `.todo/data.automerge` file is
committed to the repository. Automerge's CRDT merge semantics guarantee conflict-free
resolution when two collaborators edit independently.

```
  Dev A                          Git Remote                         Dev B
┌──────────┐                   ┌──────────┐                    ┌──────────┐
│ agt add   │  git push/pull   │          │   git push/pull   │ agt add   │
│ agt update│ ◄──────────────► │  origin  │ ◄───────────────► │ agt update│
│           │                  │          │                   │           │
└──────────┘                   └──────────┘                    └──────────┘
      │                                                              │
  .todo/data.automerge                                    .todo/data.automerge
  (committed to git)                                      (committed to git)
```

### Git Merge Driver

When `git pull` encounters concurrent edits to `data.automerge`, git would normally
report a binary conflict. We avoid this with a custom merge driver:

```gitattributes
# .gitattributes (created by `agt init`)
.todo/data.automerge merge=automerge-crdt
```

The merge driver is a script that calls `Automerge.merge()`:

```
git config merge.automerge-crdt.driver "bun <path>/merge-driver.ts %O %A %B"
```

The script loads both sides (`%A` = ours, `%B` = theirs), merges them, and writes the
result back. Automerge merge is always conflict-free — concurrent changes combine
cleanly. `git merge` just works.

### Why Not WebSocket Sync?

The original architecture planned a WebSocket sync server. Git sync is better because:
- **Already there** — every collaborator already has git. No extra infrastructure.
- **Truly local-first** — works offline, no server dependency.
- **Simpler** — no sync protocol code, no server process, no connection management.
- **Auditable** — sync history is git history.

The `agt browser` command starts a local-only server for the web dashboard UI. It reads
from disk — no WebSocket sync needed.

## Web Dashboard

### Pages

- **`/`** — Kanban board (default). Columns: Backlog | Todo | In Progress | Done.
- **`/list`** — Table/list view with sorting and filtering.
- **`/todo/:number`** — Todo detail/edit view.

### Vue + Automerge Integration

The `useAutomerge` composable wraps an Automerge document in Vue reactivity:

```typescript
// composables/useAutomerge.ts
import { shallowRef } from 'vue'
import * as Automerge from '@automerge/automerge'

export function useAutomerge<T>(initialDoc: Automerge.Doc<T>) {
  const doc = shallowRef(initialDoc)

  function change(fn: Automerge.ChangeFn<T>) {
    doc.value = Automerge.change(doc.value, fn)
    // trigger sync...
  }

  return { doc, change }
}
```

`shallowRef` is used because Automerge documents are immutable — every `change()`
returns a new document object. Vue detects the reference change and re-renders.

### Browser Command

```bash
agt browser --port 3000
```

Starts a local Bun server which:
1. Loads `.todo/data.automerge` from disk
2. Serves the built Vue app as static files
3. Provides a REST API for the frontend to read/write the Automerge doc
4. Opens the browser automatically

## Implementation Order

1. **lib/** — Schema types, Automerge operations, storage, project config, merge driver
2. **cli/** — `init`, `add`, `list`, `update`, `show`, `browser` commands
3. **server/** — `Bun.serve()` local web dashboard server
4. **web/** — Vue dashboard (kanban board, todo detail)
5. **Polish** — `--json` flag, `agt batch`, `agt export`, error handling
