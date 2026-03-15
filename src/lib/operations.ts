/**
 * Automerge mutation functions for the Project document.
 * All writes to the CRDT go through these functions.
 *
 * Audit metadata is embedded in each Automerge change's `message` field
 * as a JSON string. This leverages Automerge's built-in history tracking
 * instead of maintaining a separate auditLog array on the document.
 *
 * Change message format:
 *   { action, target, actorId, actorName, details? }
 *
 * Use getAuditLog() from ./history.ts to reconstruct the audit log
 * from Automerge's change history at read time.
 */

import * as Automerge from "#automerge";
import { Counter } from "#automerge";
import type {
  Member,
  MemberId,
  Project,
  Todo,
  Status,
  Priority,
  Difficulty,
  Label,
  Platform,
  MemberRole,
  AgentProvider,
} from "./schema.js";
import { CURRENT_SCHEMA_VERSION } from "./schema.js";

type Doc = Automerge.Doc<Project>;

// ── Change message helper ───────────────────────────────────────────

/**
 * Shape of the structured metadata stored in Automerge's change.message.
 * Kept compact but readable since columnar compression handles repetition well.
 */
export interface ChangeMessage {
  action: string;
  target: string;
  actorId: string;
  actorName: string;
  details?: Record<string, unknown>;
}

/**
 * Build a JSON change message string. Called inside Automerge.change()
 * callbacks and assigned to the mutable opts.message — Automerge reads
 * the message from the opts object after the callback completes.
 */
function buildMsg(
  d: Project,
  action: string,
  actorId: MemberId,
  target: string,
  details?: Record<string, unknown>,
): string {
  const actor = d.members.find((m) => m.id === actorId);
  const msg: ChangeMessage = {
    action,
    target,
    actorId,
    actorName: actor?.name ?? actorId,
  };
  if (details && Object.keys(details).length > 0) {
    msg.details = details;
  }
  return JSON.stringify(msg);
}

/** Resolve the "current actor" — first member if none specified. */
function resolveActor(d: Project, actorId?: MemberId): MemberId {
  return actorId ?? d.members[0]?.id ?? "system";
}

// ── Project operations ──────────────────────────────────────────────

/** Create a brand new empty project document */
export function createProject(
  prefix: string,
  name: string,
  ownerName: string,
  ownerEmail?: string | null,
): Doc {
  const ownerId = crypto.randomUUID();

  return Automerge.from<Project>({
    _version: CURRENT_SCHEMA_VERSION,
    id: crypto.randomUUID(),
    prefix: prefix.toUpperCase(),
    name,
    description: "",
    counter: new Counter(0),
    createdAt: Date.now(),
    members: [
      {
        id: ownerId,
        name: ownerName,
        email: ownerEmail ?? null,
        role: "owner",
      },
    ],
    todos: [],
  });
}

/** Update project metadata */
export function updateProject(
  doc: Doc,
  updates: Partial<Pick<Project, "name" | "description" | "prefix">>,
  actorId?: MemberId,
): Doc {
  const opts = { message: "" };
  return Automerge.change(doc, opts, (d) => {
    const actor = resolveActor(d, actorId);
    const changed: Record<string, unknown> = {};
    if (updates.name !== undefined) {
      d.name = updates.name;
      changed.name = updates.name;
    }
    if (updates.description !== undefined) {
      d.description = updates.description;
      changed.description = updates.description;
    }
    if (updates.prefix !== undefined) {
      d.prefix = updates.prefix.toUpperCase();
      changed.prefix = d.prefix;
    }
    opts.message = buildMsg(d, "project.updated", actor, d.name, changed);
  });
}

// ── Todo operations ─────────────────────────────────────────────────

