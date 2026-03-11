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

| Layer       | Technology                              |
| ----------- | --------------------------------------- |
| Runtime     | Bun                                     |
| Language    | TypeScript                              |
| CRDT        | Automerge (`@automerge/automerge`)      |
| CLI         | `commander` (or raw `process.argv`)     |
| Server      | `Bun.serve()` (native HTTP + WebSocket) |
| Frontend    | Vue 3 + Vite + Pinia                    |
| Linting     | oxlint                                  |
| Formatting  | oxfmt                                   |
| Testing     | Vitest                                  |

## Data Model

Every project is a single Automerge document containing all todos, members, and metadata.

```typescript
interface Project {
  _version: number          // Schema version for migrations
  id: string                // UUID
  prefix: string            // e.g. "ABC" — for issue IDs like ABC-1
  name: string
  description: string
  counter: number           // Auto-incrementing (Automerge Counter) for issue numbers
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
  .todo/                        # Created by `todo init`
    config.toml                 # Project config (prefix, name) — safe to commit
    data.automerge              # CRDT binary document — gitignored
  .gitignore                    # Should include .todo/data.automerge
  src/
  ...
```

- `.todo/config.toml` is small, human-readable, and can be committed so collaborators
  share the project prefix and name.
- `.todo/data.automerge` is a binary blob. It should be gitignored. Sync happens via
  the Automerge sync protocol, not git.
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
      sync.ts               # Automerge sync protocol helpers
      migrate.ts            # Schema migration logic
      export.ts             # Markdown export renderer

    cli/                    # CLI application
      index.ts              # Entry point, command router
      commands/
        init.ts             # `todo init` — create .todo/ in current repo
        add.ts              # `todo add "title"`
        list.ts             # `todo list`
        update.ts           # `todo update ABC-1 --status done`
        show.ts             # `todo show ABC-1`
        assign.ts           # `todo assign ABC-1 kaiwen`
        serve.ts            # `todo serve` — start server + web dashboard
        export.ts           # `todo export --format md`
      output.ts             # Terminal formatting (tables, colors)

    server/                 # Bun.serve() — sync hub + static file server
      index.ts              # Server entry point

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

```bash
# Project setup
todo init --name "My Project" --prefix ABC
  # Checks for .git/ (warns if missing)
  # Creates .todo/ directory with config.toml + data.automerge
  # Adds .todo/data.automerge to .gitignore

# Creating todos
todo add "Fix authentication bug" --priority high --tags auth,security
todo add "Write tests" --status todo --assignee "Kaiwen"

# Listing and filtering
todo list                              # All non-archived todos
todo list --status in_progress         # Filter by status
todo list --assignee kaiwen            # Filter by assignee
todo list --tag auth                   # Filter by tag
todo list --json                       # JSON output (for agents)

# Viewing details
todo show ABC-1                        # Full detail for a single todo

# Updating
todo update ABC-1 --status done
todo update ABC-1 --title "New title" --priority urgent
todo assign ABC-1 kaiwen              # Shorthand
todo archive ABC-1                    # Shorthand for --status archived

# Bulk operations (agent-friendly)
todo batch --file changes.json         # Apply multiple ops from JSON

# Web dashboard
todo serve                             # Start server on default port
todo serve --port 8080                 # Custom port

# Export
todo export --format md                # Print markdown to stdout
todo export --format json              # Print JSON to stdout
```

### Agent-Friendly Design

- All commands support `--json` flag for structured output
- Exit codes: 0 = success, 1 = error, 2 = not found
- No interactive prompts — all parameters are flags/arguments
- `todo batch` supports bulk operations from a JSON file (agents can write the file
  and apply many changes in one call)

## Sync Architecture

```
┌─────────────┐    Automerge Sync     ┌──────────────────┐    Automerge Sync     ┌─────────────┐
│   CLI        │ ◄─── WebSocket ────► │   Bun Server     │ ◄─── WebSocket ────► │  Vue Web     │
│  (Automerge) │     (binary msgs)    │  (Automerge hub) │     (binary msgs)    │  (Automerge) │
└─────────────┘                       └──────────────────┘                       └─────────────┘
      │                                       │
      │  (offline: direct                     │  Disk
      │   file read/write)                    │  .todo/data.automerge
      ▼                                       ▼
  .todo/data.automerge              .todo/data.automerge
  (same file)                       (same file)
```

### How Sync Works

The Automerge sync protocol is built into `@automerge/automerge`. It handles:
- Determining what each peer is missing
- Sending minimal deltas (not full documents)
- Merging concurrent changes conflict-free

**Components:**

1. **Server (`Bun.serve()`)** — The sync hub. Holds the Automerge document in memory,
   persists to `.todo/data.automerge`. Runs a WebSocket endpoint at `/sync`. Each
   connected client gets its own `Automerge.SyncState`.

2. **CLI** — Works in two modes:
   - **Offline (default):** Reads/writes `.todo/data.automerge` directly. No server needed.
   - **Connected:** If `todo serve` is running, the CLI can optionally connect via
     WebSocket to sync changes in real-time.

3. **Web dashboard** — Always connects to the server via WebSocket. Holds its own
   Automerge document in memory. Mutations are local-first (instant UI), then synced.

### Sync Protocol Flow

```
Client connects via WebSocket
  → Server creates SyncState for this client
  → Server calls Automerge.generateSyncMessage(doc, syncState)
  → Server sends sync message to client
  → Client calls Automerge.receiveSyncMessage(doc, syncState, msg)
  → Client generates response: Automerge.generateSyncMessage(doc, syncState)
  → ... messages go back and forth until both sides are in sync
  → When either side makes a local change, the cycle repeats
```

Messages are binary (`Uint8Array`). The protocol is efficient — after initial sync,
only deltas are exchanged.

### Offline-First Behavior

- The CLI always works without the server. It reads/writes the Automerge file directly.
- The web dashboard requires the server to be running.
- When the server starts, it loads the current `.todo/data.automerge`. If the CLI made
  changes while the server was off, the server picks them up on next load.
- If the CLI and server both modify the document simultaneously (e.g., CLI writes to
  disk while server has it in memory), the next sync will merge both sets of changes
  via Automerge — no data loss.

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

### Server Command

```bash
todo serve --port 3000
```

Starts the Bun server which:
1. Serves the built Vue app as static files
2. Runs the WebSocket sync endpoint at `/sync`
3. Opens the browser automatically (like `backlog browser`)

## Implementation Order

1. **lib/** — Schema types, Automerge operations, storage, project config
2. **cli/** — `init`, `add`, `list`, `update`, `show` commands
3. **server/** — `Bun.serve()` with WebSocket sync + static file serving
4. **web/** — Vue dashboard (kanban board, todo detail)
5. **Polish** — `--json` flag, `todo batch`, `todo export`, error handling
6. **Multi-user** — Deploy server, add auth, multiple peers sync
