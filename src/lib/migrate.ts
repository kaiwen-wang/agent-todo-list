/**
 * Schema migration logic.
 *
 * Automerge is schema-less — migrations transform old shapes to new ones
 * inside an Automerge.change() call. Each migration function handles one
 * version bump.
 */

import * as Automerge from "#automerge";
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
  4: (d) => {
    // v4 → v5:
    //   - Remove auditLog array from the document.
    //     Audit history is now derived from Automerge's built-in change history
    //     via getAuditLog() in src/lib/history.ts.
    //     We delete the field to stop it from growing the CRDT binary.
    if ((d as any).auditLog) {
      delete (d as any).auditLog;
    }
  },

  3: (d) => {
    // v3 → v4:
    //   - Add difficulty field to each todo (default "none" for pre-existing items)
    for (const todo of d.todos) {
      if (!(todo as any).difficulty) {
        (todo as any).difficulty = "none";
      }
    }
  },

  2: (d) => {
    // v2 → v3:
    //   - Add platform field to each todo (default "unknown" for pre-existing items)
    for (const todo of d.todos) {
      if (!(todo as any).platform) {
        (todo as any).platform = "unknown";
      }
    }
  },

  1: (d) => {
    // v1 → v2:
    //   - Convert all timestamps from ISO strings to Unix ms numbers
    //   - Add auditLog[] to project
    //   - Add comments[] and branch to each todo

    // Project timestamp
    const projectTs = d.createdAt as unknown;
    if (typeof projectTs === "string") {
      (d as any).createdAt = new Date(projectTs as string).getTime();
    }

    // Todo timestamps + new fields
    for (const todo of d.todos) {
      const created = todo.createdAt as unknown;
      if (typeof created === "string") {
        (todo as any).createdAt = new Date(created as string).getTime();
      }
      const updated = todo.updatedAt as unknown;
      if (typeof updated === "string") {
        (todo as any).updatedAt = new Date(updated as string).getTime();
      }
      if (!(todo as any).comments) {
        (todo as any).comments = [];
      }
      if ((todo as any).branch === undefined) {
        (todo as any).branch = null;
      }
    }

    // Audit log
    if (!(d as any).auditLog) {
      (d as any).auditLog = [];
    }
  },
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
