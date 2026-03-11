/**
 * Inbox file utilities — read/write .todo/TODO.md and .todo/TODO-PROCESSED.md.
 *
 * The inbox is a plain markdown file, not part of the Automerge CRDT.
 * Users jot freeform notes into TODO.md, and the Brain processes them into tasks.
 * Processed items are moved to TODO-PROCESSED.md with timestamps and task refs.
 */

import { join } from "node:path";

const INBOX_FILE = "TODO.md";
const PROCESSED_FILE = "TODO-PROCESSED.md";

export interface ProcessedEntry {
  /** Original text from the inbox */
  original: string;
  /** The created task reference (e.g. "ISD-5") */
  ref: string;
  /** The created task title */
  title: string;
}

/** Resolve the inbox file path from a .todo/ directory. */
export function inboxPath(todoDir: string): string {
  return join(todoDir, INBOX_FILE);
}

/** Resolve the processed file path from a .todo/ directory. */
export function processedPath(todoDir: string): string {
  return join(todoDir, PROCESSED_FILE);
}

/** Read the inbox (TODO.md). Returns empty string if file doesn't exist. */
export async function readInbox(todoDir: string): Promise<string> {
  const file = Bun.file(inboxPath(todoDir));
  if (!(await file.exists())) return "";
  return file.text();
}

/** Overwrite the inbox (TODO.md) with new content. Creates the file if needed. */
export async function writeInbox(todoDir: string, text: string): Promise<void> {
  await Bun.write(inboxPath(todoDir), text);
}

/** Read the processed archive (TODO-PROCESSED.md). Returns empty string if file doesn't exist. */
export async function readProcessed(todoDir: string): Promise<string> {
  const file = Bun.file(processedPath(todoDir));
  if (!(await file.exists())) return "";
  return file.text();
}

/** Append processed entries to TODO-PROCESSED.md with a timestamp header. */
export async function appendProcessed(
  todoDir: string,
  entries: ProcessedEntry[],
): Promise<void> {
  if (entries.length === 0) return;

  const existing = await readProcessed(todoDir);
  const timestamp = new Date().toISOString().slice(0, 19).replace("T", " ");

  const lines = entries.map(
    (e) => `- [${e.ref}] ${e.original} -> "${e.title}"`,
  );

  const block = `\n## ${timestamp}\n\n${lines.join("\n")}\n`;
  const content = existing ? existing.trimEnd() + "\n" + block : block.trimStart();

  await Bun.write(processedPath(todoDir), content);
}

/** Create empty inbox files if they don't exist yet. */
export async function ensureInboxFiles(todoDir: string): Promise<void> {
  const inbox = Bun.file(inboxPath(todoDir));
  if (!(await inbox.exists())) {
    await Bun.write(inboxPath(todoDir), "");
  }

  const processed = Bun.file(processedPath(todoDir));
  if (!(await processed.exists())) {
    await Bun.write(processedPath(todoDir), "");
  }
}
