//! Audit log reconstruction from Automerge's built-in change history.

use automerge::AutoCommit;

use crate::operations::ChangeMessage;
use crate::schema::AuditEntry;

/// Reconstruct the audit log from Automerge's change history.
/// Returns entries newest first.
pub fn get_audit_log(
    doc: &mut AutoCommit,
    limit: Option<usize>,
    _since_heads: Option<&[automerge::ChangeHash]>,
) -> Vec<AuditEntry> {
    let changes = doc.get_changes(&[]);
    let mut entries = Vec::new();

    for change in changes {
        let message = match change.message() {
            Some(msg) if !msg.is_empty() => msg,
            _ => continue,
        };

        let parsed: ChangeMessage = match serde_json::from_str(message) {
            Ok(p) => p,
            Err(_) => continue,
        };

        if parsed.action.is_empty() || parsed.target.is_empty() {
            continue;
        }

        entries.push(AuditEntry {
            action: parsed.action,
            actor_id: parsed.actor_id,
            actor_name: parsed.actor_name,
            target: parsed.target,
            details: parsed
                .details
                .unwrap_or(serde_json::Value::Object(Default::default())),
            timestamp: change.timestamp() * 1000,
            hash: change.hash().to_string(),
        });
    }

    entries.reverse();

    if let Some(limit) = limit {
        entries.truncate(limit);
    }

    entries
}

/// Get the total number of audit entries in the history.
pub fn get_audit_log_count(doc: &mut AutoCommit) -> usize {
    let changes = doc.get_changes(&[]);
    let mut count = 0;
    for change in changes {
        if let Some(msg) = change.message()
            && let Ok(parsed) = serde_json::from_str::<ChangeMessage>(msg)
            && !parsed.action.is_empty()
            && !parsed.target.is_empty()
        {
            count += 1;
        }
    }
    count
}
