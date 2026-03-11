/**
 * `agt browser` — Open the web dashboard in a browser.
 * Starts a local Bun server and opens the default browser.
 */

import type { Command } from "commander";
import { findProject } from "../../lib/project.js";
import { error } from "../output.js";

export function registerBrowser(program: Command): void {
  program
    .command("browser")
    .description("Open the web dashboard in a browser")
    .option("--port <port>", "Port to listen on", "3000")
    .option("--no-open", "Don't automatically open the browser")
    .action(async (_opts: { port: string; open: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      error(
        "Web dashboard not yet implemented. Coming in a future version.",
      );
    });
}
