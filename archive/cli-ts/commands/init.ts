/**
 * `agt init` — Create a .todo/ directory in the current repo.
 */

import type { Command } from "commander";
import {
  findProject,
  isGitRepo,
  initProject,
  detectProjectName,
  derivePrefix,
} from "../../lib/project.js";
import { createProject } from "../../lib/operations.js";
import { saveDoc } from "../../lib/storage.js";
import { getGitIdentity } from "../../lib/git-identity.js";
import { error, warn, success } from "../output.js";

export function registerInit(program: Command): void {
  program
    .command("init")
    .description("Initialize a new project in the current directory")
    .option(
      "-n, --name <name>",
      "Project name (auto-detected from package.json / git remote / dirname)",
    )
    .option("-p, --prefix <prefix>", "Issue prefix (auto-derived from name)")
    .option("-o, --owner <name>", "Owner name (auto-detected from git config)")
    .action(async (opts: { name?: string; prefix?: string; owner?: string }) => {
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

      // Auto-detect name → derive prefix
      const name = opts.name ?? detectProjectName(cwd);
      const prefix = (opts.prefix ?? derivePrefix(name)).toUpperCase();

      // Create project
      const id = crypto.randomUUID();
      const paths = await initProject(cwd, { id, prefix, name });

      // Resolve owner name and email from git config if not explicitly set
      const git = getGitIdentity();
      const ownerName = opts.owner ?? git.name ?? "default";
      const ownerEmail = git.email ?? null;

      // Create Automerge document
      const doc = createProject(prefix, name, ownerName, ownerEmail);
      await saveDoc(paths.dataPath, doc);

      success(`Initialized project "${name}" (${prefix}) at ${paths.todoDir}`);
      console.log(`  Config: ${paths.configPath}`);
      console.log(`  Data:   ${paths.dataPath}`);
      console.log(`  Inbox:  ${paths.inboxPath}`);
      if (isGitRepo(cwd)) {
        console.log(`  Git merge driver configured for conflict-free sync.`);
      }
    });
}
