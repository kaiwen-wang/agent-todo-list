/**
 * `agt unassign` — Remove the assignee from a todo.
 */

import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { unassignTodo } from "../../lib/operations.js";
import { parseTodoRef, findTodoByNumber } from "../../lib/queries.js";
import { error, success } from "../output.js";

export function registerUnassign(program: Command): void {
  program
    .command("unassign")
    .description("Remove the assignee from a todo")
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

      const todo = findTodoByNumber(doc, num);
      if (!todo) error(`Todo ${config.prefix}-${num} not found.`);

      if (!todo.assignee) {
        error(`Todo ${config.prefix}-${num} is not assigned to anyone.`);
      }

      const oldAssigneeName =
        doc.members.find((m) => m.id === todo.assignee)?.name ?? todo.assignee;

      doc = unassignTodo(doc, num);
      await saveDoc(paths.dataPath, doc);

      const todoRef = `${config.prefix}-${num}`;
      if (opts.json) {
        console.log(
          JSON.stringify({
            ref: todoRef,
            previousAssignee: oldAssigneeName,
            assignee: null,
          }),
        );
      } else {
        success(`Unassigned ${todoRef} (was ${oldAssigneeName})`);
      }
    });
}
