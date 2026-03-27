//! Serialization helpers for the Project document.
//! Converts Automerge doc to plain JSON-safe objects, adding computed fields.

use automerge::AutoCommit;
use serde_json::{Value, json};

use crate::queries::{read_all_members, read_all_todos, read_project_meta};

/// Serialize the project as a JSON-serializable value.
///
/// Set `include_audit_log` to true to include the audit log (expensive — scans
/// entire Automerge change history). The web API skips this for performance.
pub fn to_json(doc: &mut AutoCommit, include_audit_log: bool) -> Value {
    let (id, prefix, name, description) = read_project_meta(doc);
    let members = read_all_members(doc);
    let todos = read_all_todos(doc);
    let audit_log = if include_audit_log {
        crate::history::get_audit_log(doc, None, None)
    } else {
        vec![]
    };

    json!({
        "id": id,
        "prefix": prefix,
        "name": name,
        "description": description,
        "members": members.iter().map(|m| {
            let mut member = json!({
                "id": m.id,
                "name": m.name,
                "email": m.email,
                "role": m.role,
            });
            if let Some(provider) = &m.agent_provider {
                member.as_object_mut().unwrap().insert("agentProvider".into(), json!(provider));
            }
            if let Some(model) = &m.agent_model {
                member.as_object_mut().unwrap().insert("agentModel".into(), json!(model));
            }
            member
        }).collect::<Vec<_>>(),
        "todos": todos.iter().map(|t| {
            let assignee_name = t.assignee.as_ref().and_then(|a| {
                members.iter().find(|m| m.id == *a).map(|m| m.name.clone())
            });
            json!({
                "id": t.id,
                "ref": format!("{}-{}", prefix, t.number),
                "number": t.number,
                "title": t.title,
                "description": t.description,
                "status": t.status,
                "priority": t.priority,
                "difficulty": t.difficulty,
                "labels": t.labels,
                "assignee": t.assignee,
                "assigneeName": assignee_name,
                "branch": t.branch,
                "worktrees": t.worktrees,
                "commits": t.commits,
                "comments": t.comments.iter().map(|c| json!({
                    "id": c.id,
                    "author": c.author,
                    "authorName": c.author_name,
                    "text": c.text,
                    "createdAt": c.created_at,
                })).collect::<Vec<_>>(),
                "createdAt": t.created_at,
                "updatedAt": t.updated_at,
                "createdBy": t.created_by,
                "platform": t.platform,
                "planPath": t.plan_path,
            })
        }).collect::<Vec<_>>(),
        "auditLog": audit_log.iter().map(|e| json!({
            "action": e.action,
            "actorId": e.actor_id,
            "actorName": e.actor_name,
            "target": e.target,
            "details": e.details,
            "timestamp": e.timestamp,
            "hash": e.hash,
        })).collect::<Vec<_>>(),
    })
}
