/**
 * Dev server — starts the API backend and Vite dev server together.
 *
 * Usage: bun src/server/dev.ts
 *   or:  bun run dev
 *
 * - API server on port 3000 (with --watch for auto-reload on backend changes)
 * - Vite dev server on port 5173 (HMR for Vue, proxies /api to 3000)
 * - Open http://localhost:5173 in your browser
 */

import { findProject } from "../lib/project.js";

const VITE_PORT = 5173;

const paths = findProject();
if (!paths) {
  console.error("Not in an agt project. Run 'agt init' first.");
  process.exit(1);
}

// Spawn API server with --watch so it auto-restarts on backend changes
const api = Bun.spawn(["bun", "--watch", "src/server/index.ts"], {
  cwd: paths.root,
  stdin: "inherit",
  stdout: "inherit",
  stderr: "inherit",
  env: {
    ...process.env,
    AGT_DEV: "1",
    AGT_VITE_PORT: String(VITE_PORT),
  },
});

// Spawn Vite dev server
const vite = Bun.spawn(["bun", "run", "--cwd", "src/web", "dev"], {
  cwd: paths.root,
  stdin: "inherit",
  stdout: "inherit",
  stderr: "inherit",
});

console.log("Dev mode: API server (--watch) on :3000, Vite on :5173");

// Clean shutdown on Ctrl+C
function cleanup() {
  api.kill();
  vite.kill();
  process.exit(0);
}

process.on("SIGINT", cleanup);
process.on("SIGTERM", cleanup);

await Promise.race([api.exited, vite.exited]);
