/**
 * Dev server — starts the API backend and Vite dev server together.
 *
 * Usage: bun src/server/dev.ts
 *   or:  bun run dev
 *
 * - API server on port 3000 (handles /api/* requests only)
 * - Vite dev server on port 5173 (HMR for Vue, proxies /api to 3000)
 * - Open http://localhost:5173 in your browser
 */

import { findProject } from "../lib/project.js";
import { startServer } from "./index.js";

const VITE_PORT = 5173;

const paths = findProject();
if (!paths) {
  console.error("Not in an agt project. Run 'agt init' first.");
  process.exit(1);
}

// Start the API server (non-API requests redirect to Vite)
const server = await startServer(paths.root, 3000, {
  dev: true,
  vitePort: VITE_PORT,
});

// Spawn Vite dev server
const vite = Bun.spawn(["bun", "run", "--cwd", "src/web", "dev"], {
  cwd: paths.root,
  stdin: "inherit",
  stdout: "inherit",
  stderr: "inherit",
});

// Clean shutdown on Ctrl+C
process.on("SIGINT", () => {
  vite.kill();
  server.stop();
  process.exit(0);
});

process.on("SIGTERM", () => {
  vite.kill();
  server.stop();
  process.exit(0);
});

await vite.exited;