/** Add a new todo and return the updated doc + the created todo's number */
export function addTodo(
  doc: Doc,
  opts: {
    title: string;
    description?: string;
    status?: Status;
    priority?: Priority;
    difficulty?: Difficulty;
    labels?: Label[];
    assignee?: MemberId | null;
    createdBy?: MemberId;
    platform?: Platform;
  },
): { doc: Doc; number: number } {
  let todoNumber = 0;

  const changeOpts = { message: "" };
  const nextDoc = Automerge.change(doc, changeOpts, (d) => {
    d.counter.increment(1);
    todoNumber = d.counter.value;
    const actor = resolveActor(d, opts.createdBy);

    d.todos.push({
      id: crypto.randomUUID(),
      number: todoNumber,
      title: opts.title,
      description: opts.description ?? "",
      status: opts.status ?? "todo",
      priority: opts.priority ?? "none",
      difficulty: opts.difficulty ?? "none",
      labels: opts.labels ?? [],
      assignee: opts.assignee ?? null,
      branch: null,
      comments: [],
      createdAt: Date.now(),
      updatedAt: Date.now(),
      createdBy: actor,
      platform: opts.platform ?? "unknown",
    });

    changeOpts.message = buildMsg(d, "todo.created", actor, `${d.prefix}-${todoNumber}`, {
      title: opts.title,
      status: opts.status ?? "todo",
      priority: opts.priority ?? "none",
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
      "title" | "description" | "status" | "priority" | "difficulty" | "labels" | "assignee"
    >
  >,
  actorId?: MemberId,
): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const todo = d.todos.find((t) => t.number === todoNumber);
    if (!todo) throw new Error(`Todo #${todoNumber} not found`);

    const actor = resolveActor(d, actorId);
    const changed: Record<string, unknown> = {};

    if (updates.title !== undefined) {
      changed.title = { from: todo.title, to: updates.title };
      todo.title = updates.title;
    }
    if (updates.description !== undefined) {
      changed.description = {
        lengthBefore: todo.description.length,
        lengthAfter: updates.description.length,
      };
      todo.description = updates.description;
    }
    if (updates.status !== undefined) {
      changed.status = { from: todo.status, to: updates.status };
      todo.status = updates.status;
    }
    if (updates.priority !== undefined) {
      changed.priority = { from: todo.priority, to: updates.priority };
      todo.priority = updates.priority;
    }
    if (updates.difficulty !== undefined) {
      changed.difficulty = { from: todo.difficulty, to: updates.difficulty };
      todo.difficulty = updates.difficulty;
    }
    if (updates.labels !== undefined) {
      changed.labels = { from: [...todo.labels], to: updates.labels };
      todo.labels.splice(0, todo.labels.length);
      for (const label of updates.labels) {
        todo.labels.push(label);
      }
    }
    if (updates.assignee !== undefined) {
      const oldAssigneeName = todo.assignee
        ? (d.members.find((m) => m.id === todo.assignee)?.name ?? null)
        : null;
      const newAssigneeName = updates.assignee
        ? (d.members.find((m) => m.id === updates.assignee)?.name ?? null)
        : null;
      changed.assignee = { from: oldAssigneeName, to: newAssigneeName };
      todo.assignee = updates.assignee;
    }
    todo.updatedAt = Date.now();

    changeOpts.message = buildMsg(d, "todo.updated", actor, `${d.prefix}-${todoNumber}`, changed);
  });
}

/** Delete a todo by number (hard delete) */
export function deleteTodo(doc: Doc, todoNumber: number, actorId?: MemberId): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const idx = d.todos.findIndex((t) => t.number === todoNumber);
    if (idx === -1) throw new Error(`Todo #${todoNumber} not found`);

    const actor = resolveActor(d, actorId);
    const todo = d.todos[idx]!;
    const assigneeName = todo.assignee
      ? (d.members.find((m) => m.id === todo.assignee)?.name ?? null)
      : null;

    // Embed the deleted item's key info in the change message,
    // since it won't exist in the document after this change.
    changeOpts.message = buildMsg(d, "todo.deleted", actor, `${d.prefix}-${todoNumber}`, {
      title: todo.title,
      status: todo.status,
      priority: todo.priority,
      difficulty: todo.difficulty,
      assignee: assigneeName,
    });

    d.todos.splice(idx, 1);
  });
}

/** Unassign a todo (clear its assignee) */
export function unassignTodo(doc: Doc, todoNumber: number, actorId?: MemberId): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const todo = d.todos.find((t) => t.number === todoNumber);
    if (!todo) throw new Error(`Todo #${todoNumber} not found`);

    const actor = resolveActor(d, actorId);
    const oldAssigneeName = todo.assignee
      ? (d.members.find((m) => m.id === todo.assignee)?.name ?? null)
      : null;

    todo.assignee = null;
    todo.updatedAt = Date.now();

    changeOpts.message = buildMsg(d, "todo.unassigned", actor, `${d.prefix}-${todoNumber}`, {
      from: oldAssigneeName,
    });
  });
}

