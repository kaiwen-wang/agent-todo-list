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
import { addTodo, updateTodo, deleteTodo } from "../lib/operations.js";
import { toJSON } from "../lib/export.js";

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

export async function startServer(projectPath: string, port = 3000) {
  const dataPath = join(projectPath, ".todo", "data.automerge");
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
                  tags: body.tags,
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

      const url = new URL(req.url);
      return serveStatic(url.pathname, distDir);
    },
  });

  console.log(`Dashboard: http://localhost:${port}`);
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
  await startServer(paths.root);
}
