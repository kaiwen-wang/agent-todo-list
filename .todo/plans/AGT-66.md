# AGT-66: decide whether server is necessary or we can just call cli each time

## Research

### Current Server Architecture
The server (`agt-server`) is ~978 lines of Rust across 3 files, built on Axum + Tokio. It:

- **Keeps the Automerge document in memory** (`Arc<Mutex<AutoCommit>>`) — loaded once at startup, mutated in-place on writes
- **Exposes 4 endpoints**: `GET /api/project` (full state), `POST /api/change` (all mutations via action dispatch), `GET /api/plan/{n}`, `WS /ws`
- **Watches `data.automerge` on disk** via `notify` crate — when the CLI writes to it externally, the server reloads and broadcasts a WebSocket refresh to all connected browsers
- **Serves the Vue SPA** as static files from `~/.local/share/agt/web`
- **Spawns background agents** for `researchPlan` — invokes `claude` CLI, parses streaming JSON output, and broadcasts `plan:start/progress/done/error` events over WebSocket

### Current CLI Architecture
Each CLI invocation is stateless: load doc from disk → mutate → save to disk + `git add`. The Automerge file is currently ~114KB. Load + deserialize is ~10-50ms per invocation.

### Frontend Data Flow
The Vue frontend fetches the entire project state on load and after each mutation. It uses optimistic updates for drag-and-drop (status changes, bulk updates) and suppresses the next WebSocket-triggered reload to avoid flicker. The WebSocket connection auto-reconnects every 2 seconds if dropped.

### What the Server Does Beyond Wrapping CLI
1. **In-memory document** — reads are sub-millisecond vs. 10-50ms disk load per CLI call
2. **WebSocket real-time sync** — multi-tab/multi-user awareness, file-watcher-driven refresh when CLI modifies data
3. **Background agent streaming** — plan research progress is streamed live to the browser
4. **Static file hosting** — serves the built Vue app
5. **Concurrent write safety** — Mutex-protected document access

## Approach Options

### Option 1: Keep the server (status quo)

**Pros:**
- Real-time sync between CLI and browser via file watcher + WebSocket
- Sub-millisecond reads from in-memory doc
- Background agent streaming works naturally
- Multi-tab sync is free
- Already built and working (~978 lines)

**Cons:**
- Extra process to run (`agt serve`)
- Extra crate to maintain (axum, tokio, notify, tower-http dependencies)
- Server and CLI have duplicated mutation logic (both can write the doc)

### Option 2: CLI-only, frontend shells out per action

Replace the server with a thin static file server (or `python -m http.server`). The frontend calls the CLI via a minimal HTTP-to-CLI bridge or uses a browser extension / local proxy.

**Pros:**
- Single source of truth for all mutations (CLI only)
- No server state to manage
- Simpler mental model

**Cons:**
- Loses WebSocket real-time sync entirely — no way to detect external changes
- Each action pays 10-50ms CLI startup + disk I/O overhead
- Background agent streaming would require polling or a separate mechanism
- Still need *something* to serve the frontend and translate HTTP → CLI exec
- That "something" is basically a server anyway, just dumber

### Option 3: Hybrid — thin CLI bridge server (no in-memory state)

Replace the current server with a stateless HTTP wrapper that calls CLI commands via `std::process::Command` for each request. Keep WebSocket + file watcher for sync.

**Pros:**
- All mutation logic lives in one place (CLI/lib)
- Server is a pure transport layer — easier to reason about
- Still supports real-time sync

**Cons:**
- ~40-80ms latency per request (process spawn + doc load + serialize)
- Drag-and-drop and rapid interactions feel sluggish
- More complex than current approach (process management, stdout parsing)
- Server still needs file watcher + WebSocket, so it's not actually simpler
- Loses ability to do atomic bulk operations efficiently

### Option 4: Server calls library functions directly (no CLI subprocess)

This is essentially what the current architecture already does — `agt-server` imports `agt-lib` and calls the same operations the CLI uses. The difference is acknowledging this is the right design and not changing it.

**Pros:**
- Already implemented
- Best performance (in-memory operations)
- CLI and server share the same `agt-lib` operations crate
- No process spawn overhead

**Cons:**
- Two "entry points" to mutations (CLI and server) — but both use `agt-lib`, so logic isn't actually duplicated

## Recommendation

**Keep the current architecture (Option 4 / status quo).** The server is necessary and well-designed.

The key insight is that the server doesn't duplicate the CLI — both are thin entry points that call into `agt-lib` for actual operations. The server adds three things that genuinely require a long-running process:

1. **In-memory state** for fast reads (the dashboard is read-heavy)
2. **WebSocket push** for real-time sync between CLI and browser
3. **Background agent streaming** for plan research

None of these can be achieved by shelling out to the CLI per request. A "CLI-only" approach would still need a process to serve the frontend, watch files, and manage WebSockets — which is just the current server with worse performance.

The server is only 978 lines and its complexity is justified. If anything, the area to improve is ensuring `agt-server` always delegates to `agt-lib` operations rather than implementing mutation logic inline in route handlers.

## Questions

- Is there a specific pain point motivating this question? (e.g., server maintenance burden, startup time, deployment complexity)
- Are there plans for a desktop app (Tauri) that would change the architecture? Currently there's no Tauri setup in the project.
- Would it be useful to verify that all server route handlers delegate to `agt-lib` rather than implementing logic inline? That would be a concrete improvement regardless of this decision.