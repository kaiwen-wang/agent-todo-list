/**
 * `agt list` — List todos with optional filters.
 */

import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc } from "../../lib/storage.js";
import { queryTodos, type TodoFilter } from "../../lib/queries.js";
import type { Status, Priority } from "../../lib/schema.js";
import { error, formatTodoLine } from "../output.js";

export function registerList(program: Command): void {
  program
    .command("list")
    .alias("ls")
    .description("List todos")
    .option("-s, --status <status>", "Filter by status")
    .option("-p, --priority <priority>", "Filter by priority")
    .option("-a, --assignee <name>", "Filter by assignee")
    .option("-q, --search <query>", "Search title and description")
    .option("--all", "Include archived todos")
    .option("--json", "Output as JSON")
    .action(
      async (opts: {
        status?: string;
        priority?: string;
        assignee?: string;
        search?: string;
        all?: boolean;
        json?: boolean;
      }) => {
        const paths = findProject();
        if (!paths) error("Not in an agt project. Run 'agt init' first.");

        const config = await readConfig(paths.configPath);
        const doc = await loadDoc(paths.dataPath);
        if (!doc) error("Project data not found. Reinitialize with 'agt init'.");

        const filter: TodoFilter = {};

        if (opts.status) {
          filter.status = opts.status as Status;
        } else if (opts.all) {
          // No status filter — include everything
          filter.status = [
            "none",
            "todo",
            "needs_elaboration",
            "in_progress",
            "completed",
            "archived",
            "wont_do",
          ];
        }

        if (opts.priority) filter.priority = opts.priority as Priority;
        if (opts.assignee) filter.assignee = opts.assignee;
        if (opts.search) filter.search = opts.search;

        const todos = queryTodos(doc, filter);

        if (opts.json) {
          const out = todos.map((t) => ({
            ref: `${config.prefix}-${t.number}`,
            number: t.number,
            title: t.title,
            status: t.status,
            priority: t.priority,
            assignee: t.assignee,
          }));
          console.log(JSON.stringify(out, null, 2));
          return;
        }

        if (todos.length === 0) {
          console.log("No todos found.");
          return;
        }

        for (const todo of todos) {
          console.log(formatTodoLine(todo, config.prefix));
        }
      },
    );
}
