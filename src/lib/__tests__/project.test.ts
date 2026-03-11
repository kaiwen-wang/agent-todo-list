import { test, expect, describe, afterEach } from "bun:test";
import { join } from "node:path";
import { mkdtempSync, rmSync, mkdirSync } from "node:fs";
import { tmpdir } from "node:os";
import {
  findProject,
  isGitRepo,
  initProject,
  readConfig,
} from "../project.js";

describe("findProject", () => {
  let tempDir: string;

  afterEach(() => {
    if (tempDir) {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("finds .todo/ in current directory", () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));
    mkdirSync(join(tempDir, ".todo"));

    const result = findProject(tempDir);
    expect(result).not.toBeNull();
    expect(result!.root).toBe(tempDir);
    expect(result!.todoDir).toBe(join(tempDir, ".todo"));
  });

  test("walks up to find .todo/", () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));
    mkdirSync(join(tempDir, ".todo"));
    const subdir = join(tempDir, "src", "lib");
    mkdirSync(subdir, { recursive: true });

    const result = findProject(subdir);
    expect(result).not.toBeNull();
    expect(result!.root).toBe(tempDir);
  });

  test("returns null when no .todo/ found", () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));
    const result = findProject(tempDir);
    expect(result).toBeNull();
  });
});

describe("isGitRepo", () => {
  let tempDir: string;

  afterEach(() => {
    if (tempDir) {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("returns true for git repos", () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));
    mkdirSync(join(tempDir, ".git"));
    expect(isGitRepo(tempDir)).toBe(true);
  });

  test("returns false for non-git dirs", () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));
    expect(isGitRepo(tempDir)).toBe(false);
  });
});

describe("initProject + readConfig", () => {
  let tempDir: string;

  afterEach(() => {
    if (tempDir) {
      rmSync(tempDir, { recursive: true, force: true });
    }
  });

  test("creates .todo/ with config", async () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));

    const paths = await initProject(tempDir, {
      id: "test-id",
      prefix: "TST",
      name: "Test Project",
    });

    expect(paths.todoDir).toBe(join(tempDir, ".todo"));

    const config = await readConfig(paths.configPath);
    expect(config.id).toBe("test-id");
    expect(config.prefix).toBe("TST");
    expect(config.name).toBe("Test Project");
  });

  test("creates .gitattributes with merge driver", async () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));

    await initProject(tempDir, {
      id: "test-id",
      prefix: "TST",
      name: "Test",
    });

    const gitattrs = await Bun.file(join(tempDir, ".gitattributes")).text();
    expect(gitattrs).toContain(".todo/data.automerge merge=automerge-crdt");
  });

  test("appends to existing .gitattributes", async () => {
    tempDir = mkdtempSync(join(tmpdir(), "agt-test-"));
    await Bun.write(join(tempDir, ".gitattributes"), "*.txt text\n");

    await initProject(tempDir, {
      id: "test-id",
      prefix: "TST",
      name: "Test",
    });

    const gitattrs = await Bun.file(join(tempDir, ".gitattributes")).text();
    expect(gitattrs).toContain("*.txt text");
    expect(gitattrs).toContain("merge=automerge-crdt");
  });
});
