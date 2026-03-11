/**
 * `agt serve` — Start the web dashboard server and open browser.
 */

import type { Command } from "commander";
import { findProject } from "../../lib/project.js";
import { error } from "../output.js";
import { startServer } from "../../server/index.js";

export function registerServe(program: Command): void {
  program
    .command("serve")
    .description("Start the web dashboard and open it in a browser")
    .option("--port <port>", "Port to listen on", "3000")
    .option("--no-open", "Don't automatically open the browser")
    .action(async (opts: { port: string; open: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const port = parseInt(opts.port, 10);
      const server = await startServer(paths!.root, port);
      const url = `http://localhost:${server.port}`;

      console.log(`\nDashboard running at ${url}`);
      console.log("Press Ctrl+C to stop.\n");

      if (opts.open) {
        const cmd = process.platform === "darwin" ? "open" : "xdg-open";
        Bun.spawn([cmd, url]);
      }

      // Keep the process running
      await new Promise(() => {});
    });
}
