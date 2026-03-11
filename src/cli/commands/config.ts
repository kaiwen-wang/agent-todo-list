/**
 * `agt config` — View or update project settings.
 *
 * With no flags:  prints current settings.
 * With flags:     updates the specified fields.
 */

import type { Command } from "commander";
import { findProject, readConfig, syncConfig } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { updateProject } from "../../lib/operations.js";
import { error, success } from "../output.js";

export function registerConfig(program: Command): void {
  program
    .command("config")
    .description("View or update project settings")
    .option("-n, --name <name>", "Set project name")
    .option("-p, --prefix <prefix>", "Set issue prefix (e.g. ABC)")
    .option("-d, --description <text>", "Set project description")
    .option("--json", "Output as JSON")
    .action(
      async (opts: {
        name?: string;
        prefix?: string;
        description?: string;
        json?: boolean;
      }) => {
        const paths = findProject();
        if (!paths) error("Not in an agt project. Run 'agt init' first.");

        let doc = await loadDoc(paths.dataPath);
        if (!doc) error("Project data not found.");

        const hasUpdates =
          opts.name !== undefined ||
          opts.prefix !== undefined ||
          opts.description !== undefined;

        if (hasUpdates) {
          // Build updates from flags
          const updates: Record<string, string> = {};
          if (opts.name !== undefined) updates.name = opts.name;
          if (opts.prefix !== undefined) updates.prefix = opts.prefix;
          if (opts.description !== undefined) updates.description = opts.description;

          doc = updateProject(doc, updates);
          await saveDoc(paths.dataPath, doc);

          // Keep config.toml in sync
          await syncConfig(paths.configPath, {
            prefix: doc.prefix,
            name: doc.name,
          });

          if (opts.json) {
            console.log(
              JSON.stringify({ ok: true, updated: Object.keys(updates) }),
            );
          } else {
            success(`Updated project settings`);
          }
        } else {
          // Display current settings
          if (opts.json) {
            console.log(
              JSON.stringify({
                name: doc.name,
                prefix: doc.prefix,
                description: doc.description,
                id: doc.id,
              }),
            );
          } else {
            console.log(`  Name:        ${doc.name}`);
            console.log(`  Prefix:      ${doc.prefix}`);
            console.log(`  Description: ${doc.description || "(none)"}`);
            console.log(`  ID:          ${doc.id}`);
          }
        }
      },
    );
}
