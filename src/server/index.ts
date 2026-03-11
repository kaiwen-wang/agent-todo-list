/**
 * agt browser server — minimal local dashboard.
 *
 * API endpoints:
 *   GET  /api/project — returns everything as JSON
 *   POST /api/change  — applies a single mutation
 *
 * Serves the Vue dist/ as static files.
 */

import * as Automerge from "@automerge/automerge";
import { join } from "node:path";
import { existsSync } from "node:fs";
import { watch, type FSWatcher } from "node:fs";
import type { ServerWebSocket } from "bun";
import type { Project } from "../lib/schema.js";
import { loadDoc, saveDoc } from "../lib/storage.js";
import {
  addTodo,
  updateTodo,
  deleteTodo,
  updateProject,
  addMember,
  removeMember,
  updateMember,
  addComment,
  setBranch,
  clearBranch,
} from "../lib/operations.js";
import { findMember } from "../lib/queries.js";
import { toJSON } from "../lib/export.js";
import { syncConfig } from "../lib/project.js";
import { migrateDoc, needsMigration } from "../lib/migrate.js";
import { readInbox, writeInbox, readProcessed, ensureInboxFiles } from "../lib/inbox.js";
import { processInbox } from "../lib/brain.js";

type Doc = Automerge.Doc<Project>;

const CORS_HEADERS = {
  "Access-Control-Allow-Origin": "*",
  "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
  "Access-Control-Allow-Headers": "Content-Type",
};

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { "Content-Type": "application/json", ...CORS_HEADERS },
  });
}

export interface ServerOptions {
  dev?: boolean;
  vitePort?: number;
}

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 60);
}

/** Load and auto-migrate the document from disk. */
async function loadAndMigrate(dataPath: string): Promise<Doc | null> {
  let doc = await loadDoc(dataPath);
  if (!doc) return null;
  if (needsMigration(doc)) {
    doc = migrateDoc(doc);
    await saveDoc(dataPath, doc);
  }
  return doc;
}

