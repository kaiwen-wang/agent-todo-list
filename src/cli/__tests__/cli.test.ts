/**
 * End-to-end CLI smoke tests.
 * Runs the actual CLI binary in a temp directory and checks output.
 */

import { test, expect, describe, beforeEach, afterEach } from "bun:test";
import { join } from "node:path";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";

const CLI = join(import.meta.dir, "..", "index.ts");

async function run(
  args: string[],
  cwd: string,
): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  const proc = Bun.spawn(["bun", CLI, ...args], {
    cwd,
    stdout: "pipe",
    stderr: "pipe",
  });

  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;

  return { stdout, stderr, exitCode };
}

describe("CLI end-to-end", () => {
  let tempDir: string;

  beforeEach(() => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-cli-test-"));
    // Create a fake .git so init doesn't warn
    Bun.spawnSync(["git", "init", "-q"], { cwd: tempDir });
  });

  afterEach(() => {
    rmSync(tempDir, { recursive: true, force: true });
  });

  test("--help shows usage", async () => {
    const { stdout, exitCode } = await run(["--help"], tempDir);
    expect(exitCode).toBe(0);
    expect(stdout).toContain("agt");
    expect(stdout).toContain("init");
    expect(stdout).toContain("add");
    expect(stdout).toContain("list");
  });

  test("init creates .todo/ directory", async () => {
    const { stdout, exitCode } = await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);
    expect(exitCode).toBe(0);
    expect(stdout).toContain("Initialized project");

    // Verify files exist
    const configFile = Bun.file(join(tempDir, ".todo", "config.toml"));
    expect(await configFile.exists()).toBe(true);
    const dataFile = Bun.file(join(tempDir, ".todo", "data.automerge"));
    expect(await dataFile.exists()).toBe(true);
  });

  test("add creates a todo", async () => {
    await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);

    const { stdout, exitCode } = await run(["add", "My first task", "--priority", "high"], tempDir);
    expect(exitCode).toBe(0);
    expect(stdout).toContain("TST-1");
    expect(stdout).toContain("My first task");
  });

  test("list shows todos", async () => {
    await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);
    await run(["add", "Task one"], tempDir);
    await run(["add", "Task two", "--priority", "urgent"], tempDir);

    const { stdout, exitCode } = await run(["list"], tempDir);
    expect(exitCode).toBe(0);
    expect(stdout).toContain("TST-1");
    expect(stdout).toContain("Task one");
    expect(stdout).toContain("TST-2");
    expect(stdout).toContain("Task two");
  });

  test("list --json outputs JSON", async () => {
    await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);
    await run(["add", "JSON task"], tempDir);

    const { stdout, exitCode } = await run(["list", "--json"], tempDir);
    expect(exitCode).toBe(0);

    const parsed = JSON.parse(stdout);
    expect(parsed).toHaveLength(1);
    expect(parsed[0].title).toBe("JSON task");
    expect(parsed[0].ref).toBe("TST-1");
  });

  test("show displays todo detail", async () => {
    await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);
    await run(["add", "Detailed task", "--priority", "high"], tempDir);

    const { stdout, exitCode } = await run(["show", "TST-1"], tempDir);
    expect(exitCode).toBe(0);
    expect(stdout).toContain("Detailed task");
    expect(stdout).toContain("high");
  });

  test("show accepts plain number", async () => {
    await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);
    await run(["add", "Task"], tempDir);

    const { stdout, exitCode } = await run(["show", "1"], tempDir);
    expect(exitCode).toBe(0);
    expect(stdout).toContain("TST-1");
  });

  test("update changes status", async () => {
    await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);
    await run(["add", "Task"], tempDir);
    await run(["update", "1", "--status", "completed"], tempDir);

    const { stdout } = await run(["show", "1", "--json"], tempDir);
    const todo = JSON.parse(stdout);
    expect(todo.status).toBe("completed");
  });

  test("archive command works", async () => {
    await run(["init", "--name", "Test", "--prefix", "TST"], tempDir);
    await run(["add", "Old task"], tempDir);

    const { exitCode } = await run(["archive", "1"], tempDir);
    expect(exitCode).toBe(0);

    // Should not appear in default list
    const { stdout } = await run(["list"], tempDir);
    expect(stdout).toContain("No todos found");
  });

  test("error on missing project", async () => {
    const { stderr, exitCode } = await run(["list"], tempDir);
    expect(exitCode).toBe(1);
    expect(stderr).toContain("Not in an agt project");
  });
});
