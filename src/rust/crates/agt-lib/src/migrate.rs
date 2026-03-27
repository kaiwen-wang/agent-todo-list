//! Schema migration logic.

use anyhow::Result;
use automerge::{
    AutoCommit, ObjType, ROOT, ReadDoc, ScalarValue, transaction::CommitOptions,
    transaction::Transactable,
};

use crate::schema::CURRENT_SCHEMA_VERSION;

/// Apply all necessary migrations to bring a document up to the current version.
pub fn migrate_doc(doc: &mut AutoCommit) -> Result<()> {
    let mut version = get_version(doc);

    if version >= CURRENT_SCHEMA_VERSION {
        return Ok(());
    }

    while version < CURRENT_SCHEMA_VERSION {
        match version {
            1 => migrate_v1_to_v2(doc)?,
            2 => migrate_v2_to_v3(doc)?,
            3 => migrate_v3_to_v4(doc)?,
            4 => migrate_v4_to_v5(doc)?,
            5 => migrate_v5_to_v6(doc)?,
            6 => migrate_v6_to_v7(doc)?,
            _ => {
                doc.put(ROOT, "_version", (version + 1) as i64)?;
                doc.commit_with(CommitOptions::default().with_message(format!(
                    "schema migration v{} -> v{}",
                    version,
                    version + 1
                )));
            }
        }
        version += 1;
    }

    Ok(())
}

/// Check whether a document needs migration.
pub fn needs_migration(doc: &AutoCommit) -> bool {
    get_version(doc) < CURRENT_SCHEMA_VERSION
}

fn get_version(doc: &AutoCommit) -> u64 {
    match doc.get(ROOT, "_version") {
        Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
            ScalarValue::Int(n) => *n as u64,
            ScalarValue::Uint(n) => *n,
            ScalarValue::F64(n) => *n as u64,
            _ => 0,
        },
        _ => 0,
    }
}

fn migrate_v1_to_v2(doc: &mut AutoCommit) -> Result<()> {
    if let Ok(Some((_, todos_id))) = doc.get(ROOT, "todos") {
        let len = doc.length(&todos_id);
        for i in 0..len {
            if let Ok(Some((_, t_id))) = doc.get(&todos_id, i) {
                if doc.get(&t_id, "comments").ok().flatten().is_none() {
                    doc.put_object(&t_id, "comments", ObjType::List)?;
                }
                if doc.get(&t_id, "branch").ok().flatten().is_none() {
                    doc.put(&t_id, "branch", ScalarValue::Null)?;
                }
            }
        }
    }
    doc.put(ROOT, "_version", 2i64)?;
    doc.commit_with(CommitOptions::default().with_message("schema migration v1 -> v2"));
    Ok(())
}

fn migrate_v2_to_v3(doc: &mut AutoCommit) -> Result<()> {
    if let Ok(Some((_, todos_id))) = doc.get(ROOT, "todos") {
        let len = doc.length(&todos_id);
        for i in 0..len {
            if let Ok(Some((_, t_id))) = doc.get(&todos_id, i)
                && doc.get(&t_id, "platform").ok().flatten().is_none()
            {
                doc.put(&t_id, "platform", "unknown")?;
            }
        }
    }
    doc.put(ROOT, "_version", 3i64)?;
    doc.commit_with(CommitOptions::default().with_message("schema migration v2 -> v3"));
    Ok(())
}

fn migrate_v3_to_v4(doc: &mut AutoCommit) -> Result<()> {
    if let Ok(Some((_, todos_id))) = doc.get(ROOT, "todos") {
        let len = doc.length(&todos_id);
        for i in 0..len {
            if let Ok(Some((_, t_id))) = doc.get(&todos_id, i)
                && doc.get(&t_id, "difficulty").ok().flatten().is_none()
            {
                doc.put(&t_id, "difficulty", "none")?;
            }
        }
    }
    doc.put(ROOT, "_version", 4i64)?;
    doc.commit_with(CommitOptions::default().with_message("schema migration v3 -> v4"));
    Ok(())
}

fn migrate_v4_to_v5(doc: &mut AutoCommit) -> Result<()> {
    if doc.get(ROOT, "auditLog").ok().flatten().is_some() {
        doc.delete(ROOT, "auditLog")?;
    }
    doc.put(ROOT, "_version", 5i64)?;
    doc.commit_with(CommitOptions::default().with_message("schema migration v4 -> v5"));
    Ok(())
}

fn migrate_v5_to_v6(doc: &mut AutoCommit) -> Result<()> {
    // No-op data migration — just adds the "queued" status variant.
    // Existing docs don't have queued todos, so nothing to transform.
    doc.put(ROOT, "_version", 6i64)?;
    doc.commit_with(CommitOptions::default().with_message("schema migration v5 -> v6"));
    Ok(())
}

fn migrate_v6_to_v7(doc: &mut AutoCommit) -> Result<()> {
    // Add worktrees and commits lists to all existing todos.
    if let Ok(Some((_, todos_id))) = doc.get(ROOT, "todos") {
        let len = doc.length(&todos_id);
        for i in 0..len {
            if let Ok(Some((_, t_id))) = doc.get(&todos_id, i) {
                if doc.get(&t_id, "worktrees").ok().flatten().is_none() {
                    doc.put_object(&t_id, "worktrees", ObjType::List)?;
                }
                if doc.get(&t_id, "commits").ok().flatten().is_none() {
                    doc.put_object(&t_id, "commits", ObjType::List)?;
                }
            }
        }
    }
    doc.put(ROOT, "_version", 7i64)?;
    doc.commit_with(CommitOptions::default().with_message("schema migration v6 -> v7"));
    Ok(())
}
