/**
 * `agt add` — Create a new todo.
 */

import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { addTodo } from "../../lib/operations.js";
import { findMember } from "../../lib/queries.js";
import type { Status, Priority, Difficulty, Label } from "../../lib/schema.js";
import { STATUSES, PRIORITIES, DIFFICULTIES, LABELS } from "../../lib/schema.js";
import { error, success } from "../output.js";

export function registerAdd(program: Command): void {
  program
    .command("add")
    .description("Add a new todo")
    .argument("<title>", "Todo title")
    .option("-d, --description <text>", "Description (markdown)")
    .option("-s, --status <status>", "Initial status", "todo")
    .option("-p, --priority <priority>", "Priority level", "none")
    .option("--difficulty <difficulty>", "Difficulty (easy, medium, hard)", "none")
    .option("-a, --assignee <name>", "Assignee name")
    .option("-l, --labels <labels>", "Labels (comma-separated: bug,new_feature,feature_plus)")
    .option("--json", "Output as JSON")
    .action(
      async (
        title: string,
        opts: {
          description?: string;
          status: string;
          priority: string;
          difficulty: string;
          assignee?: string;
          labels?: string;
          json?: boolean;
        },
      ) => {
        const paths = findProject();
        if (!paths) error("Not in an agt project. Run 'agt init' first.");

        const config = await readConfig(paths.configPath);
        let doc = await loadDoc(paths.dataPath);
        if (!doc) error("Project data not found. Reinitialize with 'agt init'.");

        // Validate status
        if (!STATUSES.includes(opts.status as Status)) {
          error(`Invalid status "${opts.status}". Valid: ${STATUSES.join(", ")}`);
        }

        // Validate priority
        if (!PRIORITIES.includes(opts.priority as Priority)) {
          error(`Invalid priority "${opts.priority}". Valid: ${PRIORITIES.join(", ")}`);
        }

        // Validate difficulty
        if (!DIFFICULTIES.includes(opts.difficulty as Difficulty)) {
          error(`Invalid difficulty "${opts.difficulty}". Valid: ${DIFFICULTIES.join(", ")}`);
        }

        // Validate labels
        let parsedLabels: Label[] | undefined;
        if (opts.labels) {
          parsedLabels = opts.labels.split(",").map((l) => l.trim()) as Label[];
          for (const label of parsedLabels) {
            if (!LABELS.includes(label)) {
              error(`Invalid label "${label}". Valid: ${LABELS.join(", ")}`);
            }
          }
        }

        // Resolve assignee to member ID
        let assigneeId: string | null = null;
        if (opts.assignee) {
          const member = findMember(doc, opts.assignee);
          if (!member) {
            error(
              `Member "${opts.assignee}" not found. Add them first or use an existing member name.`,
            );
          }
          assigneeId = member.id;
        }

        const result = addTodo(doc, {
          title,
          description: opts.description,
          status: opts.status as Status,
          priority: opts.priority as Priority,
          difficulty: opts.difficulty as Difficulty,
          labels: parsedLabels,
          assignee: assigneeId,
          platform: "cli",
        });

        await saveDoc(paths.dataPath, result.doc);

        const ref = `${config.prefix}-${result.number}`;

        if (opts.json) {
          console.log(JSON.stringify({ ref, number: result.number, title }));
        } else {
          success(`Created ${ref}: ${title}`);
        }
      },
    );
}