export async function startServer(projectPath: string, port = 3000, opts: ServerOptions = {}) {
  const todoDir = join(projectPath, ".todo");
  const dataPath = join(todoDir, "data.automerge");
  const configPath = join(todoDir, "config.toml");
  const distDir = join(import.meta.dir, "..", "web", "dist");
  let doc: Doc | null = await loadAndMigrate(dataPath);

  if (!doc) throw new Error(`Cannot load project data from ${dataPath}`);

  // ── WebSocket client tracking ──
  const wsClients = new Set<ServerWebSocket<unknown>>();

  function broadcast(message: string) {
    for (const ws of wsClients) {
      try {
        ws.send(message);
      } catch {
        wsClients.delete(ws);
      }
    }
  }

  // Track whether we're currently saving (to ignore our own file changes)
  let saving = false;

  async function reload() {
    doc = await loadAndMigrate(dataPath);
  }

  async function save() {
    if (doc) {
      saving = true;
      await saveDoc(dataPath, doc);
      // Small delay to let fs.watch settle before re-enabling external detection
      setTimeout(() => {
        saving = false;
      }, 100);
      broadcast(JSON.stringify({ type: "refresh" }));
    }
  }

  // ── File watcher — detect CLI writes to the automerge file ──
  let fileWatcher: FSWatcher | undefined;
  try {
    fileWatcher = watch(dataPath, { persistent: false }, async (eventType) => {
      if (saving) return; // ignore our own writes
      if (eventType === "change") {
        await reload();
        broadcast(JSON.stringify({ type: "refresh" }));
      }
    });
  } catch {
    // File watching not available — graceful degradation
  }

  const server = Bun.serve({
    port,

    routes: {
      "/api/project": {
        async GET() {
          await reload();
          if (!doc) return jsonResponse({ error: "No data" }, 500);
          await ensureInboxFiles(todoDir);
          const inboxText = await readInbox(todoDir);
          const inboxProcessed = await readProcessed(todoDir);
          return jsonResponse({ ...toJSON(doc), inboxText, inboxProcessed });
        },
      },

      "/api/change": {
        async POST(req) {
          await reload();
          if (!doc) return jsonResponse({ error: "No data" }, 500);

          const body = await req.json();
          try {
            switch (body.action) {
              case "add": {
                const result = addTodo(doc, {
                  title: body.title ?? "Untitled",
                  description: body.description,
                  status: body.status,
                  priority: body.priority,
                  labels: body.labels,
                  assignee: body.assignee,
                  platform: "web",
                });
                doc = result.doc;
                await save();
                return jsonResponse({ ok: true, number: result.number });
              }

              case "update": {
                doc = updateTodo(doc, body.number, body.updates ?? {});
                await save();
                return jsonResponse({ ok: true });
              }

              case "delete": {
                doc = deleteTodo(doc, body.number);
                await save();
                return jsonResponse({ ok: true });
              }

              case "addComment": {
                doc = addComment(doc, body.number, body.text ?? "");
                await save();
                return jsonResponse({ ok: true });
              }

              case "createBranch": {
                const todoNum = body.number;
                const todo = doc.todos.find((t) => t.number === todoNum);
                if (!todo) return jsonResponse({ error: `Todo #${todoNum} not found` }, 400);

                if (todo.branch) {
                  return jsonResponse({ ok: true, branch: todo.branch, alreadyExists: true });
                }

                const slug = slugify(todo.title);
                const branchName = `${doc.prefix.toLowerCase()}-${todoNum}-${slug}`;

                // Ensure .worktrees/ is gitignored
                const gitignorePath = join(projectPath, ".gitignore");
                const gitignoreFile = Bun.file(gitignorePath);
                if (await gitignoreFile.exists()) {
                  const content = await gitignoreFile.text();
                  if (!content.includes(".worktrees")) {
                    await Bun.write(
                      gitignorePath,
                      content.trimEnd() + "\n\n# Git worktrees for todo branches\n.worktrees/\n",
                    );
                  }
                } else {
                  await Bun.write(
                    gitignorePath,
                    "# Git worktrees for todo branches\n.worktrees/\n",
                  );
                }

                const worktreePath = join(projectPath, ".worktrees", branchName);

                if (existsSync(worktreePath)) {
                  return jsonResponse({ error: `Worktree path already exists` }, 400);
                }

                const result = Bun.spawnSync(
                  ["git", "worktree", "add", "-b", branchName, worktreePath],
                  { cwd: projectPath, stderr: "pipe", stdout: "pipe" },
                );

                if (result.exitCode !== 0) {
                  const stderr = result.stderr.toString().trim();
                  return jsonResponse({ error: `Failed to create worktree: ${stderr}` }, 500);
                }

                doc = setBranch(doc, todoNum, branchName);
                if (todo.status === "none" || todo.status === "todo") {
                  doc = updateTodo(doc, todoNum, { status: "in_progress" });
                }
                await save();

                return jsonResponse({
                  ok: true,
                  branch: branchName,
                  worktree: `.worktrees/${branchName}`,
                });
              }

              case "removeBranch": {
                const todoNum = body.number;
                const todo = doc.todos.find((t) => t.number === todoNum);
                if (!todo) return jsonResponse({ error: `Todo #${todoNum} not found` }, 400);

                if (!todo.branch) {
                  return jsonResponse({ error: `Todo #${todoNum} has no branch` }, 400);
                }

                const branchName = todo.branch;
                const worktreePath = join(projectPath, ".worktrees", branchName);

                if (existsSync(worktreePath)) {
                  const result = Bun.spawnSync(["git", "worktree", "remove", worktreePath], {
                    cwd: projectPath,
                    stderr: "pipe",
                    stdout: "pipe",
                  });

                  if (result.exitCode !== 0) {
                    const stderr = result.stderr.toString().trim();
                    return jsonResponse({ error: `Failed to remove worktree: ${stderr}` }, 500);
                  }
                }

                doc = clearBranch(doc, todoNum);
                await save();

                return jsonResponse({ ok: true, branch: branchName });
              }

              case "updateProject": {
                doc = updateProject(doc, body.updates ?? {});
                await save();
                await syncConfig(configPath, {
                  prefix: doc.prefix,
                  name: doc.name,
                });
                return jsonResponse({ ok: true });
              }

              case "addMember": {
                doc = addMember(
                  doc,
                  body.name ?? "Unnamed",
                  body.role ?? "member",
                  body.email ?? null,
                );
                await save();
                const added = doc.members[doc.members.length - 1]!;
                return jsonResponse({ ok: true, id: added.id });
              }

              case "removeMember": {
                const member = findMember(doc, body.memberId ?? body.name ?? "");
                if (!member) return jsonResponse({ error: "Member not found" }, 400);
                doc = removeMember(doc, member.id);
                await save();
                return jsonResponse({ ok: true });
              }

              case "updateMember": {
                const target = findMember(doc, body.memberId ?? body.name ?? "");
                if (!target) return jsonResponse({ error: "Member not found" }, 400);
                doc = updateMember(doc, target.id, body.updates ?? {});
                await save();
                return jsonResponse({ ok: true });
              }

              case "updateInbox": {
                await ensureInboxFiles(todoDir);
                await writeInbox(todoDir, body.text ?? "");
                return jsonResponse({ ok: true });
              }

              case "processInbox": {
                await ensureInboxFiles(todoDir);
                // Start brain processing in background — events stream over WebSocket
                processInbox(projectPath, (event) => {
                  broadcast(JSON.stringify(event));
                }).catch((err) => {
                  broadcast(
                    JSON.stringify({
                      type: "brain:error",
                      message: err instanceof Error ? err.message : String(err),
                    }),
                  );
                  broadcast(
                    JSON.stringify({
                      type: "brain:done",
                      processed: 0,
                      tasks: [],
                    }),
                  );
                });
                return jsonResponse({ ok: true, message: "Brain processing started" });
              }

              default:
                return jsonResponse({ error: `Unknown action: ${body.action}` }, 400);
            }
          } catch (e: unknown) {
            const msg = e instanceof Error ? e.message : String(e);
            return jsonResponse({ error: msg }, 400);
          }
        },
      },
    },

    fetch(req) {
      if (req.method === "OPTIONS") {
        return new Response(null, { headers: CORS_HEADERS });
      }

      // WebSocket upgrade for /ws
      const url = new URL(req.url);
      if (url.pathname === "/ws") {
        const upgraded = server.upgrade(req);
        if (upgraded) return undefined as unknown as Response;
        return new Response("WebSocket upgrade failed", { status: 400 });
      }

      if (opts.dev) {
        url.port = String(opts.vitePort ?? 5173);
        return Response.redirect(url.toString(), 307);
      }

      return serveStatic(url.pathname, distDir);
    },

    websocket: {
      open(ws) {
        wsClients.add(ws);
      },
      message() {
        // Client-to-server messages not used; refresh is server-pushed
      },
      close(ws) {
        wsClients.delete(ws);
      },
    },
  });

  return server;
}

async function serveStatic(pathname: string, distDir: string) {
  const filePath = pathname === "/" ? "/index.html" : pathname;
  let file = Bun.file(join(distDir, filePath));
  if (await file.exists()) return new Response(file);

  file = Bun.file(join(distDir, "index.html"));
  if (await file.exists()) return new Response(file);

  return new Response("Not Found", { status: 404 });
}

/* ── Run standalone ── */
if (import.meta.main) {
  const { findProject } = await import("../lib/project.js");
  const paths = findProject();
  if (!paths) {
    console.error("Not in an agt project. Run 'agt init' first.");
    process.exit(1);
  }
  const dev = process.env.AGT_DEV === "1";
  const vitePort = parseInt(process.env.AGT_VITE_PORT || "5173", 10);
  const server = await startServer(paths.root, 3000, dev ? { dev: true, vitePort } : {});
  console.log(`API server: http://localhost:${server.port}`);
}
