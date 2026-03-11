/**
 * `agt assign` — Assign a todo to a member.
 */

import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { updateTodo } from "../../lib/operations.js";
import {
  parseTodoRef,
  findTodoByNumber,
  findMember,
} from "../../lib/queries.js";
import { error, success } from "../output.js";

export function registerAssign(program: Command): void {
  program
    .command("assign")
    .description("Assign a todo to a member")
    .argument("<ref>", "Todo reference (e.g. ABC-1 or 1)")
    .argument("<member>", "Member name or ID")
    .option("--json", "Output as JSON")
    .action(
      async (ref: string, memberName: string, opts: { json?: boolean }) => {
        const paths = findProject();
        if (!paths) error("Not in an agt project. Run 'agt init' first.");

        const config = await readConfig(paths.configPath);
        let doc = await loadDoc(paths.dataPath);
        if (!doc) error("Project data not found.");

        const num = parseTodoRef(ref, config.prefix);
        if (num === null) error(`Invalid todo reference: "${ref}".`);

        const todo = findTodoByNumber(doc, num);
        if (!todo) error(`Todo ${config.prefix}-${num} not found.`);

        const member = findMember(doc, memberName);
        if (!member) error(`Member "${memberName}" not found.`);

        doc = updateTodo(doc, num, { assignee: member.id });
        await saveDoc(paths.dataPath, doc);

        const todoRef = `${config.prefix}-${num}`;
        if (opts.json) {
          console.log(
            JSON.stringify({
              ref: todoRef,
              assignee: member.name,
              assigneeId: member.id,
            }),
          );
        } else {
          success(`Assigned ${todoRef} to ${member.name}`);
        }
      },
    );
}
