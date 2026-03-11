/**
 * agt server — Bun.serve() with WebSocket Automerge sync + static file serving.
 *
 * Responsibilities:
 * 1. Serve the built Vue app as static files
 * 2. WebSocket endpoint at /sync for Automerge sync protocol
 * 3. GET /api/doc — serve the full document binary for initial load
 * 4. Watch data.automerge for CLI-driven changes
 */

import * as Automerge from "@automerge/automerge";
import { watch } from "node:fs";
import { join } from "node:path";
import type { Server, ServerWebSocket } from "bun";
import type { Project } from "../lib/schema.js";
import { loadDoc, saveDoc } from "../lib/storage.js";

type Doc = Automerge.Doc<Project>;

interface PeerData {
  syncState: Automerge.SyncState;
}

export class AgtServer {
  private doc: Doc | null = null;
  private server: Server | null = null;
  private sockets = new Set<ServerWebSocket<PeerData>>();
  private dataPath: string;
  private distDir: string;
  private saveTimeout: ReturnType<typeof setTimeout> | null = null;
  private fileWatcher: ReturnType<typeof watch> | null = null;
  private ignoreSave = false; // prevent watcher loop when we save

  constructor(
    private projectPath: string,
    private port: number = 3000,
  ) {
    this.dataPath = join(projectPath, ".todo", "data.automerge");
    this.distDir = join(import.meta.dir, "..", "web", "dist");
  }

  async start(): Promise<void> {
    // Load the Automerge document
    this.doc = await loadDoc(this.dataPath);
    if (!this.doc) {
      throw new Error(`Cannot load project data from ${this.dataPath}`);
    }

    // Start file watcher for CLI-driven changes
    this.startFileWatcher();

    const self = this;

    this.server = Bun.serve<PeerData>({
      port: this.port,
      routes: {
        // Serve the full binary document for initial load
        "/api/doc": {
          GET() {
            if (!self.doc) {
              return new Response("No document loaded", { status: 500 });
            }
            const binary = Automerge.save(self.doc);
            return new Response(binary, {
              headers: { "Content-Type": "application/octet-stream" },
            });
          },
        },
      },

      fetch(req, server) {
        const url = new URL(req.url);

        // WebSocket upgrade for /sync
        if (url.pathname === "/sync") {
          const upgraded = server.upgrade<PeerData>(req, {
            data: { syncState: Automerge.initSyncState() },
          });
          if (!upgraded) {
            return new Response("WebSocket upgrade failed", { status: 400 });
          }
          return undefined;
        }

        // Serve static files from Vue dist
        return self.serveStatic(url.pathname);
      },

      websocket: {
        open(ws) {
          self.sockets.add(ws);
          // Send initial sync message to the new client
          self.sendSyncMessage(ws);
        },

        message(ws, message) {
          if (!self.doc) return;

          // Receive sync message from client
          const msg = new Uint8Array(
            message instanceof ArrayBuffer
              ? message
              : typeof message === "string"
                ? new TextEncoder().encode(message)
                : message.buffer,
          );

          const [nextDoc, nextSyncState] = Automerge.receiveSyncMessage(
            self.doc,
            ws.data.syncState,
            msg,
          );
          self.doc = nextDoc;
          ws.data.syncState = nextSyncState;

          // Send response sync message back
          self.sendSyncMessage(ws);

          // Broadcast to other connected clients
          for (const other of self.sockets) {
            if (other !== ws) {
              self.sendSyncMessage(other);
            }
          }

          // Persist to disk (debounced)
          self.scheduleSave();
        },

        close(ws) {
          self.sockets.delete(ws);
        },
      },
    });

    console.log(`Server running at http://localhost:${this.port}`);
  }

  stop(): void {
    if (this.fileWatcher) {
      this.fileWatcher.close();
      this.fileWatcher = null;
    }
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout);
      this.saveTimeout = null;
    }
    if (this.server) {
      this.server.stop();
      this.server = null;
    }
  }

  /** Send a sync message to a specific peer */
  private sendSyncMessage(ws: ServerWebSocket<PeerData>): void {
    if (!this.doc) return;

    const [nextSyncState, msg] = Automerge.generateSyncMessage(
      this.doc,
      ws.data.syncState,
    );
    ws.data.syncState = nextSyncState;

    if (msg) {
      ws.sendBinary(msg);
    }
  }

  /** Save to disk after a short delay (debounced to avoid thrashing) */
  private scheduleSave(): void {
    if (this.saveTimeout) clearTimeout(this.saveTimeout);
    this.saveTimeout = setTimeout(async () => {
      if (this.doc) {
        this.ignoreSave = true;
        await saveDoc(this.dataPath, this.doc);
        // Reset ignore flag after a short delay to let watcher fire and be ignored
        setTimeout(() => {
          this.ignoreSave = false;
        }, 100);
      }
    }, 200);
  }

  /** Watch the data file for external changes (e.g. CLI writes while server is running) */
  private startFileWatcher(): void {
    const dir = join(this.projectPath, ".todo");
    this.fileWatcher = watch(dir, async (event, filename) => {
      if (filename !== "data.automerge" || this.ignoreSave) return;

      // Reload the file and merge with our in-memory doc
      const diskDoc = await loadDoc(this.dataPath);
      if (!diskDoc || !this.doc) return;

      this.doc = Automerge.merge(this.doc, diskDoc);

      // Broadcast updated state to all connected clients
      for (const ws of this.sockets) {
        this.sendSyncMessage(ws);
      }
    });
  }

  /** Serve static files from the Vue dist directory */
  private async serveStatic(pathname: string): Promise<Response> {
    // Map / to /index.html
    let filePath = pathname === "/" ? "/index.html" : pathname;

    // Try the exact path
    let file = Bun.file(join(this.distDir, filePath));
    if (await file.exists()) {
      return new Response(file);
    }

    // SPA fallback — serve index.html for unmatched routes
    file = Bun.file(join(this.distDir, "index.html"));
    if (await file.exists()) {
      return new Response(file);
    }

    return new Response("Not Found", { status: 404 });
  }
}
