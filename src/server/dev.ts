/**
 * Dev server — starts the API backend and Vite dev server together.
 *
 * Usage: bun src/server/dev.ts
 *   or:  bun run dev
 *
 * - API server on port 3000 (reads .todo/data.automerge)
 * - Vite dev server on port 5173 (HMR for Vue, proxies /api to 3000)
 */

import { findProject } from "../lib/project.js";
import { startServer } from "./index.js";

const paths = findProject();
if (!paths) {
  console.error("Not in an agt project. Run 'agt init' first.");
  process.exit(1);
}

// Start the API server
const server = await startServer(paths.root, 3000);
console.log("API server ready on http://localhost:3000\n");

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
