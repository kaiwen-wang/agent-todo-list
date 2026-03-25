
# agent-todo-list

A local-first, CRDT-backed todo/project management tool for developers and AI coding agents.
CLI for agents and power users. Web dashboard for visual management. Automerge for
conflict-free sync via git.

## Coding Guidelines

Default to using Bun instead of Node.js.

- Use `bun <file>` instead of `node <file>` or `ts-node <file>`
- Use `bun test` instead of `jest` or `vitest`
- Use `bun build <file.html|file.ts|file.css>` instead of `webpack` or `esbuild`
- Use `bun install` instead of `npm install` or `yarn install` or `pnpm install`
- Use `bun run <script>` instead of `npm run <script>` or `yarn run <script>` or `pnpm run <script>`
- Use `bunx <package> <command>` instead of `npx <package> <command>`
- Bun automatically loads .env, so don't use dotenv.

### APIs

- `Bun.serve()` supports WebSockets, HTTPS, and routes. Don't use `express`.
- `bun:sqlite` for SQLite. Don't use `better-sqlite3`.
- `Bun.redis` for Redis. Don't use `ioredis`.
- `Bun.sql` for Postgres. Don't use `pg` or `postgres.js`.
- `WebSocket` is built-in. Don't use `ws`.
- Prefer `Bun.file` over `node:fs`'s readFile/writeFile
- Bun.$`ls` instead of execa.

### Testing

Use `bun test` for lib/ tests:

```ts
import { test, expect } from "bun:test";

test("hello world", () => {
  expect(1).toBe(1);
});
```

Use Vitest for `src/web/` tests only. Vue Single File Components (`.vue` files) require
Vite's plugin pipeline to compile, so Vitest is needed there. Do not use Vitest for
lib/ code. Use `cargo test` for Rust CLI tests.

### CLI

The CLI is a Rust binary (`src/cli/`). Run via `cargo run --` during development, or
`agt` after `make deploy`.

### Frontend

The web dashboard uses Vue 3 + Vite. Vue SFCs require Vite's compiler toolchain.
Bun.serve() serves the built Vue app as static files in production, but Vite handles
the dev/build pipeline.

### Linting & Formatting

- oxlint for linting (`bunx oxlint`)
- oxfmt for formatting (`bunx oxfmt`)
- lefthook for pre-commit hooks (see `lefthook.toml`)

## Architecture

### Design Principles

1. **Local-first** -- Works offline. Data lives in the repo. No cloud dependency.
2. **Agent-native** -- CLI is the primary interface. Agents call it directly via bash.
3. **CRDT-backed** -- Automerge handles merge conflicts. Multi-user sync is a transport
   problem, not a data problem.
4. **Per-repo** -- One project per git repo. The `.todo/` directory makes a repo "tracked."
5. **Simple until proven otherwise** -- No frameworks, ORMs, or abstractions until needed.

### Tech Stack

| Layer       | Technology                              |
| ----------- | --------------------------------------- |
| Runtime     | Bun                                     |
| Language    | TypeScript                              |
| CRDT        | Automerge (`@automerge/automerge`)      |
| CLI         | Rust + `clap` (`src/cli/`)               |
| Server      | `Bun.serve()` (native HTTP)             |
| Frontend    | Vue 3 + Vite + Pinia                    |
| Linting     | oxlint                                  |
| Formatting  | oxfmt                                   |
| Testing     | `bun test` (lib) + `cargo test` (cli) + Vitest (web) |

### Data Model

Every project is a single Automerge document containing all todos, members, audit log,
and metadata. See `src/lib/schema.ts` for the full type definitions.

Key types:

- **Status**: `none`, `todo`, `in_progress`, `completed`, `archived`, `wont_do`, `needs_elaboration`
- **Priority**: `none`, `low`, `medium`, `high`, `urgent`
- **Difficulty**: `none`, `easy`, `medium`, `hard`
- **Labels**: `new_feature`, `bug`, `feature_plus`
- **MemberRole**: `owner`, `member`, `agent`
- **AgentProvider**: `claude-code`, `opencode`, `custom`

The `Project.counter` field uses `Automerge.Counter` -- a CRDT-native type where
concurrent increments from different peers merge correctly. This prevents duplicate
todo numbers when multiple agents add todos simultaneously.

Timestamps are Unix milliseconds (`Date.now()`), not ISO strings.

Schema version is tracked via `_version` (currently 4). Migrations run at load time
in `src/lib/migrate.ts`.

