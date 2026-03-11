/**
 * `agt export` — Export project data to markdown or JSON.
 */

import type { Command } from "commander";
import { findProject } from "../../lib/project.js";
import { loadDoc } from "../../lib/storage.js";
import { toMarkdown, toJSON } from "../../lib/export.js";
import { error } from "../output.js";

export function registerExport(program: Command): void {
  program
    .command("export")
    .description("Export project data")
    .option("-f, --format <format>", "Output format: md or json", "md")
    .action(async (opts: { format: string }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      switch (opts.format) {
        case "md":
        case "markdown":
          console.log(toMarkdown(doc));
          break;
        case "json":
          console.log(JSON.stringify(toJSON(doc), null, 2));
          break;
        default:
          error(`Unknown format "${opts.format}". Use "md" or "json".`);
      }
    });
}
