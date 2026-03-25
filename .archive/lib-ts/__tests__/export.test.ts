import { test, expect, describe } from "bun:test";
import { createProject, addTodo } from "../operations.js";
import { toJSON } from "../export.js";

function makeProject() {
  const doc = createProject("TST", "Test Project", "Alice");
  const r1 = addTodo(doc, {
    title: "In progress task",
    status: "in_progress",
    priority: "high",
  });
  const r2 = addTodo(r1.doc, {
    title: "Completed task",
    status: "completed",
  });
  return r2.doc;
}

describe("toJSON", () => {
  test("returns a serializable object", () => {
    const doc = makeProject();
    const json = toJSON(doc);

    // Should be JSON-safe (no Automerge proxies)
    const parsed = JSON.parse(JSON.stringify(json));
    expect(parsed.name).toBe("Test Project");
    expect(parsed.prefix).toBe("TST");
    expect(parsed.todos).toHaveLength(2);
    expect(parsed.todos[0].ref).toBe("TST-1");
    expect(parsed.todos[0].title).toBe("In progress task");
    expect(parsed.members).toHaveLength(1);
    expect(parsed.members[0].name).toBe("Alice");
  });
});
