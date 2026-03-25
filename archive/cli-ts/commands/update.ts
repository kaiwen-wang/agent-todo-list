/**
 * `agt update` — Update fields on an existing todo.
 */

import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { updateTodo } from "../../lib/operations.js";
import { parseTodoRef, findTodoByNumber, findMember } from "../../lib/queries.js";
import type { Status, Priority, Difficulty, Label, Todo } from "../../lib/schema.js";
import { STATUSES, PRIORITIES, DIFFICULTIES, LABELS } from "../../lib/schema.js";
import { error, success } from "../output.js";

export function registerUpdate(program: Command): void {
  program
    .command("update")
    .description("Update a todo")
    .argument("<ref>", "Todo reference (e.g. ABC-1 or 1)")
    .option("--title <title>", "New title")
    .option("-d, --description <text>", "New description")
    .option("-s, --status <status>", "New status")
    .option("-p, --priority <priority>", "New priority")
    .option("--difficulty <difficulty>", "New difficulty (easy, medium, hard)")
    .option("-a, --assignee <name>", "New assignee")
    .option("-l, --labels <labels>", "Labels (comma-separated: bug,new_feature,feature_plus)")
    .option("--json", "Output as JSON")
    .action(
      async (
        ref: string,
        opts: {
          title?: string;
          description?: string;
          status?: string;
          priority?: string;
          difficulty?: string;
          assignee?: string;
          labels?: string;
          json?: boolean;
        },
      ) => {
        const paths = findProject();
        if (!paths) error("Not in an agt project. Run 'agt init' first.");

        const config = await readConfig(paths.configPath);
        let doc = await loadDoc(paths.dataPath);
        if (!doc) error("Project data not found.");

        const num = parseTodoRef(ref, config.prefix);
        if (num === null) {
          error(`Invalid todo reference: "${ref}".`);
        }

        const existing = findTodoByNumber(doc, num);
        if (!existing) {
          error(`Todo ${config.prefix}-${num} not found.`);
        }

        // Build updates
        const updates: Partial<
          Pick<
            Todo,
            "title" | "description" | "status" | "priority" | "difficulty" | "labels" | "assignee"
          >
        > = {};

        if (opts.title !== undefined) updates.title = opts.title;
        if (opts.description !== undefined) updates.description = opts.description;

        if (opts.status !== undefined) {
          if (!STATUSES.includes(opts.status as Status)) {
            error(`Invalid status "${opts.status}". Valid: ${STATUSES.join(", ")}`);
          }
          updates.status = opts.status as Status;
        }

        if (opts.priority !== undefined) {
          if (!PRIORITIES.includes(opts.priority as Priority)) {
            error(`Invalid priority "${opts.priority}". Valid: ${PRIORITIES.join(", ")}`);
          }
          updates.priority = opts.priority as Priority;
        }

        if (opts.difficulty !== undefined) {
          if (!DIFFICULTIES.includes(opts.difficulty as Difficulty)) {
            error(`Invalid difficulty "${opts.difficulty}". Valid: ${DIFFICULTIES.join(", ")}`);
          }
          updates.difficulty = opts.difficulty as Difficulty;
        }

        if (opts.assignee !== undefined) {
          const member = findMember(doc, opts.assignee);
          if (!member) {
            error(`Member "${opts.assignee}" not found.`);
          }
          updates.assignee = member.id;
        }

        if (opts.labels !== undefined) {
          const parsed = opts.labels.split(",").map((l) => l.trim()) as Label[];
          for (const label of parsed) {
            if (!LABELS.includes(label)) {
              error(`Invalid label "${label}". Valid: ${LABELS.join(", ")}`);
            }
          }
          updates.labels = parsed;
        }

        if (Object.keys(updates).length === 0) {
          error("No updates specified. Use --title, --status, --priority, --difficulty, etc.");
        }

        doc = updateTodo(doc, num, updates);
        await saveDoc(paths.dataPath, doc);

        const todoRef = `${config.prefix}-${num}`;
        if (opts.json) {
          console.log(JSON.stringify({ ref: todoRef, updated: Object.keys(updates) }));
        } else {
          success(`Updated ${todoRef}`);
        }
      },
    );

  // `agt archive` shorthand
  program
    .command("archive")
    .description("Archive a todo (shorthand for update --status archived)")
    .argument("<ref>", "Todo reference")
    .action(async (ref: string) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const config = await readConfig(paths.configPath);
      let doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const num = parseTodoRef(ref, config.prefix);
      if (num === null) error(`Invalid todo reference: "${ref}".`);

      const existing = findTodoByNumber(doc, num);
      if (!existing) error(`Todo ${config.prefix}-${num} not found.`);

      doc = updateTodo(doc, num, { status: "archived" });
      await saveDoc(paths.dataPath, doc);
      success(`Archived ${config.prefix}-${num}`);
    });
}
