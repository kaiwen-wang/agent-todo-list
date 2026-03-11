/**
 * `agt member` — Manage project members.
 */

import type { Command } from "commander";
import type { MemberRole } from "../../lib/schema.js";
import { findProject } from "../../lib/project.js";
import { loadDoc, saveDoc } from "../../lib/storage.js";
import { addMember, removeMember, updateMember } from "../../lib/operations.js";
import { findMember } from "../../lib/queries.js";
import { error, success } from "../output.js";
import chalk from "chalk";

const VALID_ROLES: MemberRole[] = ["owner", "member", "agent"];

export function registerMember(program: Command): void {
  const member = program
    .command("member")
    .description("Manage project members");

  member
    .command("add")
    .description("Add a member to the project")
    .argument("<name>", "Member display name")
    .option("-r, --role <role>", "Role: owner, member, or agent", "member")
    .option("-e, --email <email>", "Email address")
    .option("--json", "Output as JSON")
    .action(
      async (
        name: string,
        opts: { role: string; email?: string; json?: boolean },
      ) => {
        const paths = findProject();
        if (!paths) error("Not in an agt project. Run 'agt init' first.");

        let doc = await loadDoc(paths.dataPath);
        if (!doc) error("Project data not found.");

        if (!VALID_ROLES.includes(opts.role as MemberRole)) {
          error(`Invalid role "${opts.role}". Use: ${VALID_ROLES.join(", ")}`);
        }

        // Check for duplicate name
        const existing = findMember(doc, name);
        if (existing && existing.name.toLowerCase() === name.toLowerCase()) {
          error(`A member named "${existing.name}" already exists.`);
        }

        doc = addMember(doc, name, opts.role as MemberRole, opts.email ?? null);
        await saveDoc(paths.dataPath, doc);

        const added = doc.members[doc.members.length - 1]!;

        if (opts.json) {
          console.log(
            JSON.stringify({
              id: added.id,
              name: added.name,
              role: added.role,
              email: added.email,
            }),
          );
        } else {
          success(`Added member "${name}" (${opts.role})`);
        }
      },
    );

  member
    .command("list")
    .alias("ls")
    .description("List all project members")
    .option("--json", "Output as JSON")
    .action(async (opts: { json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      const doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      if (opts.json) {
        console.log(
          JSON.stringify(
            doc.members.map((m) => ({
              id: m.id,
              name: m.name,
              role: m.role,
              email: m.email,
            })),
          ),
        );
      } else {
        if (doc.members.length === 0) {
          console.log("No members.");
          return;
        }
        for (const m of doc.members) {
          const role = chalk.dim(`(${m.role})`);
          const email = m.email ? chalk.dim(` <${m.email}>`) : "";
          console.log(`  ${m.name} ${role}${email}`);
        }
      }
    });

  member
    .command("remove")
    .alias("rm")
    .description("Remove a member from the project")
    .argument("<name>", "Member name or ID")
    .option("--json", "Output as JSON")
    .action(async (nameOrId: string, opts: { json?: boolean }) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      let doc = await loadDoc(paths.dataPath);
      if (!doc) error("Project data not found.");

      const member = findMember(doc, nameOrId);
      if (!member) error(`Member "${nameOrId}" not found.`);

      doc = removeMember(doc, member.id);
      await saveDoc(paths.dataPath, doc);

      if (opts.json) {
        console.log(JSON.stringify({ removed: member.name, id: member.id }));
      } else {
        success(`Removed member "${member.name}"`);
      }
    });

  member
    .command("update")
    .description("Update a member's role or email")
    .argument("<name>", "Member name or ID")
    .option("-r, --role <role>", "New role: owner, member, or agent")
    .option("-e, --email <email>", "New email address")
    .option("--rename <newName>", "Rename the member")
    .option("--json", "Output as JSON")
    .action(
      async (
        nameOrId: string,
        opts: { role?: string; email?: string; rename?: string; json?: boolean },
      ) => {
        const paths = findProject();
        if (!paths) error("Not in an agt project. Run 'agt init' first.");

        let doc = await loadDoc(paths.dataPath);
        if (!doc) error("Project data not found.");

        const found = findMember(doc, nameOrId);
        if (!found) error(`Member "${nameOrId}" not found.`);

        if (opts.role && !VALID_ROLES.includes(opts.role as MemberRole)) {
          error(`Invalid role "${opts.role}". Use: ${VALID_ROLES.join(", ")}`);
        }

        const updates: Partial<{ name: string; email: string | null; role: MemberRole }> = {};
        if (opts.role !== undefined) updates.role = opts.role as MemberRole;
        if (opts.email !== undefined) updates.email = opts.email;
        if (opts.rename !== undefined) updates.name = opts.rename;

        if (Object.keys(updates).length === 0) {
          error("No updates specified. Use --role, --email, or --rename.");
        }

        doc = updateMember(doc, found.id, updates);
        await saveDoc(paths.dataPath, doc);

        if (opts.json) {
          console.log(
            JSON.stringify({ updated: found.name, fields: Object.keys(updates) }),
          );
        } else {
          success(`Updated member "${found.name}"`);
        }
      },
    );
}
