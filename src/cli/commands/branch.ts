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
import { updateTodo, setBranch } from "../../lib/operations.js";
import { parseTodoRef, findTodoByNumber } from "../../lib/queries.js";
import { error, success, warn } from "../output.js";

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 60);
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

      // Store branch name on the todo and set status to in_progress
      doc = setBranch(doc, num, branchName);
      if (todo.status === "none" || todo.status === "todo") {
        doc = updateTodo(doc, num, { status: "in_progress" });
      }
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
}
