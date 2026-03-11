/**
 * Read/filter functions for querying the Project document.
 * All functions are pure — they read from an Automerge Doc without mutating.
 */

import type * as Automerge from "@automerge/automerge";
import type { Project, Todo, Status, Priority } from "./schema.js";

type Doc = Automerge.Doc<Project>;

export interface TodoFilter {
  status?: Status | Status[];
  priority?: Priority | Priority[];
  assignee?: string;
  search?: string;
}

/** Get all todos, optionally filtered. Excludes archived by default. */
export function queryTodos(doc: Doc, filter: TodoFilter = {}): Todo[] {
  let todos = [...doc.todos];

  // Default: exclude archived unless explicitly filtering for it
  const statusFilter = filter.status;
  if (statusFilter) {
    const statuses = Array.isArray(statusFilter)
      ? statusFilter
      : [statusFilter];
    todos = todos.filter((t) => statuses.includes(t.status));
  } else {
    todos = todos.filter((t) => t.status !== "archived");
  }

  if (filter.priority) {
    const priorities = Array.isArray(filter.priority)
      ? filter.priority
      : [filter.priority];
    todos = todos.filter((t) => priorities.includes(t.priority));
  }

  if (filter.assignee) {
    const assigneeLower = filter.assignee.toLowerCase();
    todos = todos.filter((t) => {
      if (!t.assignee) return false;
      // Match by member ID or by member name
      const member = doc.members.find((m) => m.id === t.assignee);
      return (
        t.assignee.toLowerCase() === assigneeLower ||
        (member && member.name.toLowerCase().includes(assigneeLower))
      );
    });
  }

  if (filter.search) {
    const searchLower = filter.search.toLowerCase();
    todos = todos.filter(
      (t) =>
        t.title.toLowerCase().includes(searchLower) ||
        t.description.toLowerCase().includes(searchLower),
    );
  }

  return todos;
}

/** Find a single todo by its number. */
export function findTodoByNumber(doc: Doc, num: number): Todo | undefined {
  return doc.todos.find((t) => t.number === num);
}

/**
 * Parse a todo reference like "ABC-1" into the number part.
 * Returns the number, or null if the format is invalid.
 */
export function parseTodoRef(ref: string, prefix: string): number | null {
  // Accept both "ABC-1" and plain "1"
  const upper = ref.toUpperCase();
  const expectedPrefix = prefix.toUpperCase() + "-";

  if (upper.startsWith(expectedPrefix)) {
    const num = parseInt(upper.slice(expectedPrefix.length), 10);
    return Number.isNaN(num) ? null : num;
  }

  const num = parseInt(ref, 10);
  return Number.isNaN(num) ? null : num;
}

/** Find a member by name (case-insensitive partial match) or ID. */
export function findMember(
  doc: Doc,
  nameOrId: string,
): (typeof doc.members)[number] | undefined {
  const lower = nameOrId.toLowerCase();
  return (
    doc.members.find((m) => m.id === nameOrId) ||
    doc.members.find((m) => m.name.toLowerCase() === lower) ||
    doc.members.find((m) => m.name.toLowerCase().includes(lower))
  );
}

/** Get a count of todos grouped by status. */
export function countByStatus(doc: Doc): Record<Status, number> {
  const counts: Record<Status, number> = {
    none: 0,
    todo: 0,
    in_progress: 0,
    completed: 0,
    archived: 0,
    wont_do: 0,
  };
  for (const todo of doc.todos) {
    counts[todo.status]++;
  }
  return counts;
}
