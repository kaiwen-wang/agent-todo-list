/**
 * `agt serve` — Start the web dashboard server.
 * Stub — server implementation comes in Phase 3.
 */

import type { Command } from "commander";
import { findProject } from "../../lib/project.js";
import { error } from "../output.js";

export function registerServe(program: Command): void {
  program
    .command("serve")
    .description("Start the web dashboard server")
    .option("--port <port>", "Port to listen on", "3000")
    .action(async (_opts: { port: string }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      error(
        "Server not yet implemented. Coming in a future version.",
      );
    });
}
