/**
 * .todo/ directory management — finding, creating, and reading project config.
 */

import { join, dirname, basename } from "node:path";
import { existsSync, mkdirSync, statSync } from "node:fs";
import type { ProjectConfig } from "./schema.js";



const TODO_DIR = ".todo";
const CONFIG_FILE = "config.toml";
const DATA_FILE = "data.automerge";

export interface TodoPaths {
  /** Root of the git repo (or wherever .todo/ lives) */
  root: string;
  /** Path to .todo/ directory */
  todoDir: string;
  /** Path to .todo/config.toml */
  configPath: string;
  /** Path to .todo/data.automerge */
  dataPath: string;
  /** Path to .todo/TODO.md (inbox) */
  inboxPath: string;
  /** Path to .todo/TODO-PROCESSED.md (processed inbox archive) */
  processedPath: string;
}

/**
 * Walk up from `startDir` looking for a .todo/ directory.
 * Returns paths if found, null otherwise.
 */
export function findProject(startDir: string = process.cwd()): TodoPaths | null {
  let dir = startDir;

  // eslint-disable-next-line no-constant-condition
  while (true) {
    const candidate = join(dir, TODO_DIR);
    if (existsSync(candidate) && statSync(candidate).isDirectory()) {
      return {
        root: dir,
        todoDir: candidate,
        configPath: join(candidate, CONFIG_FILE),
        dataPath: join(candidate, DATA_FILE),
        inboxPath: join(candidate, "TODO.md"),
        processedPath: join(candidate, "TODO-PROCESSED.md"),
      };
    }

    const parent = dirname(dir);
    if (parent === dir) break; // reached filesystem root
    dir = parent;
  }

  return null;
}

/**
 * Check if the given directory is a git repo.
 */
export function isGitRepo(dir: string = process.cwd()): boolean {
  return existsSync(join(dir, ".git"));
}

/**
 * Create the .todo/ directory and config file for a new project.
 *
 * The data.automerge file IS committed to git (not gitignored).
 * A custom git merge driver handles binary merges using Automerge.merge().
 */
export async function initProject(dir: string, config: ProjectConfig): Promise<TodoPaths> {
  const todoDir = join(dir, TODO_DIR);
  mkdirSync(todoDir, { recursive: true });

  const paths: TodoPaths = {
    root: dir,
    todoDir,
    configPath: join(todoDir, CONFIG_FILE),
    dataPath: join(todoDir, DATA_FILE),
    inboxPath: join(todoDir, "TODO.md"),
    processedPath: join(todoDir, "TODO-PROCESSED.md"),
  };

  // Write config.toml
  const toml = [
    `id = "${config.id}"`,
    `prefix = "${config.prefix}"`,
    `name = "${config.name}"`,
  ].join("\n");

  await Bun.write(paths.configPath, toml + "\n");

  // Set up .gitattributes for the Automerge merge driver
  const gitattrsPath = join(dir, ".gitattributes");
  const gitattrsFile = Bun.file(gitattrsPath);
  const mergeDriverLine = ".todo/data.automerge merge=automerge-crdt";

  if (await gitattrsFile.exists()) {
    const content = await gitattrsFile.text();
    if (!content.includes("merge=automerge-crdt")) {
      await Bun.write(
        gitattrsPath,
        content.trimEnd() +
          "\n\n# CRDT merge driver for Automerge binary data\n" +
          mergeDriverLine +
          "\n",
      );
    }
  } else {
    await Bun.write(
      gitattrsPath,
      "# CRDT merge driver for Automerge binary data\n" + mergeDriverLine + "\n",
    );
  }

  // Create inbox files
  await Bun.write(paths.inboxPath, "");
  await Bun.write(paths.processedPath, "");

  // Configure the git merge driver (local repo config only)
  if (isGitRepo(dir)) {
    const mergeDriverScript = join(import.meta.dir, "merge-driver.ts");
    Bun.spawnSync(["git", "config", "merge.automerge-crdt.name", "Automerge CRDT merge driver"], {
      cwd: dir,
    });
    Bun.spawnSync(
      ["git", "config", "merge.automerge-crdt.driver", `bun ${mergeDriverScript} %O %A %B`],
      { cwd: dir },
    );
  }

  return paths;
}

/**
 * Write config.toml to keep it in sync with the Automerge doc.
 * Called after any project-level metadata change (name, prefix).
 * Preserves the existing config ID — only updates name and prefix.
 */
export async function syncConfig(
  configPath: string,
  fields: { prefix: string; name: string },
): Promise<void> {
  const existing = await readConfig(configPath);
  const toml = [
    `id = "${existing.id}"`,
    `prefix = "${fields.prefix}"`,
    `name = "${fields.name}"`,
  ].join("\n");
  await Bun.write(configPath, toml + "\n");
}

/**
 * Read the config.toml file and parse it (simple TOML — just key = "value" lines).
 */
export async function readConfig(configPath: string): Promise<ProjectConfig> {
  const text = await Bun.file(configPath).text();
  const config: Record<string, string> = {};

  for (const line of text.split("\n")) {
    const match = line.match(/^(\w+)\s*=\s*"([^"]*)"/);
    if (match && match[1] && match[2]) {
      config[match[1]] = match[2];
    }
  }

  if (!config.id || !config.prefix || !config.name) {
    throw new Error(`Invalid config at ${configPath}: missing required fields`);
  }

  return {
    id: config.id,
    prefix: config.prefix,
    name: config.name,
  };
}

/**
 * Auto-detect a human-friendly project name from the environment.
 * Priority: package.json "name" → git remote repo name → directory name.
 */
export function detectProjectName(dir: string): string {
  // 1. package.json name
  const pkgPath = join(dir, "package.json");
  if (existsSync(pkgPath)) {
    try {
      const pkg = JSON.parse(require("node:fs").readFileSync(pkgPath, "utf-8"));
      if (typeof pkg.name === "string" && pkg.name.trim()) {
        return prettifyName(pkg.name.trim());
      }
    } catch {
      // malformed package.json — fall through
    }
  }

  // 2. git remote (origin) repo name
  const remoteResult = Bun.spawnSync(["git", "remote", "get-url", "origin"], {
    cwd: dir,
    stdout: "pipe",
    stderr: "pipe",
  });
  if (remoteResult.exitCode === 0) {
    const url = remoteResult.stdout.toString().trim();
    // Handle both HTTPS and SSH URLs:
    //   https://github.com/user/repo-name.git  →  repo-name
    //   git@github.com:user/repo-name.git      →  repo-name
    const repoName = basename(url).replace(/\.git$/, "");
    if (repoName) return prettifyName(repoName);
  }

  // 3. Directory name
  return prettifyName(basename(dir));
}

/**
 * Turn a slug like "agent-todo-list" into "Agent Todo List".
 */
function prettifyName(slug: string): string {
  return slug
    .replace(/^@[^/]+\//, "") // strip npm scope
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

/**
 * Derive a short prefix from a project name.
 * Takes the first letter of each word, up to 4 characters.
 * Falls back to first 3 chars if only one word.
 *
 *   "Agent Todo List"    →  "ATL"
 *   "my-app"             →  "MA"
 *   "server"             →  "SER"
 *   "The Great Project"  →  "TGP"
 */
export function derivePrefix(name: string): string {
  const words = name
    .replace(/^@[^/]+\//, "")
    .split(/[-_\s]+/)
    .filter(Boolean);

  if (words.length === 1) {
    return words[0]!.slice(0, 3).toUpperCase();
  }

  return words
    .slice(0, 4)
    .map((w) => w.charAt(0))
    .join("")
    .toUpperCase();
}
