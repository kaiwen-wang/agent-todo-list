/**
 * Serialization helpers for the Project document.
 * Converts Automerge doc → plain JSON-safe objects, adding computed fields.
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
    members: doc.members.map((m) => {
      const member: Record<string, unknown> = {
        id: m.id,
        name: m.name,
        email: m.email,
        role: m.role,
      };
      if (m.agentProvider) member.agentProvider = m.agentProvider;
      if (m.agentModel) member.agentModel = m.agentModel;
      return member;
    }),
    todos: doc.todos.map((t) => ({
      id: t.id,
      ref: `${doc.prefix}-${t.number}`, // computed
      number: t.number,
      title: t.title,
      description: t.description,
      status: t.status,
      priority: t.priority,
      difficulty: t.difficulty ?? "none",
      labels: t.labels ? [...t.labels] : [],
      assignee: t.assignee,
      assigneeName: t.assignee ? findMemberName(doc, t.assignee) : null, // computed
      branch: t.branch ?? null,
      comments: (t.comments ?? []).map((c) => ({
        id: c.id,
        author: c.author,
        authorName: c.authorName,
        text: c.text,
        createdAt: c.createdAt,
      })),
      createdAt: t.createdAt,
      updatedAt: t.updatedAt,
      createdBy: t.createdBy,
      platform: t.platform ?? "unknown",
    })),
    auditLog: (doc.auditLog ?? []).map((e) => ({
      id: e.id,
      action: e.action,
      actor: e.actor,
      actorName: e.actorName,
      target: e.target,
      details: e.details,
      timestamp: e.timestamp,
    })),
  };
}

function findMemberName(doc: Doc, memberId: string): string {
  const member = doc.members.find((m) => m.id === memberId);
  return member ? member.name : memberId;
}
