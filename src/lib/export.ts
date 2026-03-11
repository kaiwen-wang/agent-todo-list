/**
 * Serialization helpers for the Project document.
 */

import type * as Automerge from "@automerge/automerge";
import type { Project } from "./schema.js";

type Doc = Automerge.Doc<Project>;

/** Serialize the project as a JSON-serializable object. */
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
      labels: t.labels ? [...t.labels] : [],
      assignee: t.assignee,
      assigneeName: t.assignee ? findMemberName(doc, t.assignee) : null,
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
