/**
 * Schema migration logic.
 *
 * Automerge is schema-less — migrations transform old shapes to new ones
 * inside an Automerge.change() call. Each migration function handles one
 * version bump.
 */

import * as Automerge from "@automerge/automerge";
import type { Project } from "./schema.js";
import { CURRENT_SCHEMA_VERSION } from "./schema.js";

type Doc = Automerge.Doc<Project>;

/**
 * Each migration takes a mutable Automerge doc proxy and upgrades it
 * from version N to N+1.
 */
type MigrationFn = (d: Project) => void;

/**
 * Registry of migrations. Key is the version we're migrating FROM.
 * e.g. migrations[1] upgrades from v1 → v2.
 */
const migrations: Record<number, MigrationFn> = {
  // Example for future use:
  // 1: (d) => {
  //   // Rename a field, add a new field, etc.
  //   d._version = 2;
  // },
};

/**
 * Apply all necessary migrations to bring a document up to the current version.
 * Returns the (potentially updated) document.
 *
 * If the document is already at the current version, returns it unchanged.
 */
export function migrateDoc(doc: Doc): Doc {
  let version = doc._version ?? 0;

  if (version >= CURRENT_SCHEMA_VERSION) {
    return doc;
  }

  let current = doc;
  while (version < CURRENT_SCHEMA_VERSION) {
    const migrateFn = migrations[version];
    if (!migrateFn) {
      // No explicit migration needed — just bump the version
      current = Automerge.change(current, (d) => {
        d._version = version + 1;
      });
    } else {
      current = Automerge.change(current, (d) => {
        migrateFn(d);
        d._version = version + 1;
      });
    }
    version++;
  }

  return current;
}

/** Check whether a document needs migration. */
export function needsMigration(doc: Doc): boolean {
  return (doc._version ?? 0) < CURRENT_SCHEMA_VERSION;
}
