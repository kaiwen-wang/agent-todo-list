/**
 * `agt init` — Create a .todo/ directory in the current repo.
 */

import type { Command } from "commander";
import { findProject, isGitRepo, initProject } from "../../lib/project.js";
import { createProject } from "../../lib/operations.js";
import { saveDoc } from "../../lib/storage.js";
import { error, warn, success } from "../output.js";

export function registerInit(program: Command): void {
  program
    .command("init")
    .description("Initialize a new project in the current directory")
    .option("-n, --name <name>", "Project name", "My Project")
    .option("-p, --prefix <prefix>", "Issue prefix (e.g. ABC)", "TODO")
    .option("-o, --owner <name>", "Owner name", "default")
    .action(async (opts: { name: string; prefix: string; owner: string }) => {
      const cwd = process.cwd();

      // Check if already initialized
      const existing = findProject(cwd);
      if (existing) {
        error(`Project already initialized at ${existing.todoDir}`);
      }

      // Warn if not a git repo
      if (!isGitRepo(cwd)) {
        warn(
          "No .git/ directory found. It's recommended to run 'agt init' inside a git repository.",
        );
      }

      // Create project
      const id = crypto.randomUUID();
      const paths = await initProject(cwd, {
        id,
        prefix: opts.prefix.toUpperCase(),
        name: opts.name,
      });

      // Create Automerge document
      const doc = createProject(opts.prefix, opts.name, opts.owner);
      await saveDoc(paths.dataPath, doc);

      success(`Initialized project "${opts.name}" (${opts.prefix.toUpperCase()}) at ${paths.todoDir}`);
      console.log(`  Config: ${paths.configPath}`);
      console.log(`  Data:   ${paths.dataPath}`);
    });
}
