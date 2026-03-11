/**
 * agt browser server — minimal local dashboard.
 *
 * Two API endpoints:
 *   GET  /api/project — returns everything as JSON
 *   POST /api/change  — applies a single mutation
 *
 * Serves the Vue dist/ as static files. That's it.
 */

import * as Automerge from "@automerge/automerge";
import { join } from "node:path";
import type { Project } from "../lib/schema.js";
import { loadDoc, saveDoc } from "../lib/storage.js";
import { addTodo, updateTodo, deleteTodo, updateProject, addMember, removeMember, updateMember } from "../lib/operations.js";
import { findMember } from "../lib/queries.js";
import { toJSON } from "../lib/export.js";
import { syncConfig } from "../lib/project.js";

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
  /** When true, non-API requests redirect to the Vite dev server instead of serving dist/. */
  dev?: boolean;
  /** Port the Vite dev server is on (default 5173). Only used when dev=true. */
  vitePort?: number;
}

export async function startServer(projectPath: string, port = 3000, opts: ServerOptions = {}) {
  const dataPath = join(projectPath, ".todo", "data.automerge");
  const configPath = join(projectPath, ".todo", "config.toml");
  const distDir = join(import.meta.dir, "..", "web", "dist");
  let doc: Doc | null = await loadDoc(dataPath);

  if (!doc) throw new Error(`Cannot load project data from ${dataPath}`);

  async function save() {
    if (doc) await saveDoc(dataPath, doc);
  }

  const server = Bun.serve({
    port,

    routes: {
      "/api/project": {
        async GET() {
          doc = await loadDoc(dataPath);
          if (!doc) return jsonResponse({ error: "No data" }, 500);
          return jsonResponse(toJSON(doc));
        },
      },

      "/api/change": {
        async POST(req) {
          doc = await loadDoc(dataPath);
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
                  assignee: body.assignee,

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

              case "updateProject": {
                doc = updateProject(doc, body.updates ?? {});
                await save();
                // Keep config.toml in sync with the CRDT
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
      // CORS preflight
      if (req.method === "OPTIONS") {
        return new Response(null, { headers: CORS_HEADERS });
      }

      // In dev mode, redirect non-API requests to the Vite dev server
      if (opts.dev) {
        const url = new URL(req.url);
        url.port = String(opts.vitePort ?? 5173);
        return Response.redirect(url.toString(), 307);
      }

      const url = new URL(req.url);
      return serveStatic(url.pathname, distDir);
    },
  });

  return server;
}

async function serveStatic(pathname: string, distDir: string) {
  const filePath = pathname === "/" ? "/index.html" : pathname;
  let file = Bun.file(join(distDir, filePath));
  if (await file.exists()) return new Response(file);

  // SPA fallback
  file = Bun.file(join(distDir, "index.html"));
  if (await file.exists()) return new Response(file);

  return new Response("Not Found", { status: 404 });
}

/* ── Run standalone when executed directly (`bun src/server/index.ts`) ── */
if (import.meta.main) {
  const { findProject } = await import("../lib/project.js");
  const paths = findProject();
  if (!paths) {
    console.error("Not in an agt project. Run 'agt init' first.");
    process.exit(1);
  }
  const server = await startServer(paths.root);
  console.log(`Dashboard: http://localhost:${server.port}`);
}
