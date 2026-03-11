import { test, expect, describe } from "bun:test";
import { createProject, addTodo, updateTodo, addMember } from "../operations.js";
import {
  queryTodos,
  findTodoByNumber,
  parseTodoRef,
  findMember,
  countByStatus,
} from "../queries.js";

function makeProject() {
  let doc = createProject("TST", "Test", "Alice");
  doc = addMember(doc, "Bob");

  const r1 = addTodo(doc, {
    title: "Auth bug",
    priority: "high",
    difficulty: "hard",
  });
  const r2 = addTodo(r1.doc, {
    title: "Write tests",
    status: "in_progress",
    priority: "medium",
    difficulty: "easy",
  });
  const r3 = addTodo(r2.doc, {
    title: "Deploy",
    status: "completed",
    priority: "low",
    difficulty: "medium",
  });
  const r4 = addTodo(r3.doc, {
    title: "Old task",
    status: "archived",
  });

  return r4.doc;
}

describe("queryTodos", () => {
  test("excludes archived by default", () => {
    const doc = makeProject();
    const todos = queryTodos(doc);
    expect(todos).toHaveLength(3);
    expect(todos.every((t) => t.status !== "archived")).toBe(true);
  });

  test("filters by status", () => {
    const doc = makeProject();
    const todos = queryTodos(doc, { status: "in_progress" });
    expect(todos).toHaveLength(1);
    expect(todos[0]!.title).toBe("Write tests");
  });

  test("filters by multiple statuses", () => {
    const doc = makeProject();
    const todos = queryTodos(doc, { status: ["todo", "in_progress"] });
    expect(todos).toHaveLength(2);
  });

  test("filters by priority", () => {
    const doc = makeProject();
    const todos = queryTodos(doc, { priority: "high" });
    expect(todos).toHaveLength(1);
    expect(todos[0]!.title).toBe("Auth bug");
  });

  test("filters by search in title", () => {
    const doc = makeProject();
    const todos = queryTodos(doc, { search: "deploy" });
    expect(todos).toHaveLength(1);
    expect(todos[0]!.title).toBe("Deploy");
  });

  test("filters by assignee name", () => {
    let doc = makeProject();
    const bob = doc.members.find((m) => m.name === "Bob")!;
    doc = updateTodo(doc, 1, { assignee: bob.id });

    const todos = queryTodos(doc, { assignee: "bob" });
    expect(todos).toHaveLength(1);
    expect(todos[0]!.title).toBe("Auth bug");
  });

  test("filters by difficulty", () => {
    const doc = makeProject();
    const todos = queryTodos(doc, { difficulty: "hard" });
    expect(todos).toHaveLength(1);
    expect(todos[0]!.title).toBe("Auth bug");
  });

  test("filters by multiple difficulties", () => {
    const doc = makeProject();
    const todos = queryTodos(doc, { difficulty: ["easy", "medium"] });
    expect(todos).toHaveLength(2);
  });

  test("returns empty for no matches", () => {
    const doc = makeProject();
    const todos = queryTodos(doc, { search: "nonexistent" });
    expect(todos).toHaveLength(0);
  });
});

describe("findTodoByNumber", () => {
  test("finds existing todo", () => {
    const doc = makeProject();
    const todo = findTodoByNumber(doc, 1);
    expect(todo).toBeDefined();
    expect(todo!.title).toBe("Auth bug");
  });

  test("returns undefined for missing todo", () => {
    const doc = makeProject();
    expect(findTodoByNumber(doc, 999)).toBeUndefined();
  });
});

describe("parseTodoRef", () => {
  test("parses PREFIX-N format", () => {
    expect(parseTodoRef("TST-1", "TST")).toBe(1);
    expect(parseTodoRef("TST-42", "TST")).toBe(42);
  });

  test("case insensitive prefix", () => {
    expect(parseTodoRef("tst-1", "TST")).toBe(1);
  });

  test("parses plain number", () => {
    expect(parseTodoRef("1", "TST")).toBe(1);
    expect(parseTodoRef("42", "TST")).toBe(42);
  });

  test("returns null for invalid input", () => {
    expect(parseTodoRef("abc", "TST")).toBeNull();
    expect(parseTodoRef("OTHER-1", "TST")).toBeNull();
  });
});

describe("findMember", () => {
  test("finds by exact name", () => {
    const doc = makeProject();
    const member = findMember(doc, "Alice");
    expect(member).toBeDefined();
    expect(member!.name).toBe("Alice");
  });

  test("case insensitive", () => {
    const doc = makeProject();
    expect(findMember(doc, "alice")).toBeDefined();
    expect(findMember(doc, "BOB")).toBeDefined();
  });

  test("partial match", () => {
    const doc = makeProject();
    expect(findMember(doc, "Ali")).toBeDefined();
  });

  test("returns undefined for nonexistent", () => {
    const doc = makeProject();
    expect(findMember(doc, "Charlie")).toBeUndefined();
  });
});

describe("countByStatus", () => {
  test("counts todos by status", () => {
    const doc = makeProject();
    const counts = countByStatus(doc);

    expect(counts.todo).toBe(1);
    expect(counts.in_progress).toBe(1);
    expect(counts.completed).toBe(1);
    expect(counts.archived).toBe(1);
    expect(counts.none).toBe(0);
    expect(counts.wont_do).toBe(0);
  });
});
