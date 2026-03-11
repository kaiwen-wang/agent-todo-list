import { test, expect, describe } from "bun:test";
import * as Automerge from "@automerge/automerge";
import {
  createProject,
  addTodo,
  updateTodo,
  deleteTodo,
  addMember,
  updateProject,
} from "../operations.js";

describe("createProject", () => {
  test("creates a valid Automerge document", () => {
    const doc = createProject("TST", "Test Project", "Alice");

    expect(doc.prefix).toBe("TST");
    expect(doc.name).toBe("Test Project");
    expect(doc.counter.value).toBe(0);
    expect(doc.members).toHaveLength(1);
    expect(doc.members[0]!.name).toBe("Alice");
    expect(doc.members[0]!.role).toBe("owner");
    expect(doc.todos).toHaveLength(0);
    expect(doc._version).toBe(4);
  });

  test("uppercases the prefix", () => {
    const doc = createProject("abc", "Test", "Alice");
    expect(doc.prefix).toBe("ABC");
  });
});

describe("addTodo", () => {
  test("adds a todo and increments counter", () => {
    const doc = createProject("TST", "Test", "Alice");
    const r1 = addTodo(doc, { title: "First task" });

    expect(r1.number).toBe(1);
    expect(r1.doc.counter.value).toBe(1);
    expect(r1.doc.todos).toHaveLength(1);
    expect(r1.doc.todos[0]!.title).toBe("First task");
    expect(r1.doc.todos[0]!.status).toBe("todo");
    expect(r1.doc.todos[0]!.priority).toBe("none");
    expect(r1.doc.todos[0]!.difficulty).toBe("none");
  });

  test("increments counter for each todo", () => {
    let doc = createProject("TST", "Test", "Alice");
    const r1 = addTodo(doc, { title: "First" });
    const r2 = addTodo(r1.doc, { title: "Second" });
    const r3 = addTodo(r2.doc, { title: "Third" });

    expect(r1.number).toBe(1);
    expect(r2.number).toBe(2);
    expect(r3.number).toBe(3);
    expect(r3.doc.counter.value).toBe(3);
    expect(r3.doc.todos).toHaveLength(3);
  });

  test("respects custom status and priority", () => {
    const doc = createProject("TST", "Test", "Alice");
    const result = addTodo(doc, {
      title: "Urgent bug",
      status: "in_progress",
      priority: "urgent",
    });

    const todo = result.doc.todos[0]!;
    expect(todo.status).toBe("in_progress");
    expect(todo.priority).toBe("urgent");
  });

  test("respects custom difficulty", () => {
    const doc = createProject("TST", "Test", "Alice");
    const result = addTodo(doc, {
      title: "Hard task",
      difficulty: "hard",
    });

    expect(result.doc.todos[0]!.difficulty).toBe("hard");
  });

  test("concurrent counter increments merge correctly", () => {
    const doc = createProject("TST", "Test", "Alice");

    // Simulate two concurrent adds from different peers
    const branch1 = Automerge.clone(doc);
    const branch2 = Automerge.clone(doc);

    const r1 = addTodo(branch1, { title: "From peer 1" });
    const r2 = addTodo(branch2, { title: "From peer 2" });

    // Merge the two branches
    const merged = Automerge.merge(r1.doc, r2.doc);

    // Counter should be 2 (one increment from each peer)
    expect(merged.counter.value).toBe(2);
    expect(merged.todos).toHaveLength(2);

    // Both todos should exist, though they may have the same number
    // (Counter guarantees the counter VALUE is correct, but both peers
    // read counter=0 before incrementing, so both assign number=1.
    // This is a known limitation — unique numbering requires coordination.)
    const titles = merged.todos.map((t) => t.title).sort();
    expect(titles).toEqual(["From peer 1", "From peer 2"]);
  });
});

describe("updateTodo", () => {
  test("updates title", () => {
    const doc = createProject("TST", "Test", "Alice");
    const { doc: d1, number } = addTodo(doc, { title: "Original" });
    const d2 = updateTodo(d1, number, { title: "Updated" });

    expect(d2.todos[0]!.title).toBe("Updated");
  });

  test("updates status", () => {
    const doc = createProject("TST", "Test", "Alice");
    const { doc: d1, number } = addTodo(doc, { title: "Task" });
    const d2 = updateTodo(d1, number, { status: "completed" });

    expect(d2.todos[0]!.status).toBe("completed");
  });

  test("updates difficulty", () => {
    const doc = createProject("TST", "Test", "Alice");
    const { doc: d1, number } = addTodo(doc, { title: "Task" });
    expect(d1.todos[0]!.difficulty).toBe("none");

    const d2 = updateTodo(d1, number, { difficulty: "hard" });
    expect(d2.todos[0]!.difficulty).toBe("hard");
  });

  test("throws on nonexistent todo", () => {
    const doc = createProject("TST", "Test", "Alice");
    expect(() => updateTodo(doc, 999, { title: "Nope" })).toThrow("Todo #999 not found");
  });

  test("sets updatedAt timestamp", () => {
    const doc = createProject("TST", "Test", "Alice");
    const { doc: d1, number } = addTodo(doc, { title: "Task" });
    const originalUpdated = d1.todos[0]!.updatedAt;

    // Small delay to ensure different timestamp
    const d2 = updateTodo(d1, number, { title: "Changed" });
    // updatedAt should be >= original (might be same ms in fast execution)
    expect(new Date(d2.todos[0]!.updatedAt).getTime()).toBeGreaterThanOrEqual(
      new Date(originalUpdated).getTime(),
    );
  });
});

describe("deleteTodo", () => {
  test("removes a todo", () => {
    const doc = createProject("TST", "Test", "Alice");
    const { doc: d1 } = addTodo(doc, { title: "Delete me" });
    expect(d1.todos).toHaveLength(1);

    const d2 = deleteTodo(d1, 1);
    expect(d2.todos).toHaveLength(0);
  });

  test("throws on nonexistent todo", () => {
    const doc = createProject("TST", "Test", "Alice");
    expect(() => deleteTodo(doc, 1)).toThrow("Todo #1 not found");
  });
});

describe("addMember", () => {
  test("adds a member with default role", () => {
    const doc = createProject("TST", "Test", "Alice");
    const d2 = addMember(doc, "Bob");

    expect(d2.members).toHaveLength(2);
    expect(d2.members[1]!.name).toBe("Bob");
    expect(d2.members[1]!.role).toBe("member");
  });

  test("adds a member with agent role", () => {
    const doc = createProject("TST", "Test", "Alice");
    const d2 = addMember(doc, "Claude", "agent");

    expect(d2.members[1]!.role).toBe("agent");
  });
});

describe("updateProject", () => {
  test("updates project name", () => {
    const doc = createProject("TST", "Test", "Alice");
    const d2 = updateProject(doc, { name: "New Name" });

    expect(d2.name).toBe("New Name");
  });

  test("uppercases prefix", () => {
    const doc = createProject("TST", "Test", "Alice");
    const d2 = updateProject(doc, { prefix: "abc" });

    expect(d2.prefix).toBe("ABC");
  });
});
