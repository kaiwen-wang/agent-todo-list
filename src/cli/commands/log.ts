/**
 * `agt log` — View the audit log.
 *
 * Audit entries are reconstructed from Automerge's built-in change history
 * rather than stored separately on the document.
 */

import type { Command } from "commander";
import chalk from "chalk";
import { findProject } from "../../lib/project.js";
import { loadDoc } from "../../lib/storage.js";
import { getAuditLog, getAuditLogCount } from "../../lib/history.js";
import { error, formatDate } from "../output.js";

const ACTION_COLORS: Record<string, (s: string) => string> = {
  "todo.created": chalk.green,
  "todo.updated": chalk.yellow,
  "todo.deleted": chalk.red,
  "todo.unassigned": chalk.magenta,
  "todo.commented": chalk.cyan,
  "todo.branched": chalk.blue,
  "todo.unbranched": chalk.blue,
  "member.added": chalk.green,
  "member.removed": chalk.red,
  "member.updated": chalk.yellow,
  "project.updated": chalk.yellow,
};

function colorAction(action: string): string {
  const colorFn = ACTION_COLORS[action] ?? chalk.white;
  return colorFn(action);
}

export function registerLog(program: Command): void {
  program
    .command("log")
    .description("View the audit log")
    .option("-n, --limit <n>", "Number of entries to show", "20")
    .option("--json", "Output as JSON")
    .action(async (opts: { limit: string; json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const limit = parseInt(opts.limit, 10) || 20;
      const totalCount = getAuditLogCount(doc);
      const entries = getAuditLog(doc, { limit });

      if (opts.json) {
        const out = entries.map((e) => ({
          action: e.action,
          actor: e.actorName,
          target: e.target,
          details: e.details,
          timestamp: e.timestamp,
          hash: e.hash,
        }));
        console.log(JSON.stringify(out, null, 2));
        return;
      }

      if (entries.length === 0) {
        console.log("No audit log entries.");
        return;
      }

      console.log(chalk.dim(`Showing ${entries.length} of ${totalCount} entries\n`));

      for (const entry of entries) {
        const date = chalk.dim(formatDate(entry.timestamp));
        const actor = chalk.bold(entry.actorName);
        const action = colorAction(entry.action);
        const target = chalk.white(entry.target);

        let detailStr = "";
        const details = entry.details;
        const keys = Object.keys(details);
        if (keys.length > 0) {
          const parts: string[] = [];
          for (const key of keys) {
            const val = details[key];
            if (val && typeof val === "object" && "from" in val && "to" in val) {
              const fromTo = val as { from: unknown; to: unknown };
              parts.push(`${key}: ${fromTo.from} -> ${fromTo.to}`);
            } else if (val !== undefined && val !== null) {
              parts.push(`${key}: ${JSON.stringify(val)}`);
            }
          }
          if (parts.length > 0) {
            detailStr = chalk.dim(`  ${parts.join(", ")}`);
          }
        }

        console.log(`${date}  ${actor}  ${action}  ${target}${detailStr}`);
      }
    });

  // Note: `agt log clear` and `agt log trim` are no longer needed.
  // Automerge's change history is immutable — entries cannot be removed
  // without creating a new document (Automerge.clone() without history).
  // This is intentional: the audit trail is now tamper-evident.
}
