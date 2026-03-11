import { test, expect, describe, afterEach } from "bun:test";
import { join } from "node:path";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { createProject, addTodo } from "../operations.js";
import { saveDoc, loadDoc } from "../storage.js";

describe("storage", () => {
  let tempDir: string;

  afterEach(() => {
    if (tempDir) {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("save and load round-trip", async () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));
    const filePath = join(tempDir, "data.automerge");

    const doc = createProject("TST", "Test", "Alice");
    const { doc: withTodo } = addTodo(doc, {
      title: "Saved task",
      priority: "high",
      tags: ["test"],
    });

    await saveDoc(filePath, withTodo);
    const loaded = await loadDoc(filePath);

    expect(loaded).not.toBeNull();
    expect(loaded!.name).toBe("Test");
    expect(loaded!.prefix).toBe("TST");
    expect(loaded!.todos).toHaveLength(1);
    expect(loaded!.todos[0]!.title).toBe("Saved task");
    expect(loaded!.todos[0]!.priority).toBe("high");
    expect([...loaded!.todos[0]!.tags]).toEqual(["test"]);
    expect(loaded!.counter.value).toBe(1);
  });

  test("loadDoc returns null for missing file", async () => {
    const result = await loadDoc("/nonexistent/path/data.automerge");
    expect(result).toBeNull();
  });
});