### On-Disk Layout

```
my-project/
  .git/
  .gitattributes          # Merge driver for .automerge files
  .todo/                  # Created by `agt init`
    config.toml           # Project config (id, prefix, name)
    data.automerge        # CRDT binary document -- committed to git
    TODO.md               # Freeform inbox (plain markdown, not in CRDT)
    TODO-PROCESSED.md     # Archive of processed inbox items
```

Both `.todo/config.toml` and `.todo/data.automerge` are committed to git. A custom git
merge driver handles binary conflicts using `Automerge.merge()` (configured in
`.gitattributes`). Sync between collaborators happens via `git push`/`git pull`.

### Project Structure

```
agent-todo-list/
  package.json
  tsconfig.json
  .oxlintrc.json
  lefthook.toml
  Makefile                # `make deploy` builds standalone binary

  src/
    lib/                  # Shared library (not a separate package)
      schema.ts           # TypeScript types (Project, Todo, Member, etc.)
      operations.ts       # Automerge mutation functions
      queries.ts          # Read/filter functions
      storage.ts          # Load/save .automerge files
      project.ts          # .todo/ directory management, config I/O
      merge-driver.ts     # Git merge driver for .automerge files
      migrate.ts          # Schema migration logic
      export.ts           # JSON serialization helpers
      brain.ts            # AI agent: processes inbox notes into structured tasks
      inbox.ts            # Manages .todo/TODO.md and TODO-PROCESSED.md
      git-identity.ts     # Reads git config user.name/email for member resolution
      __tests__/          # bun test

    server/               # Bun.serve() -- local web dashboard server
      index.ts            # Serves Vue app + REST API
      __tests__/

    web/                  # Vue 3 + Vite frontend
      src/
        App.vue
        main.ts
        api.ts            # REST API client
        types.ts          # Frontend type definitions
        router/index.ts
        stores/
          project.ts      # Pinia store wrapping project data
        views/
          BoardView.vue   # Kanban board (default at /board)
          ListView.vue    # Table/list view (/list)
          TodoDetailView.vue  # Single todo detail (/todo/:number)
          InboxView.vue   # Freeform inbox (/inbox)
          MembersView.vue # Team members management (/members)
        components/
          TodoCard.vue
          StatusColumn.vue
          CreateTodoModal.vue
          TodoDetailModal.vue
          BatchActionBar.vue
          SettingsModal.vue
      vite.config.ts
      package.json
```

The CLI is implemented in Rust under `src/cli/`. TypeScript `src/lib/` is shared by the
server and used at build time. `src/web/` is the Vue frontend. A previous TypeScript CLI
implementation is archived in `archive/cli-ts/` for reference but is not used.

`lib/` is NOT a workspace package -- it's a shared directory imported via TypeScript path
aliases. Bun workspaces are used for `src/web` (it needs Vue/Vite dependencies).

### CLI Commands

All commands support `--json` for structured output. No interactive prompts -- all
parameters are flags/arguments. Exit codes: 0 = success, 1 = error.

```bash
agt init --name "My Project" --prefix ABC
agt add "Fix auth bug" --priority high --status todo
agt list [--status in_progress] [--assignee name] [--json]
agt show PREFIX-N
agt update PREFIX-N --status done --priority urgent --title "New title"
agt delete PREFIX-N
agt assign PREFIX-N member-name
agt unassign PREFIX-N
agt comment PREFIX-N "This is a comment"
agt branch PREFIX-N
agt member add "Name" --role member
agt member list
agt member remove "Name"
agt config
agt serve [--port 3000]
agt inbox [show|append|clear]
agt brain [--auto]
agt log
agt commit [--push] [-m "message"]
```

### Brain / Inbox System

The inbox (`agt inbox`) lets users dump freeform notes into `.todo/TODO.md`. The brain
(`agt brain`) spawns the Claude CLI to process those notes into structured tasks -- it
reads the inbox, the full CLI reference, and project context, then creates/updates todos
automatically. Processed items move to `.todo/TODO-PROCESSED.md`.

### Sync via Git

Sync happens through git, not a custom server. The `.todo/data.automerge` file is
committed to the repository. Automerge's CRDT merge semantics guarantee conflict-free
resolution when two collaborators edit independently. A custom git merge driver
(configured by `agt init`) handles binary merges using `Automerge.merge()`.

The `agt serve` command starts a local-only HTTP server for the web dashboard. It reads
from disk -- no sync protocol needed.
