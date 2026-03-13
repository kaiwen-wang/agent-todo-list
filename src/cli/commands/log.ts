/**
 * `agt log` — View and manage the audit log.
 */

import type { Command } from "commander";
import chalk from "chalk";
import { findProject } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { clearAuditLog, trimAuditLog } from "../../lib/operations.js";
import { error, success, formatDate } from "../output.js";

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
  const logCmd = program
    .command("log")
    .description("View the audit log")
    .option("-n, --limit <n>", "Number of entries to show", "20")
    .option("--json", "Output as JSON")
    .action(async (opts: { limit: string; json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const auditLog = doc.auditLog ?? [];
      const limit = parseInt(opts.limit, 10) || 20;

      // Show most recent entries (last N)
      const entries = auditLog.slice(-limit).reverse();

      if (opts.json) {
        const out = entries.map((e) => ({
          id: e.id,
          action: e.action,
          actor: e.actorName,
          target: e.target,
          details: JSON.parse(e.details || "{}"),
          timestamp: e.timestamp,
        }));
        console.log(JSON.stringify(out, null, 2));
        return;
      }

      if (entries.length === 0) {
        console.log("No audit log entries.");
        return;
      }

      console.log(chalk.dim(`Showing ${entries.length} of ${auditLog.length} entries\n`));

      for (const entry of entries) {
        const date = chalk.dim(formatDate(entry.timestamp));
        const actor = chalk.bold(entry.actorName);
        const action = colorAction(entry.action);
        const target = chalk.white(entry.target);

        let detailStr = "";
        try {
          const details = JSON.parse(entry.details || "{}");
          const keys = Object.keys(details);
          if (keys.length > 0) {
            const parts: string[] = [];
            for (const key of keys) {
              const val = details[key];
              if (val && typeof val === "object" && "from" in val && "to" in val) {
                parts.push(`${key}: ${val.from} -> ${val.to}`);
              } else if (val !== undefined && val !== null) {
                parts.push(`${key}: ${JSON.stringify(val)}`);
              }
            }
            if (parts.length > 0) {
              detailStr = chalk.dim(`  ${parts.join(", ")}`);
            }
          }
        } catch {
          // ignore malformed details
        }

        console.log(`${date}  ${actor}  ${action}  ${target}${detailStr}`);
      }
    });

  // agt log clear
  logCmd
    .command("clear")
    .description("Clear all audit log entries")
    .action(async () => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      let doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const count = doc.auditLog?.length ?? 0;
      if (count === 0) {
        console.log("Audit log is already empty.");
        return;
      }

      doc = clearAuditLog(doc);
      await saveDoc(paths.dataPath, doc);
      success(`Cleared ${count} audit log entries.`);
    });

  // agt log trim --before <date>
  logCmd
    .command("trim")
    .description("Remove audit log entries older than a date")
    .requiredOption("--before <date>", "ISO date (e.g. 2026-01-01)")
    .action(async (opts: { before: string }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      let doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const beforeDate = new Date(opts.before);
      if (isNaN(beforeDate.getTime())) {
        error(`Invalid date: "${opts.before}". Use ISO format (e.g. 2026-01-01).`);
      }

      const countBefore = doc.auditLog?.length ?? 0;
      doc = trimAuditLog(doc, beforeDate);
      await saveDoc(paths.dataPath, doc);

      const countAfter = doc.auditLog?.length ?? 0;
      const removed = countBefore - countAfter;
      success(
        `Trimmed ${removed} entries older than ${opts.before}. ${countAfter} entries remaining.`,
      );
    });
}
