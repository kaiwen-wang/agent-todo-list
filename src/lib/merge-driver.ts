#!/usr/bin/env bun
/**
 * Git merge driver for Automerge .automerge files.
 *
 * Registered via .gitattributes + git config. When git encounters a merge
 * conflict on a .automerge file, it calls this script instead of its
 * default binary conflict handler.
 *
 * Usage (called by git automatically):
 *   bun merge-driver.ts <base> <ours> <theirs>
 *
 * - Loads all three versions as Automerge documents
 * - Merges ours + theirs (Automerge merge is conflict-free)
 * - Writes the result to <ours> (git convention)
 * - Exits 0 on success (tells git the merge succeeded)
 */

import * as Automerge from "@automerge/automerge";
import type { Project } from "./schema.js";

const [, , basePath, oursPath, theirsPath] = process.argv;

if (!basePath || !oursPath || !theirsPath) {
  console.error("Usage: merge-driver.ts <base> <ours> <theirs>");
  process.exit(1);
}

async function loadFile(path: string): Promise<Automerge.Doc<Project>> {
  const buffer = await Bun.file(path).arrayBuffer();
  return Automerge.load<Project>(new Uint8Array(buffer));
}

try {
  const ours = await loadFile(oursPath);
  const theirs = await loadFile(theirsPath);

  // Automerge.merge is conflict-free — concurrent changes combine cleanly
  const merged = Automerge.merge(ours, theirs);

  // Write merged result back to "ours" path (git convention)
  const binary = Automerge.save(merged);
  await Bun.write(oursPath, binary);

  process.exit(0);
} catch (err) {
  console.error("Automerge merge driver failed:", err);
  process.exit(1);
}
