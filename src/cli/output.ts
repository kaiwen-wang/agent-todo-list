/**
 * Terminal formatting helpers for CLI output.
 * Uses chalk for colors.
 */

import chalk from "chalk";
import type { Todo, Member, Status, Priority, Difficulty, Timestamp } from "../lib/schema.js";
import { LABEL_DISPLAY } from "../lib/schema.js";

const STATUS_COLORS: Record<Status, (s: string) => string> = {
  none: chalk.gray,
  todo: chalk.white,
  in_progress: chalk.cyan,
  completed: chalk.green,
  archived: chalk.dim,
  wont_do: chalk.strikethrough,
  needs_elaboration: chalk.magenta,
};

const PRIORITY_COLORS: Record<Priority, (s: string) => string> = {
  none: chalk.gray,
  low: chalk.dim,
  medium: chalk.white,
  high: chalk.yellow,
  urgent: chalk.red.bold,
};

const STATUS_ICONS: Record<Status, string> = {
  none: " ",
  todo: " ",
  in_progress: "*",
  completed: "x",
  archived: "-",
  wont_do: "~",
  needs_elaboration: "?",
};

const PRIORITY_LABELS: Record<Priority, string> = {
  none: "",
  low: "low",
  medium: "med",
  high: "high",
  urgent: "URGENT",
};

const DIFFICULTY_COLORS: Record<Difficulty, (s: string) => string> = {
  none: chalk.gray,
  easy: chalk.green,
  medium: chalk.yellow,
  hard: chalk.red,
};

const DIFFICULTY_LABELS: Record<Difficulty, string> = {
  none: "",
  easy: "low",
  medium: "medium",
  hard: "high",
};

/** Format a single todo as a one-line summary for list view. */
export function formatTodoLine(todo: Todo, prefix: string, members?: Member[]): string {
  const ref = chalk.bold(`${prefix}-${todo.number}`);
  const icon = STATUS_ICONS[todo.status];
  const statusColor = STATUS_COLORS[todo.status];
  const title = statusColor(todo.title);
  const priority =
    todo.priority !== "medium" && todo.priority !== "none"
      ? " " + PRIORITY_COLORS[todo.priority](PRIORITY_LABELS[todo.priority])
      : "";
  const difficulty =
    todo.difficulty && todo.difficulty !== "none"
      ? " " + DIFFICULTY_COLORS[todo.difficulty](DIFFICULTY_LABELS[todo.difficulty])
      : "";
  const assignee =
    todo.assignee && members
      ? " " + chalk.dim(`@${members.find((m) => m.id === todo.assignee)?.name ?? "unknown"}`)
      : "";
  return `[${icon}] ${ref} ${title}${priority}${difficulty}${assignee}`;
}

/** Format a todo's full detail view. */
export function formatTodoDetail(todo: Todo, prefix: string, memberName?: string): string {
  const lines: string[] = [];
  const ref = `${prefix}-${todo.number}`;

  lines.push(chalk.bold(`${ref}: ${todo.title}`));
  lines.push("");
  lines.push(`  Status:   ${STATUS_COLORS[todo.status](todo.status.replace("_", " "))}`);
  lines.push(`  Priority: ${PRIORITY_COLORS[todo.priority](PRIORITY_LABELS[todo.priority])}`);

  if (todo.difficulty && todo.difficulty !== "none") {
    lines.push(
      `  Difficulty: ${DIFFICULTY_COLORS[todo.difficulty](DIFFICULTY_LABELS[todo.difficulty])}`,
    );
  }

  if (todo.assignee) {
    lines.push(`  Assignee: ${memberName ?? todo.assignee}`);
  }

  if (todo.branch) {
    lines.push(`  Branch:   ${chalk.cyan(todo.branch)}`);
  }

  if (todo.labels && todo.labels.length > 0) {
    const labelStrs = todo.labels.map((l) => LABEL_DISPLAY[l] ?? l);
    lines.push(`  Labels:   ${labelStrs.join(", ")}`);
  }

  lines.push(`  Created:  ${formatDate(todo.createdAt)}`);
  if (todo.updatedAt !== todo.createdAt) {
    lines.push(`  Updated:  ${formatDate(todo.updatedAt)}`);
  }

  if (todo.description) {
    lines.push("");
    lines.push(todo.description);
  }

  // Comments
  const comments = todo.comments ?? [];
  if (comments.length > 0) {
    lines.push("");
    lines.push(chalk.dim(`--- Comments (${comments.length}) ---`));
    for (const c of comments) {
      lines.push(`  ${chalk.bold(c.authorName)} ${chalk.dim(formatDate(c.createdAt))}`);
      lines.push(`  ${c.text}`);
      lines.push("");
    }
  }

  return lines.join("\n");
}

/** Format a timestamp (Unix ms or ISO string) to a shorter form. */
export function formatDate(ts: Timestamp | string): string {
  const d = typeof ts === "number" ? new Date(ts) : new Date(ts);
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