/** Add a comment to a todo */
export function addComment(doc: Doc, todoNumber: number, text: string, actorId?: MemberId): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const todo = d.todos.find((t) => t.number === todoNumber);
    if (!todo) throw new Error(`Todo #${todoNumber} not found`);

    const actor = resolveActor(d, actorId);
    const member = d.members.find((m) => m.id === actor);

    if (!todo.comments) (todo as any).comments = [];
    todo.comments.push({
      id: crypto.randomUUID(),
      author: actor,
      authorName: member?.name ?? actor,
      text,
      createdAt: Date.now(),
    });
    todo.updatedAt = Date.now();

    changeOpts.message = buildMsg(d, "todo.commented", actor, `${d.prefix}-${todoNumber}`, {
      text: text.length > 100 ? text.slice(0, 100) + "..." : text,
    });
  });
}

/** Set the branch name on a todo (for git worktree tracking) */
export function setBranch(
  doc: Doc,
  todoNumber: number,
  branchName: string,
  actorId?: MemberId,
): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const todo = d.todos.find((t) => t.number === todoNumber);
    if (!todo) throw new Error(`Todo #${todoNumber} not found`);

    const actor = resolveActor(d, actorId);
    todo.branch = branchName;
    todo.updatedAt = Date.now();

    changeOpts.message = buildMsg(d, "todo.branched", actor, `${d.prefix}-${todoNumber}`, {
      branch: branchName,
    });
  });
}

/** Clear the branch name on a todo (after worktree removal) */
export function clearBranch(doc: Doc, todoNumber: number, actorId?: MemberId): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const todo = d.todos.find((t) => t.number === todoNumber);
    if (!todo) throw new Error(`Todo #${todoNumber} not found`);

    const actor = resolveActor(d, actorId);
    const oldBranch = todo.branch;
    todo.branch = null;
    todo.updatedAt = Date.now();

    changeOpts.message = buildMsg(d, "todo.unbranched", actor, `${d.prefix}-${todoNumber}`, {
      branch: oldBranch,
    });
  });
}

// ── Member operations ───────────────────────────────────────────────

/** Add a member to the project */
export function addMember(
  doc: Doc,
  name: string,
  role: MemberRole = "member",
  email: string | null = null,
  actorId?: MemberId,
  agentOpts?: { provider?: AgentProvider; model?: string },
): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const actor = resolveActor(d, actorId);
    const member: Record<string, unknown> = {
      id: crypto.randomUUID(),
      name,
      email,
      role,
    };
    if (role === "agent" && agentOpts?.provider) {
      member.agentProvider = agentOpts.provider;
      if (agentOpts.model) member.agentModel = agentOpts.model;
    }
    d.members.push(member as any);
    changeOpts.message = buildMsg(d, "member.added", actor, name, { role });
  });
}

/** Remove a member by ID or name. Clears assignee on any todos assigned to them. */
export function removeMember(doc: Doc, memberId: string, actorId?: MemberId): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const idx = d.members.findIndex((m) => m.id === memberId);
    if (idx === -1) throw new Error(`Member "${memberId}" not found`);

    const actor = resolveActor(d, actorId);
    const member = d.members[idx]!;

    // Count and unassign any todos assigned to this member
    let todosUnassigned = 0;
    for (const todo of d.todos) {
      if (todo.assignee === memberId) {
        todo.assignee = null;
        todosUnassigned++;
      }
    }

    changeOpts.message = buildMsg(d, "member.removed", actor, member.name, {
      role: member.role,
      todosUnassigned,
    });

    d.members.splice(idx, 1);
  });
}

/** Update a member's name, email, role, or agent config */
export function updateMember(
  doc: Doc,
  memberId: string,
  updates: Partial<Pick<Member, "name" | "email" | "role" | "agentProvider" | "agentModel">>,
  actorId?: MemberId,
): Doc {
  const changeOpts = { message: "" };
  return Automerge.change(doc, changeOpts, (d) => {
    const member = d.members.find((m) => m.id === memberId);
    if (!member) throw new Error(`Member "${memberId}" not found`);

    const actor = resolveActor(d, actorId);
    const changed: Record<string, unknown> = {};

    if (updates.name !== undefined) {
      changed.name = { from: member.name, to: updates.name };
      member.name = updates.name;
    }
    if (updates.email !== undefined) {
      changed.email = updates.email;
      member.email = updates.email;
    }
    if (updates.role !== undefined) {
      changed.role = { from: member.role, to: updates.role };
      member.role = updates.role;
    }
    if (updates.agentProvider !== undefined) {
      changed.agentProvider = updates.agentProvider;
      (member as any).agentProvider = updates.agentProvider;
    }
    if (updates.agentModel !== undefined) {
      changed.agentModel = updates.agentModel;
      (member as any).agentModel = updates.agentModel;
    }

    changeOpts.message = buildMsg(d, "member.updated", actor, member.name, changed);
  });
}
