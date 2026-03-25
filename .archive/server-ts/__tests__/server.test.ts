/**
 * Tests for the agt browser server API endpoints.
 * Spins up a real server against a temp project, exercises both endpoints.
 */

import { test, expect, beforeAll, afterAll, describe } from "bun:test";
import { mkdtempSync, mkdirSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { createProject } from "../../lib/operations.js";
import { saveDoc } from "../../lib/storage.js";
import { startServer } from "../index.js";
let tmpDir: string;
let server: ReturnType<typeof Bun.serve>;
let baseUrl: string;

beforeAll(async () => {
  // Create a temp project directory with .todo/data.automerge
  tmpDir = mkdtempSync(join(tmpdir(), "agt-server-test-"));
  mkdirSync(join(tmpDir, ".todo"), { recursive: true });

  const doc = createProject("TEST", "Test Project", "tester");
  await saveDoc(join(tmpDir, ".todo", "data.automerge"), doc);

  server = await startServer(tmpDir, 0); // port 0 = random available port
  baseUrl = `http://localhost:${server.port}`;
});

afterAll(() => {
  server?.stop(true);
});

describe("GET /api/project", () => {
  test("returns project data as JSON", async () => {
    const res = await fetch(`${baseUrl}/api/project`);
    expect(res.status).toBe(200);
    expect(res.headers.get("content-type")).toContain("application/json");

    const data = (await res.json()) as any;
    expect(data.name).toBe("Test Project");
    expect(data.prefix).toBe("TEST");
    expect(data.todos).toBeArray();
    expect(data.members).toBeArray();
    expect(data.members.length).toBe(1);
    expect(data.members[0].name).toBe("tester");
    expect(data.members[0].role).toBe("owner");
  });

  test("has CORS headers", async () => {
    const res = await fetch(`${baseUrl}/api/project`);
    expect(res.headers.get("access-control-allow-origin")).toBe("*");
  });
});

describe("POST /api/change", () => {
  test("action=add creates a new todo", async () => {
    const res = await fetch(`${baseUrl}/api/change`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        action: "add",
        title: "Buy milk",
        priority: "high",
      }),
    });

    expect(res.status).toBe(200);
    const data = (await res.json()) as any;
    expect(data.ok).toBe(true);
    expect(data.number).toBeGreaterThan(0);

    // Verify it shows up in the project
    const projectRes = await fetch(`${baseUrl}/api/project`);
    const project = (await projectRes.json()) as any;
    const todo = project.todos.find((t: any) => t.title === "Buy milk");
    expect(todo).toBeDefined();
    expect(todo.priority).toBe("high");
  });

  test("action=update modifies an existing todo", async () => {
    // First add a todo
    const addRes = await fetch(`${baseUrl}/api/change`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ action: "add", title: "Update me" }),
    });
    const { number } = (await addRes.json()) as any;

    // Now update it
    const updateRes = await fetch(`${baseUrl}/api/change`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        action: "update",
        number,
        updates: { title: "Updated!", status: "in_progress", priority: "urgent" },
      }),
    });

    expect(updateRes.status).toBe(200);
    const data = (await updateRes.json()) as any;
    expect(data.ok).toBe(true);

    // Verify the update stuck
    const projectRes = await fetch(`${baseUrl}/api/project`);
    const project = (await projectRes.json()) as any;
    const todo = project.todos.find((t: any) => t.number === number);
    expect(todo.title).toBe("Updated!");
    expect(todo.status).toBe("in_progress");
    expect(todo.priority).toBe("urgent");
  });

  test("action=delete removes a todo", async () => {
    // Add a todo
    const addRes = await fetch(`${baseUrl}/api/change`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ action: "add", title: "Delete me" }),
    });
    const { number } = (await addRes.json()) as any;

    // Delete it
    const deleteRes = await fetch(`${baseUrl}/api/change`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ action: "delete", number }),
    });

    expect(deleteRes.status).toBe(200);

    // Verify it's gone
    const projectRes = await fetch(`${baseUrl}/api/project`);
    const project = (await projectRes.json()) as any;
    const todo = project.todos.find((t: any) => t.number === number);
    expect(todo).toBeUndefined();
  });

  test("unknown action returns 400", async () => {
    const res = await fetch(`${baseUrl}/api/change`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ action: "explode" }),
    });

    expect(res.status).toBe(400);
    const data = (await res.json()) as any;
    expect(data.error).toContain("Unknown action");
  });

  test("updating nonexistent todo returns 400", async () => {
    const res = await fetch(`${baseUrl}/api/change`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        action: "update",
        number: 99999,
        updates: { title: "nope" },
      }),
    });

    expect(res.status).toBe(400);
    const data = (await res.json()) as any;
    expect(data.error).toContain("not found");
  });
});

describe("CORS preflight", () => {
  test("OPTIONS returns CORS headers", async () => {
    const res = await fetch(`${baseUrl}/api/project`, { method: "OPTIONS" });
    expect(res.status).toBe(200);
    expect(res.headers.get("access-control-allow-origin")).toBe("*");
    expect(res.headers.get("access-control-allow-methods")).toContain("POST");
  });
});

describe("Static file serving", () => {
  test("unknown paths return SPA fallback or 404", async () => {
    const res = await fetch(`${baseUrl}/some/random/path`);
    // Since dist exists (we built it), should get the SPA fallback (index.html)
    // or 404 if dist doesn't exist at the expected path
    expect([200, 404]).toContain(res.status);
  });
});
