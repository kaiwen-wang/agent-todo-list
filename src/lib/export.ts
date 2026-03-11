/**
 * Export the project document to various formats.
 */

import type * as Automerge from "@automerge/automerge";
import type { Project, Todo, Status } from "./schema.js";
import { STATUS_DISPLAY, PRIORITY_DISPLAY } from "./schema.js";

type Doc = Automerge.Doc<Project>;

/** Export the project as Markdown. */
export function toMarkdown(doc: Doc): string {
  const lines: string[] = [];
  lines.push(`# ${doc.name}`);
  if (doc.description) {
    lines.push("", doc.description);
  }
  lines.push("");

  const statusOrder: Status[] = [
    "in_progress",
    "todo",
    "backlog",
    "done",
    "archived",
  ];

  for (const status of statusOrder) {
    const todos = doc.todos.filter((t) => t.status === status);
    if (todos.length === 0) continue;

    lines.push(`## ${STATUS_DISPLAY[status]}`);
    lines.push("");

    for (const todo of todos) {
      const prefix = `${doc.prefix}-${todo.number}`;
      const priority =
        todo.priority !== "medium" ? ` [${PRIORITY_DISPLAY[todo.priority]}]` : "";
      const assignee = todo.assignee
        ? ` @${findMemberName(doc, todo.assignee)}`
        : "";
      const tags =
        todo.tags.length > 0
          ? ` ${todo.tags.map((t) => `\`${t}\``).join(" ")}`
          : "";
      const check = status === "done" ? "x" : " ";

      lines.push(`- [${check}] **${prefix}** ${todo.title}${priority}${assignee}${tags}`);

      if (todo.description) {
        // Indent description under the todo
        for (const descLine of todo.description.split("\n")) {
          lines.push(`  ${descLine}`);
        }
      }
    }
    lines.push("");
  }

  return lines.join("\n");
}

/** Export the project as a JSON-serializable object. */
export function toJSON(doc: Doc): object {
  return {
    id: doc.id,
    prefix: doc.prefix,
    name: doc.name,
    description: doc.description,
    members: doc.members.map((m) => ({
      id: m.id,
      name: m.name,
      email: m.email,
      role: m.role,
    })),
    todos: doc.todos.map((t) => ({
      id: t.id,
      ref: `${doc.prefix}-${t.number}`,
      number: t.number,
      title: t.title,
      description: t.description,
      status: t.status,
      priority: t.priority,
      assignee: t.assignee,
      assigneeName: t.assignee ? findMemberName(doc, t.assignee) : null,
      tags: [...t.tags],
      createdAt: t.createdAt,
      updatedAt: t.updatedAt,
      createdBy: t.createdBy,
    })),
  };
}

function findMemberName(doc: Doc, memberId: string): string {
  const member = doc.members.find((m) => m.id === memberId);
  return member ? member.name : memberId;
}
