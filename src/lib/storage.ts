/**
 * Load and save Automerge documents to disk.
 */

import * as Automerge from "#automerge";
import type { Project } from "./schema.js";

type Doc = Automerge.Doc<Project>;

/** Save an Automerge document to a file, then git-add it. */
export async function saveDoc(filePath: string, doc: Doc): Promise<void> {
  const binary = Automerge.save(doc);
  await Bun.write(filePath, binary);

  // Auto-stage so the file is included in the next commit
  Bun.spawnSync(["git", "add", filePath], {
    stdout: "ignore",
    stderr: "ignore",
  });
}

/** Load an Automerge document from a file. Returns null if file doesn't exist. */
export async function loadDoc(filePath: string): Promise<Doc | null> {
  const file = Bun.file(filePath);
  if (!(await file.exists())) return null;

  const buffer = await file.arrayBuffer();
  return Automerge.load<Project>(new Uint8Array(buffer));
}
