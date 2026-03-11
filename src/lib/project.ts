/**
 * .todo/ directory management — finding, creating, and reading project config.
 */

import { join, dirname } from "node:path";
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
export async function initProject(
  dir: string,
  config: ProjectConfig,
): Promise<TodoPaths> {
  const todoDir = join(dir, TODO_DIR);
  mkdirSync(todoDir, { recursive: true });

  const paths: TodoPaths = {
    root: dir,
    todoDir,
    configPath: join(todoDir, CONFIG_FILE),
    dataPath: join(todoDir, DATA_FILE),
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
        content.trimEnd() + "\n\n# CRDT merge driver for Automerge binary data\n" + mergeDriverLine + "\n",
      );
    }
  } else {
    await Bun.write(
      gitattrsPath,
      "# CRDT merge driver for Automerge binary data\n" + mergeDriverLine + "\n",
    );
  }

  // Configure the git merge driver (local repo config only)
  if (isGitRepo(dir)) {
    const mergeDriverScript = join(
      import.meta.dir,
      "merge-driver.ts",
    );
    Bun.spawnSync(
      ["git", "config", "merge.automerge-crdt.name", "Automerge CRDT merge driver"],
      { cwd: dir },
    );
    Bun.spawnSync(
      ["git", "config", "merge.automerge-crdt.driver", `bun ${mergeDriverScript} %O %A %B`],
      { cwd: dir },
    );
  }

  return paths;
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
