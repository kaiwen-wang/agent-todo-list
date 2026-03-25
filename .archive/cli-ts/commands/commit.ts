/**
 * `agt commit` — Commit and optionally push the .todo/ data.
 *
 * Auto-generates a commit message from the audit log entries that
 * are new since the last git commit.
 */

import * as Automerge from "#automerge";
import type { Command } from "commander";
import type { Project } from "../../lib/schema.js";
import { findProject } from "../../lib/project.js";
import { loadDoc } from "../../lib/storage.js";
import { getAuditLog } from "../../lib/history.js";
import { error, success } from "../output.js";

/**
 * Load the last-committed version of data.automerge from git HEAD.
 * Returns null if the file isn't committed yet.
 */
function loadCommittedDoc(): Automerge.Doc<Project> | null {
  const result = Bun.spawnSync(["git", "show", "HEAD:.todo/data.automerge"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  if (result.exitCode !== 0) return null;

  const buffer = result.stdout;
  if (buffer.byteLength === 0) return null;

  try {
    return Automerge.load<Project>(new Uint8Array(buffer));
  } catch {
    return null;
  }
}

/**
 * Build a commit message from new audit log entries.
 *
 * Compares the current doc's audit log against the last-committed version
 * to find only the new entries, then summarizes them.
 */
function buildCommitMessage(currentDoc: Automerge.Doc<Project>): string {
  const committedDoc = loadCommittedDoc();

  // Get all entries from current doc
  const allEntries = getAuditLog(currentDoc);

  if (!committedDoc) {
    // First commit — everything is new
    return summarizeEntries(allEntries);
  }

  // Get the count of entries already committed
  const committedEntries = getAuditLog(committedDoc);
  const committedCount = committedEntries.length;

  // New entries = total - committed (entries are newest-first)
  const newEntries = allEntries.slice(0, allEntries.length - committedCount);

  if (newEntries.length === 0) {
    return "todo: update";
  }

  return summarizeEntries(newEntries);
}

/**
 * Summarize audit entries into a short commit message.
 *
 * Examples:
 *   "todo: created ATL-5, updated ATL-3"
 *   "todo: created ATL-5 and 3 more changes"
 */
function summarizeEntries(entries: ReturnType<typeof getAuditLog>): string {
  if (entries.length === 0) return "todo: update";

  // Map action names to short verbs
  const verbMap: Record<string, string> = {
    "todo.created": "created",
    "todo.updated": "updated",
    "todo.deleted": "deleted",
    "todo.unassigned": "unassigned",
    "todo.commented": "commented on",
    "todo.branched": "branched",
    "todo.unbranched": "unbranched",
    "member.added": "added member",
    "member.removed": "removed member",
    "member.updated": "updated member",
    "project.updated": "updated project",
  };

  const parts: string[] = [];
  for (const entry of entries) {
    const verb = verbMap[entry.action] ?? entry.action;
    parts.push(`${verb} ${entry.target}`);
  }

  // Keep it short — show up to 3, then "and N more"
  const MAX_SHOWN = 3;
  if (parts.length <= MAX_SHOWN) {
    return `todo: ${parts.join(", ")}`;
  }

  const shown = parts.slice(0, MAX_SHOWN).join(", ");
  const remaining = parts.length - MAX_SHOWN;
  return `todo: ${shown} and ${remaining} more`;
}

export function registerCommit(program: Command): void {
  program
    .command("commit")
    .description("Commit .todo/ changes and optionally push")
    .option("--push", "Push to remote after committing")
    .option("-m, --message <msg>", "Custom commit message")
    .action(async (opts: { push?: boolean; message?: string }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      // Stage all .todo/ files (data.automerge is auto-staged on save,
      // but config.toml / inbox files may also have changed)
      const addResult = Bun.spawnSync(["git", "add", paths.todoDir], {
        cwd: paths.root,
        stdout: "pipe",
        stderr: "pipe",
      });
      if (addResult.exitCode !== 0) {
        error(`git add failed: ${addResult.stderr.toString()}`);
      }

      // Check if there's anything staged in .todo/
      const diffResult = Bun.spawnSync(
        ["git", "diff", "--cached", "--quiet", "--", paths.todoDir],
        { cwd: paths.root, stdout: "pipe", stderr: "pipe" },
      );
      if (diffResult.exitCode === 0) {
        success("Nothing to commit — .todo/ is already up to date.");
        return;
      }

      // Build message from audit log if no custom message provided
      let msg = opts.message;
      if (!msg) {
        const doc = await loadDoc(paths.dataPath);
        msg = doc ? buildCommitMessage(doc) : "todo: update";
      }

      const commitResult = Bun.spawnSync(["git", "commit", "-m", msg, "--", paths.todoDir], {
        cwd: paths.root,
        stdout: "pipe",
        stderr: "pipe",
      });
      if (commitResult.exitCode !== 0) {
        error(`git commit failed: ${commitResult.stderr.toString()}`);
      }

      success(`Committed: ${msg}`);

      if (opts.push) {
        const pushResult = Bun.spawnSync(["git", "push"], {
          cwd: paths.root,
          stdout: "pipe",
          stderr: "pipe",
        });
        if (pushResult.exitCode !== 0) {
          error(`git push failed: ${pushResult.stderr.toString()}`);
        }
        success("Pushed to remote.");
      }
    });
}
