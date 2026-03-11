/**
 * Terminal formatting helpers for CLI output.
 * Uses chalk for colors.
 */

import chalk from "chalk";
import type { Todo, Status, Priority } from "../lib/schema.js";

const STATUS_COLORS: Record<Status, (s: string) => string> = {
  backlog: chalk.gray,
  todo: chalk.white,
  in_progress: chalk.cyan,
  done: chalk.green,
  archived: chalk.dim,
};

const PRIORITY_COLORS: Record<Priority, (s: string) => string> = {
  low: chalk.dim,
  medium: chalk.white,
  high: chalk.yellow,
  urgent: chalk.red.bold,
};

const STATUS_ICONS: Record<Status, string> = {
  backlog: " ",
  todo: " ",
  in_progress: "*",
  done: "x",
  archived: "-",
};

const PRIORITY_LABELS: Record<Priority, string> = {
  low: "low",
  medium: "med",
  high: "high",
  urgent: "URGENT",
};

/** Format a single todo as a one-line summary for list view. */
export function formatTodoLine(todo: Todo, prefix: string): string {
  const ref = chalk.bold(`${prefix}-${todo.number}`);
  const icon = STATUS_ICONS[todo.status];
  const statusColor = STATUS_COLORS[todo.status];
  const title = statusColor(todo.title);
  const priority =
    todo.priority !== "medium"
      ? " " + PRIORITY_COLORS[todo.priority](PRIORITY_LABELS[todo.priority])
      : "";
  const tags =
    todo.tags.length > 0
      ? " " + todo.tags.map((t) => chalk.dim(`#${t}`)).join(" ")
      : "";

  return `[${icon}] ${ref} ${title}${priority}${tags}`;
}

/** Format a todo's full detail view. */
export function formatTodoDetail(
  todo: Todo,
  prefix: string,
  memberName?: string,
): string {
  const lines: string[] = [];
  const ref = `${prefix}-${todo.number}`;

  lines.push(chalk.bold(`${ref}: ${todo.title}`));
  lines.push("");
  lines.push(
    `  Status:   ${STATUS_COLORS[todo.status](todo.status.replace("_", " "))}`,
  );
  lines.push(
    `  Priority: ${PRIORITY_COLORS[todo.priority](PRIORITY_LABELS[todo.priority])}`,
  );

  if (todo.assignee) {
    lines.push(`  Assignee: ${memberName ?? todo.assignee}`);
  }

  if (todo.tags.length > 0) {
    lines.push(`  Tags:     ${todo.tags.join(", ")}`);
  }

  lines.push(`  Created:  ${formatDate(todo.createdAt)}`);
  if (todo.updatedAt !== todo.createdAt) {
    lines.push(`  Updated:  ${formatDate(todo.updatedAt)}`);
  }

  if (todo.description) {
    lines.push("");
    lines.push(todo.description);
  }

  return lines.join("\n");
}

/** Format an ISO date string to a shorter form. */
function formatDate(iso: string): string {
  const d = new Date(iso);
  return d.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** Print an error message and exit with code 1. */
export function error(msg: string): never {
  console.error(chalk.red(`error: ${msg}`));
  process.exit(1);
}

/** Print a warning message. */
export function warn(msg: string): void {
  console.error(chalk.yellow(`warning: ${msg}`));
}

/** Print a success message. */
export function success(msg: string): void {
  console.log(chalk.green(msg));
}
