/**
 * `agt inbox` — View and manage the freeform inbox (TODO.md).
 */

import type { Command } from "commander";
import { findProject } from "../../lib/project.js";
import { readInbox, writeInbox, readProcessed, ensureInboxFiles } from "../../lib/inbox.js";
import { error, success } from "../output.js";

export function registerInbox(program: Command): void {
  const inbox = program
    .command("inbox")
    .description("View and manage the inbox (TODO.md)")
    .action(async () => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      await ensureInboxFiles(paths.todoDir);
      const text = await readInbox(paths.todoDir);

      if (!text.trim()) {
        console.log("(inbox is empty)");
      } else {
        console.log(text);
      }
    });

  inbox
    .command("add")
    .description("Append a note to the inbox")
    .argument("<text...>", "Text to add (joined with spaces)")
    .action(async (textParts: string[]) => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      await ensureInboxFiles(paths.todoDir);
      const existing = await readInbox(paths.todoDir);
      const line = textParts.join(" ");
      const newContent = existing.trimEnd() ? existing.trimEnd() + "\n" + line + "\n" : line + "\n";

      await writeInbox(paths.todoDir, newContent);
      success(`Added to inbox: ${line}`);
    });

  inbox
    .command("clear")
    .description("Clear the inbox")
    .action(async () => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      await writeInbox(paths.todoDir, "");
      success("Inbox cleared.");
    });

  inbox
    .command("processed")
    .description("View processed inbox items")
    .action(async () => {
      const paths = findProject();
      if (!paths) error("Not in an agt project. Run 'agt init' first.");

      await ensureInboxFiles(paths.todoDir);
      const text = await readProcessed(paths.todoDir);

      if (!text.trim()) {
        console.log("(no processed items yet)");
      } else {
        console.log(text);
      }
    });
}
