import { test, expect, describe } from "bun:test";
import { createProject, addTodo, updateTodo } from "../operations.js";
import { toMarkdown, toJSON } from "../export.js";

function makeProject() {
  const doc = createProject("TST", "Test Project", "Alice");
  const r1 = addTodo(doc, {
    title: "In progress task",
    status: "in_progress",
    priority: "high",
    tags: ["frontend"],
  });
  const r2 = addTodo(r1.doc, {
    title: "Todo task",
    description: "Some details here",
  });
  const r3 = addTodo(r2.doc, {
    title: "Done task",
    status: "done",
  });
  return r3.doc;
}

describe("toMarkdown", () => {
  test("renders markdown with status sections", () => {
    const doc = makeProject();
    const md = toMarkdown(doc);

    expect(md).toContain("# Test Project");
    expect(md).toContain("## In Progress");
    expect(md).toContain("## Todo");
    expect(md).toContain("## Done");
    expect(md).toContain("**TST-1**");
    expect(md).toContain("In progress task");
    expect(md).toContain("[High]");
    expect(md).toContain("`frontend`");
    expect(md).toContain("- [x]"); // done item
    expect(md).toContain("Some details here");
  });

  test("omits empty status sections", () => {
    const doc = makeProject();
    const md = toMarkdown(doc);
    expect(md).not.toContain("## Backlog");
    expect(md).not.toContain("## Archived");
  });
});

describe("toJSON", () => {
  test("returns a serializable object", () => {
    const doc = makeProject();
    const json = toJSON(doc);

    // Should be JSON-safe (no Automerge proxies)
    const parsed = JSON.parse(JSON.stringify(json));
    expect(parsed.name).toBe("Test Project");
    expect(parsed.prefix).toBe("TST");
    expect(parsed.todos).toHaveLength(3);
    expect(parsed.todos[0].ref).toBe("TST-1");
    expect(parsed.todos[0].title).toBe("In progress task");
    expect(parsed.members).toHaveLength(1);
    expect(parsed.members[0].name).toBe("Alice");
  });
});
