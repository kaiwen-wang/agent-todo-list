/**
 * `agt delete` — Delete a todo (hard delete).
 */

import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { deleteTodo } from "../../lib/operations.js";
import { parseTodoRef, findTodoByNumber } from "../../lib/queries.js";
import { error, success } from "../output.js";

export function registerDelete(program: Command): void {
  program
    .command("delete")
    .aliases(["rm"])
    .description("Delete a todo (permanent)")
    .argument("<ref>", "Todo reference (e.g. ABC-1 or 1)")
    .option("--json", "Output as JSON")
    .action(async (ref: string, opts: { json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const config = await readConfig(paths.configPath);
      let doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const num = parseTodoRef(ref, config.prefix);
      if (num === null) error(`Invalid todo reference: "${ref}".`);

      const existing = findTodoByNumber(doc, num);
      if (!existing) error(`Todo ${config.prefix}-${num} not found.`);

      doc = deleteTodo(doc, num);
      await saveDoc(paths.dataPath, doc);

      const todoRef = `${config.prefix}-${num}`;
      if (opts.json) {
        console.log(JSON.stringify({ ref: todoRef, deleted: true }));
      } else {
        success(`Deleted ${todoRef}`);
      }
    });
}
