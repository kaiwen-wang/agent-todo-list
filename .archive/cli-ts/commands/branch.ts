/**
 * `agt branch` — Create a git worktree + branch for a todo.
 *
 * Creates .worktrees/PREFIX-N-title-slug/ with a new branch.
 * Sets the todo status to in_progress and stores the branch name.
 */

import { join } from "node:path";
import { existsSync } from "node:fs";
import type { Command } from "commander";
import { findProject, readConfig } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { setBranch, clearBranch } from "../../lib/operations.js";
import { parseTodoRef, findTodoByNumber } from "../../lib/queries.js";
import { error, success, warn } from "../output.js";

const ARTICLES = new Set(["a", "an", "the"]);

function slugify(text: string, maxWords = 5): string {
  return text
    .toLowerCase()
    .split(/[^a-z0-9]+/)
    .filter((w) => w && !ARTICLES.has(w))
    .slice(0, maxWords)
    .join("-");
}

export function registerBranch(program: Command): void {
  program
    .command("branch")
    .description("Create a git worktree + branch for a todo")
    .argument("<ref>", "Todo reference (e.g. ABC-1 or 1)")
    .option("--no-worktree", "Create branch only, skip worktree")
    .option("--json", "Output as JSON")
    .action(async (ref: string, opts: { worktree: boolean; json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const config = await readConfig(paths.configPath);
      let doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const num = parseTodoRef(ref, config.prefix);
      if (num === null) error(`Invalid todo reference: "${ref}".`);

      const todo = findTodoByNumber(doc, num);
      if (!todo) error(`Todo ${config.prefix}-${num} not found.`);

      // Check if already branched
      if (todo.branch) {
        if (opts.json) {
          console.log(
            JSON.stringify({
              ref: `${config.prefix}-${num}`,
              branch: todo.branch,
              alreadyExists: true,
            }),
          );
        } else {
          warn(`Todo ${config.prefix}-${num} already has branch: ${todo.branch}`);
        }
        return;
      }

      const slug = slugify(todo.title);
      const branchName = `${config.prefix.toLowerCase()}-${num}-${slug}`;

      if (opts.worktree) {
        const worktreesDir = join(paths.root, ".worktrees");
        const worktreePath = join(worktreesDir, branchName);

        if (existsSync(worktreePath)) {
          error(`Worktree path already exists: ${worktreePath}`);
        }

        // Ensure .worktrees/ is in .gitignore
        const gitignorePath = join(paths.root, ".gitignore");
        const gitignoreFile = Bun.file(gitignorePath);
        if (await gitignoreFile.exists()) {
          const content = await gitignoreFile.text();
          if (!content.includes(".worktrees")) {
            await Bun.write(
              gitignorePath,
              content.trimEnd() + "\n\n# Git worktrees for todo branches\n.worktrees/\n",
            );
          }
        } else {
          await Bun.write(gitignorePath, "# Git worktrees for todo branches\n.worktrees/\n");
        }

        const result = Bun.spawnSync(["git", "worktree", "add", "-b", branchName, worktreePath], {
          cwd: paths.root,
          stderr: "pipe",
          stdout: "pipe",
        });

        if (result.exitCode !== 0) {
          const stderr = result.stderr.toString().trim();
          error(`Failed to create worktree: ${stderr}`);
        }
      } else {
        const result = Bun.spawnSync(["git", "branch", branchName], {
          cwd: paths.root,
          stderr: "pipe",
          stdout: "pipe",
        });

        if (result.exitCode !== 0) {
          const stderr = result.stderr.toString().trim();
          error(`Failed to create branch: ${stderr}`);
        }
      }

      // Store branch name on the todo
      doc = setBranch(doc, num, branchName);
      await saveDoc(paths.dataPath, doc);

      const todoRef = `${config.prefix}-${num}`;
      if (opts.json) {
        console.log(
          JSON.stringify({
            ref: todoRef,
            branch: branchName,
            worktree: opts.worktree ? `.worktrees/${branchName}` : null,
          }),
        );
      } else {
        success(`Created branch: ${branchName}`);
        if (opts.worktree) {
          console.log(`  Worktree: .worktrees/${branchName}/`);
        }
      }
    });

  program
    .command("unbranch")
    .description("Remove a git worktree + branch for a todo")
    .argument("<ref>", "Todo reference (e.g. ABC-1 or 1)")
    .option("--keep-branch", "Remove worktree but keep the git branch")
    .option("--json", "Output as JSON")
    .action(async (ref: string, opts: { keepBranch?: boolean; json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const config = await readConfig(paths.configPath);
      let doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const num = parseTodoRef(ref, config.prefix);
      if (num === null) error(`Invalid todo reference: "${ref}".`);

      const todo = findTodoByNumber(doc, num);
      if (!todo) error(`Todo ${config.prefix}-${num} not found.`);

      if (!todo.branch) {
        if (opts.json) {
          console.log(JSON.stringify({ ref: `${config.prefix}-${num}`, error: "no branch" }));
        } else {
          warn(`Todo ${config.prefix}-${num} has no branch.`);
        }
        return;
      }

      const branchName = todo.branch;
      const worktreePath = join(paths.root, ".worktrees", branchName);

      // Remove worktree if it exists
      if (existsSync(worktreePath)) {
        const result = Bun.spawnSync(["git", "worktree", "remove", worktreePath], {
          cwd: paths.root,
          stderr: "pipe",
          stdout: "pipe",
        });

        if (result.exitCode !== 0) {
          const stderr = result.stderr.toString().trim();
          error(`Failed to remove worktree: ${stderr}`);
        }
      }

      // Delete the git branch unless --keep-branch
      if (!opts.keepBranch) {
        const result = Bun.spawnSync(["git", "branch", "-d", branchName], {
          cwd: paths.root,
          stderr: "pipe",
          stdout: "pipe",
        });

        if (result.exitCode !== 0) {
          const stderr = result.stderr.toString().trim();
          // Not fatal — branch may have been deleted already or not yet merged
          warn(`Could not delete branch: ${stderr}`);
        }
      }

      // Clear branch reference on the todo
      doc = clearBranch(doc, num);
      await saveDoc(paths.dataPath, doc);

      const todoRef = `${config.prefix}-${num}`;
      if (opts.json) {
        console.log(JSON.stringify({ ref: todoRef, branch: branchName, removed: true }));
      } else {
        success(`Removed branch: ${branchName}`);
      }
    });
}
