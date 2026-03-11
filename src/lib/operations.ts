/**
 * Automerge mutation functions for the Project document.
 * All writes to the CRDT go through these functions.
 */

import * as Automerge from "@automerge/automerge";
import { Counter } from "@automerge/automerge";
import type {
  Project,
  Todo,
  Status,
  Priority,
  MemberRole,
} from "./schema.js";
import { CURRENT_SCHEMA_VERSION } from "./schema.js";

type Doc = Automerge.Doc<Project>;

/** Create a brand new empty project document */
export function createProject(
  prefix: string,
  name: string,
  ownerName: string,
): Doc {
  const now = new Date().toISOString();
  const ownerId = crypto.randomUUID();

  return Automerge.from<Project>({
    _version: CURRENT_SCHEMA_VERSION,
    id: crypto.randomUUID(),
    prefix: prefix.toUpperCase(),
    name,
    description: "",
    counter: new Counter(0),
    createdAt: now,
    members: [
      {
        id: ownerId,
        name: ownerName,
        email: null,
        role: "owner",
      },
    ],
    todos: [],
  });
}

/** Add a new todo and return the updated doc + the created todo's number */
export function addTodo(
  doc: Doc,
  opts: {
    title: string;
    description?: string;
    status?: Status;
    priority?: Priority;
    assignee?: string | null;
    tags?: string[];
    createdBy?: string;
  },
): { doc: Doc; number: number } {
  let todoNumber = 0;

  const nextDoc = Automerge.change(doc, (d) => {
    d.counter.increment(1);
    todoNumber = d.counter.value;
    const now = new Date().toISOString();

    d.todos.push({
      id: crypto.randomUUID(),
      number: todoNumber,
      title: opts.title,
      description: opts.description ?? "",
      status: opts.status ?? "todo",
      priority: opts.priority ?? "medium",
      assignee: opts.assignee ?? null,
      tags: opts.tags ?? [],
      createdAt: now,
      updatedAt: now,
      createdBy: opts.createdBy ?? d.members[0]?.id ?? "unknown",
    });
  });

  return { doc: nextDoc, number: todoNumber };
}

/** Update fields on an existing todo (found by number) */
export function updateTodo(
  doc: Doc,
  todoNumber: number,
  updates: Partial<
    Pick<
      Todo,
      "title" | "description" | "status" | "priority" | "assignee" | "tags"
    >
  >,
): Doc {
  return Automerge.change(doc, (d) => {
    const todo = d.todos.find((t) => t.number === todoNumber);
    if (!todo) throw new Error(`Todo #${todoNumber} not found`);

    if (updates.title !== undefined) todo.title = updates.title;
    if (updates.description !== undefined)
      todo.description = updates.description;
    if (updates.status !== undefined) todo.status = updates.status;
    if (updates.priority !== undefined) todo.priority = updates.priority;
    if (updates.assignee !== undefined) todo.assignee = updates.assignee;
    if (updates.tags !== undefined) {
      // Replace tags array — use splice since Automerge proxies don't support .length = 0
      todo.tags.splice(0, todo.tags.length);
      for (const tag of updates.tags) {
        todo.tags.push(tag);
      }
    }
    todo.updatedAt = new Date().toISOString();
  });
}

/** Delete a todo by number (hard delete) */
export function deleteTodo(doc: Doc, todoNumber: number): Doc {
  return Automerge.change(doc, (d) => {
    const idx = d.todos.findIndex((t) => t.number === todoNumber);
    if (idx === -1) throw new Error(`Todo #${todoNumber} not found`);
    d.todos.splice(idx, 1);
  });
}

/** Add a member to the project */
export function addMember(
  doc: Doc,
  name: string,
  role: MemberRole = "member",
  email: string | null = null,
): Doc {
  return Automerge.change(doc, (d) => {
    d.members.push({
      id: crypto.randomUUID(),
      name,
      email,
      role,
    });
  });
}

/** Update project metadata */
export function updateProject(
  doc: Doc,
  updates: Partial<Pick<Project, "name" | "description" | "prefix">>,
): Doc {
  return Automerge.change(doc, (d) => {
    if (updates.name !== undefined) d.name = updates.name;
    if (updates.description !== undefined) d.description = updates.description;
    if (updates.prefix !== undefined) d.prefix = updates.prefix.toUpperCase();
  });
}
