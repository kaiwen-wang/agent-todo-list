/**
 * Automerge mutation functions for the Project document.
 * All writes to the CRDT go through these functions.
 */

import * as Automerge from "@automerge/automerge";
import { Counter } from "@automerge/automerge";
import type {
  Member,
  Project,
  Todo,
  Status,
  Priority,
  Label,
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
    labels?: Label[];
    assignee?: string | null;
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
      priority: opts.priority ?? "none",
      labels: opts.labels ?? [],
      assignee: opts.assignee ?? null,
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
      "title" | "description" | "status" | "priority" | "labels" | "assignee"
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
    if (updates.labels !== undefined) {
      todo.labels.splice(0, todo.labels.length);
      for (const label of updates.labels) {
        todo.labels.push(label);
      }
    }
    if (updates.assignee !== undefined) todo.assignee = updates.assignee;
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

/** Remove a member by ID or name. Clears assignee on any todos assigned to them. */
export function removeMember(doc: Doc, memberId: string): Doc {
  return Automerge.change(doc, (d) => {
    const idx = d.members.findIndex((m) => m.id === memberId);
    if (idx === -1) throw new Error(`Member "${memberId}" not found`);

    // Unassign any todos assigned to this member
    for (const todo of d.todos) {
      if (todo.assignee === memberId) {
        todo.assignee = null;
      }
    }

    d.members.splice(idx, 1);
  });
}

/** Update a member's name, email, or role */
export function updateMember(
  doc: Doc,
  memberId: string,
  updates: Partial<Pick<Member, "name" | "email" | "role">>,
): Doc {
  return Automerge.change(doc, (d) => {
    const member = d.members.find((m) => m.id === memberId);
    if (!member) throw new Error(`Member "${memberId}" not found`);

    if (updates.name !== undefined) member.name = updates.name;
    if (updates.email !== undefined) member.email = updates.email;
    if (updates.role !== undefined) member.role = updates.role;
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
