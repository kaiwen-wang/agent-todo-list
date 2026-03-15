import { test, expect, describe } from "bun:test";
import {
  createProject,
  addTodo,
  updateTodo,
  deleteTodo,
  addMember,
  removeMember,
  updateMember,
  addComment,
  setBranch,
  clearBranch,
  unassignTodo,
  updateProject,
} from "../operations.js";
import { getAuditLog, getAuditLogCount } from "../history.js";

function makeProject() {
  return createProject("TST", "Test Project", "Alice");
}

describe("getAuditLog", () => {
  test("returns empty for a fresh project (no change messages)", () => {
    const doc = makeProject();
    const log = getAuditLog(doc);
    // createProject uses Automerge.from() which doesn't set a change message
    expect(log).toHaveLength(0);
  });

  test("captures todo.created", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, {
      title: "Fix bug",
      status: "in_progress",
      priority: "high",
    });

    const log = getAuditLog(d1);
    expect(log).toHaveLength(1);
    expect(log[0]!.action).toBe("todo.created");
    expect(log[0]!.target).toBe("TST-1");
    expect(log[0]!.actorName).toBe("Alice");
    expect(log[0]!.details.title).toBe("Fix bug");
    expect(log[0]!.details.status).toBe("in_progress");
    expect(log[0]!.details.priority).toBe("high");
    expect(log[0]!.timestamp).toBeGreaterThan(0);
    expect(log[0]!.hash).toBeTruthy();
  });

  test("captures todo.updated with field diffs", () => {
    const doc = makeProject();
    const { doc: d1, number } = addTodo(doc, { title: "Task" });
    const d2 = updateTodo(d1, number, {
      status: "in_progress",
      priority: "high",
    });

    const log = getAuditLog(d2);
    expect(log).toHaveLength(2);
    // newest first
    const entry = log[0]!;
    expect(entry.action).toBe("todo.updated");
    expect(entry.target).toBe("TST-1");
    const details = entry.details as Record<string, any>;
    expect(details.status).toEqual({ from: "todo", to: "in_progress" });
    expect(details.priority).toEqual({ from: "none", to: "high" });
  });

  test("captures todo.deleted with snapshot of deleted item", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, {
      title: "Delete me",
      priority: "urgent",
    });
    const d2 = deleteTodo(d1, 1);

    const log = getAuditLog(d2);
    expect(log).toHaveLength(2);
    const entry = log[0]!;
    expect(entry.action).toBe("todo.deleted");
    expect(entry.target).toBe("TST-1");
    expect(entry.details.title).toBe("Delete me");
    expect(entry.details.priority).toBe("urgent");
  });

  test("captures todo.commented", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, { title: "Task" });
    const d2 = addComment(d1, 1, "Working on it");

    const log = getAuditLog(d2);
    const entry = log[0]!;
    expect(entry.action).toBe("todo.commented");
    expect(entry.target).toBe("TST-1");
    expect(entry.details.text).toBe("Working on it");
  });

  test("truncates long comment text in audit", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, { title: "Task" });
    const longText = "a".repeat(200);
    const d2 = addComment(d1, 1, longText);

    const log = getAuditLog(d2);
    const entry = log[0]!;
    expect((entry.details.text as string).length).toBeLessThanOrEqual(103); // 100 + "..."
  });

  test("captures todo.unassigned", () => {
    const doc = makeProject();
    const memberId = doc.members[0]!.id;
    const { doc: d1 } = addTodo(doc, { title: "Task", assignee: memberId });
    const d2 = unassignTodo(d1, 1);

    const log = getAuditLog(d2);
    const entry = log[0]!;
    expect(entry.action).toBe("todo.unassigned");
    expect(entry.details.from).toBe("Alice");
  });

  test("captures todo.branched and todo.unbranched", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, { title: "Task" });
    const d2 = setBranch(d1, 1, "feat/task-1");
    const d3 = clearBranch(d2, 1);

    const log = getAuditLog(d3);
    expect(log[0]!.action).toBe("todo.unbranched");
    expect(log[0]!.details.branch).toBe("feat/task-1");
    expect(log[1]!.action).toBe("todo.branched");
    expect(log[1]!.details.branch).toBe("feat/task-1");
  });

  test("captures member.added", () => {
    const doc = makeProject();
    const d2 = addMember(doc, "Bob", "agent");

    const log = getAuditLog(d2);
    expect(log).toHaveLength(1);
    expect(log[0]!.action).toBe("member.added");
    expect(log[0]!.target).toBe("Bob");
    expect(log[0]!.details.role).toBe("agent");
  });

  test("captures member.removed", () => {
    const doc = makeProject();
    const d2 = addMember(doc, "Bob");
    const bobId = d2.members[1]!.id;
    const d3 = removeMember(d2, bobId);

    const log = getAuditLog(d3);
    expect(log[0]!.action).toBe("member.removed");
    expect(log[0]!.target).toBe("Bob");
  });

  test("captures member.updated", () => {
    const doc = makeProject();
    const d2 = addMember(doc, "Bob");
    const bobId = d2.members[1]!.id;
    const d3 = updateMember(d2, bobId, { name: "Robert" });

    const log = getAuditLog(d3);
    expect(log[0]!.action).toBe("member.updated");
    const details = log[0]!.details as Record<string, any>;
    expect(details.name).toEqual({ from: "Bob", to: "Robert" });
  });

  test("captures project.updated", () => {
    const doc = makeProject();
    const d2 = updateProject(doc, { name: "New Name" });

    const log = getAuditLog(d2);
    expect(log[0]!.action).toBe("project.updated");
    expect(log[0]!.target).toBe("New Name");
    expect(log[0]!.details.name).toBe("New Name");
  });

  test("returns entries newest-first", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, { title: "First" });
    const { doc: d2 } = addTodo(d1, { title: "Second" });
    const { doc: d3 } = addTodo(d2, { title: "Third" });

    const log = getAuditLog(d3);
    expect(log).toHaveLength(3);
    expect(log[0]!.details.title).toBe("Third");
    expect(log[1]!.details.title).toBe("Second");
    expect(log[2]!.details.title).toBe("First");
  });

  test("respects limit option", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, { title: "First" });
    const { doc: d2 } = addTodo(d1, { title: "Second" });
    const { doc: d3 } = addTodo(d2, { title: "Third" });

    const log = getAuditLog(d3, { limit: 2 });
    expect(log).toHaveLength(2);
    // Should be the 2 newest
    expect(log[0]!.details.title).toBe("Third");
    expect(log[1]!.details.title).toBe("Second");
  });

  test("each entry has a unique hash", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, { title: "First" });
    const { doc: d2 } = addTodo(d1, { title: "Second" });

    const log = getAuditLog(d2);
    const hashes = log.map((e) => e.hash);
    expect(new Set(hashes).size).toBe(hashes.length);
  });
});

describe("getAuditLogCount", () => {
  test("returns 0 for a fresh project", () => {
    const doc = makeProject();
    expect(getAuditLogCount(doc)).toBe(0);
  });

  test("counts only changes with valid audit messages", () => {
    const doc = makeProject();
    const { doc: d1 } = addTodo(doc, { title: "First" });
    const { doc: d2 } = addTodo(d1, { title: "Second" });
    const d3 = updateTodo(d2, 1, { status: "completed" });

    expect(getAuditLogCount(d3)).toBe(3);
  });
});
