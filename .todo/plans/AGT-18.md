# AGT-18: Archived is probably different from trash.

## Research

### Current State

The project has **no "trash" concept** — there are only two ways to remove a todo from active views:

1. **Archive** (`status: "archived"`) — reversible status change, item still exists in the CRDT document
2. **Delete** (`operations::delete_todo()`) — permanent removal from the CRDT, no recovery possible

### Terminology Confusion in the UI

The web frontend conflates "archive" and "trash":

- **`TodoDetailModal.vue:179`** — The archive handler displays `"Todo moved to trash"` but actually sets status to `"archived"`
- **`TodoDetailModal.vue:433`** — Uses a **trash icon** (`@vicons/tabler`) for the archive button
- **`BatchActionBar.vue:43-49`** — Bulk operations have both "archive" (status change) and "delete" (permanent removal) as separate actions, but the individual todo modal only has the archive button

### CLI Handling

The CLI has proper separation:
- `agt list` — hides archived/wont_do by default
- `agt list --archived` — shows **only** archived todos (`src/rust/crates/agt-cli/src/main.rs:79-81`)
- `agt list --all` — shows everything including archived
- `agt delete` — permanently removes from the CRDT (no recovery)

### Web Handling

- **BoardView** — Archived is shown as its own column alongside all other statuses (all 8 statuses are visible columns via `BOARD_STATUSES` in `types.ts:111-120`)
- **ListView** — Archived can be filtered via the status dropdown
- **Store** — `activeTodos` computed property filters out archived and wont_do (`stores/project.ts:59-61`)

### Default Query Behavior

In `src/rust/crates/agt-lib/src/queries.rs:194-198`, when no explicit status filter is provided, both `Archived` and `WontDo` are filtered out automatically.

### What the Task is Actually About

The task description says "Have a 'show archived' or show trash sitting around somewhere." The core issue is:

1. **The UI treats archive as trash** — wrong icon, wrong message, misleading UX
2. **There's no actual trash/soft-delete** — delete is permanent and irreversible
3. **Archived items are always visible on the board** — there's no toggle to hide/show them, they just sit in a column

## Approach Options

### Option 1: Fix terminology only (minimal)

Just fix the misleading UI text and icon — archive is archive, not trash.

- Change `"Todo moved to trash"` → `"Todo archived"` in `TodoDetailModal.vue:179`
- Replace trash icon with an archive icon
- **Pros:** Tiny change, removes confusion, accurate to actual behavior
- **Cons:** Doesn't address whether archived items should be hidden by default on the board, doesn't add a real trash/soft-delete concept

### Option 2: Add a real "Trash" status separate from "Archived"

Add a new `Trash` status to the schema. Archive = "done, keep for reference." Trash = "soft-deleted, recover within X days or purge."

- New `Status::Trash` variant in schema
- Schema migration (version bump)
- Delete button → moves to Trash instead of permanent delete
- New "Empty Trash" action for permanent deletion
- Trash column on board (or hidden section)
- **Pros:** Matches mental model users expect, prevents accidental permanent deletion, two distinct concepts
- **Cons:** Schema migration, more complexity, need to decide on auto-purge policy, adds another status to an already large enum

### Option 3: Hide archived column by default on the board + fix terminology

Keep the current model (archive only, delete is permanent) but:
- Fix the trash/archive terminology confusion
- Hide the "Archived" column on the board by default
- Add a toggle/button like "Show archived" to reveal the column
- Same for ListView — default filter excludes archived (matching CLI behavior)

- **Pros:** Consistent with CLI behavior (archived hidden by default), cleaner board view, no schema change
- **Cons:** Doesn't add soft-delete safety net for accidental deletes

### Option 4: Option 3 + make delete go through archive first

Combine Option 3 with changing the delete flow: instead of permanent deletion, `delete` sets status to `archived`. Add a separate "permanently delete" or "purge" action for archived items only.

- **Pros:** Safety net against accidental deletion, consistent "archive as soft-delete" model, no schema change needed
- **Cons:** Changes semantics of delete (could confuse agents that expect delete to be permanent), muddies the distinction between "I'm done with this" and "this was a mistake"

## Recommendation

**Option 3** — Fix the terminology and hide archived by default on the board.

This is the right scope because:
- The terminology bug is the most concrete problem (UI says "trash" but means "archive")
- Hiding archived by default on the board matches the CLI's behavior and keeps the board clean
- A toggle to "show archived" directly addresses the task description
- No schema migration needed
- The permanent delete behavior is fine for a developer/agent tool — users of this tool understand what delete means

Implementation would touch:
1. `TodoDetailModal.vue` — fix message text and icon
2. `types.ts` — remove `archived` from `BOARD_STATUSES`, or make it conditional
3. `BoardView.vue` — add a "Show archived" toggle
4. `ListView.vue` — default status filter to exclude archived (with option to include)

## Questions

- Do you actually want a separate "trash" concept (soft-delete before permanent delete), or is the real issue just that archived items clutter the board and the UI language is wrong?
- Should `wont_do` items also be hidden by default on the board (they're filtered out by default in CLI/queries too)?
- For the board toggle, preference on placement — a button in the board header, or a sidebar filter?

## Answers