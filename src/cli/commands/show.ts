/**
 * `agt show` — Show full details of a single todo.
 */

import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc } from "../../lib/storage.js";
import { findTodoByNumber, parseTodoRef, findMember } from "../../lib/queries.js";
import { error, formatTodoDetail } from "../output.js";

export function registerShow(program: Command): void {
  program
    .command("show")
    .description("Show details of a todo")
    .argument("<ref>", "Todo reference (e.g. ABC-1 or 1)")
    .option("--json", "Output as JSON")
    .action(async (ref: string, opts: { json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const config = await readConfig(paths.configPath);
      const doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const num = parseTodoRef(ref, config.prefix);
      if (num === null) {
        error(`Invalid todo reference: "${ref}". Use ${config.prefix}-N or just N.`);
      }

      const todo = findTodoByNumber(doc, num);
      if (!todo) {
        error(`Todo ${config.prefix}-${num} not found.`);
      }

      if (opts.json) {
        const member = todo.assignee
          ? findMember(doc, todo.assignee)
          : undefined;
        console.log(
          JSON.stringify(
            {
              ref: `${config.prefix}-${todo.number}`,
              ...todo,
              assigneeName: member?.name ?? null,
              tags: [...todo.tags],
            },
            null,
            2,
          ),
        );
        return;
      }

      const member = todo.assignee
        ? findMember(doc, todo.assignee)
        : undefined;
      console.log(formatTodoDetail(todo, config.prefix, member?.name));
    });
}
