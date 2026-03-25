//! API route handlers.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::{json, Value};
use std::process::Command;

use agt_lib::export::to_json;
use agt_lib::inbox;
use agt_lib::operations::{self, AddTodoOpts, UpdateTodoFields};
use agt_lib::project::sync_config;
use agt_lib::queries;
use agt_lib::schema::*;

use crate::state::AppState;

fn json_ok(data: Value) -> Json<Value> {
    Json(data)
}

fn json_err(msg: &str, status: u16) -> impl IntoResponse {
    (
        axum::http::StatusCode::from_u16(status).unwrap_or(axum::http::StatusCode::BAD_REQUEST),
        Json(json!({ "error": msg })),
    )
}

// ── GET /api/project ────────────────────────────────────────────────

pub async fn get_project(State(state): State<AppState>) -> impl IntoResponse {
    let mut doc = state.doc.lock().await;

    // Ensure inbox files exist
    let _ = inbox::ensure_inbox_files(&state.todo_dir);

    let inbox_text = inbox::read_inbox(&state.todo_dir).unwrap_or_default();
    let inbox_processed = inbox::read_processed(&state.todo_dir).unwrap_or_default();

    let mut project = to_json(&mut doc, false);
    if let Some(obj) = project.as_object_mut() {
        obj.insert("inboxText".into(), json!(inbox_text));
        obj.insert("inboxProcessed".into(), json!(inbox_processed));
    }

    Json(project).into_response()
}

// ── POST /api/change ────────────────────────────────────────────────

pub async fn post_change(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let action = body.get("action").and_then(|a| a.as_str()).unwrap_or("");

    let result = match action {
        "add" => handle_add(&state, &body).await,
        "update" => handle_update(&state, &body).await,
        "delete" => handle_delete(&state, &body).await,
        "addComment" => handle_add_comment(&state, &body).await,
        "createBranch" => handle_create_branch(&state, &body).await,
        "removeBranch" => handle_remove_branch(&state, &body).await,
        "updateProject" => handle_update_project(&state, &body).await,
        "addMember" => handle_add_member(&state, &body).await,
        "removeMember" => handle_remove_member(&state, &body).await,
        "updateMember" => handle_update_member(&state, &body).await,
        "updateInbox" => handle_update_inbox(&state, &body).await,
        "bulk" => handle_bulk(&state, &body).await,
        _ => Err(format!("Unknown action: {action}")),
    };

    match result {
        Ok(data) => json_ok(data).into_response(),
        Err(msg) => json_err(&msg, 400).into_response(),
    }
}

async fn handle_add(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let title = body.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled");
    let description = body.get("description").and_then(|d| d.as_str());
    let status: Option<Status> = body.get("status").and_then(|s| s.as_str()).and_then(|s| s.parse().ok());
    let priority: Option<Priority> = body.get("priority").and_then(|p| p.as_str()).and_then(|p| p.parse().ok());
    let difficulty: Option<Difficulty> = body.get("difficulty").and_then(|d| d.as_str()).and_then(|d| d.parse().ok());
    let assignee = body.get("assignee").and_then(|a| a.as_str());

    let labels: Option<Vec<Label>> = body.get("labels").and_then(|l| l.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str())
            .filter_map(|s| s.parse().ok())
            .collect()
    });

    let number = operations::add_todo(
        &mut doc,
        AddTodoOpts {
            title,
            description,
            status,
            priority,
            difficulty,
            labels,
            assignee,
            created_by: None,
            platform: Some(Platform::Web),
        },
    )
    .map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "number": number }))
}

async fn handle_update(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;
    let number = body.get("number").and_then(|n| n.as_u64()).ok_or("missing number")?;

    let empty = json!({});
    let updates = body.get("updates").unwrap_or(&empty);
    let fields = parse_update_fields(updates);

    operations::update_todo(&mut doc, number, fields, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_delete(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;
    let number = body.get("number").and_then(|n| n.as_u64()).ok_or("missing number")?;

    operations::delete_todo(&mut doc, number, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_add_comment(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;
    let number = body.get("number").and_then(|n| n.as_u64()).ok_or("missing number")?;
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("");

    operations::add_comment(&mut doc, number, text, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_create_branch(state: &AppState, body: &Value) -> Result<Value, String> {
    let doc = state.doc.lock().await;
    let number = body.get("number").and_then(|n| n.as_u64()).ok_or("missing number")?;

    let todo = queries::find_todo_by_number(&doc, number)
        .ok_or_else(|| format!("Todo #{number} not found"))?;

    if let Some(branch) = &todo.branch {
        return Ok(json!({ "ok": true, "branch": branch, "alreadyExists": true }));
    }

    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let slug = slugify(&todo.title, 5);
    let branch_name = format!("{}-{}-{}", prefix.to_lowercase(), number, slug);

    drop(doc);

    // Get project root from data_path
    let project_path = state.data_path.parent().and_then(|p| p.parent())
        .ok_or("invalid project path")?;
    let worktree_path = project_path.join(".worktrees").join(&branch_name);

    if worktree_path.exists() {
        return Err("Worktree path already exists".into());
    }

    let output = Command::new("git")
        .args(["worktree", "add", "-b", &branch_name, worktree_path.to_str().unwrap_or("")])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to spawn git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create worktree: {}", stderr.trim()));
    }

    let mut doc = state.doc.lock().await;
    operations::set_branch(&mut doc, number, &branch_name, None).map_err(|e| e.to_string())?;
    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    Ok(json!({
        "ok": true,
        "branch": branch_name,
        "worktree": format!(".worktrees/{branch_name}"),
    }))
}

async fn handle_remove_branch(state: &AppState, body: &Value) -> Result<Value, String> {
    let doc = state.doc.lock().await;
    let number = body.get("number").and_then(|n| n.as_u64()).ok_or("missing number")?;

    let todo = queries::find_todo_by_number(&doc, number)
        .ok_or_else(|| format!("Todo #{number} not found"))?;

    let branch = todo.branch.clone().ok_or_else(|| format!("Todo #{number} has no branch"))?;
    drop(doc);

    let project_path = state.data_path.parent().and_then(|p| p.parent())
        .ok_or("invalid project path")?;
    let worktree_path = project_path.join(".worktrees").join(&branch);

    if worktree_path.exists() {
        let output = Command::new("git")
            .args(["worktree", "remove", worktree_path.to_str().unwrap_or("")])
            .current_dir(project_path)
            .output()
            .map_err(|e| format!("Failed to spawn git: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to remove worktree: {}", stderr.trim()));
        }
    }

    let mut doc = state.doc.lock().await;
    operations::clear_branch(&mut doc, number, None).map_err(|e| e.to_string())?;
    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    Ok(json!({ "ok": true, "branch": branch }))
}

async fn handle_update_project(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;
    let empty = json!({});
    let updates = body.get("updates").unwrap_or(&empty);

    let name = updates.get("name").and_then(|n| n.as_str());
    let description = updates.get("description").and_then(|d| d.as_str());
    let prefix = updates.get("prefix").and_then(|p| p.as_str());

    operations::update_project(&mut doc, name, description, prefix, None)
        .map_err(|e| e.to_string())?;

    // Sync config.toml
    let (_, current_prefix, current_name, _) = queries::read_project_meta(&doc);
    drop(doc);

    let _ = sync_config(&state.config_path, &current_prefix, &current_name);
    state.save().await.map_err(|e| e.to_string())?;

    Ok(json!({ "ok": true }))
}

async fn handle_add_member(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let name = body.get("name").and_then(|n| n.as_str()).unwrap_or("Unnamed");
    let role: MemberRole = body.get("role").and_then(|r| r.as_str())
        .and_then(|r| r.parse().ok())
        .unwrap_or(MemberRole::Member);
    let email = body.get("email").and_then(|e| e.as_str());
    let provider: Option<AgentProvider> = body.get("agentProvider")
        .and_then(|p| p.as_str())
        .and_then(|p| p.parse().ok());
    let model = body.get("agentModel").and_then(|m| m.as_str());

    operations::add_member(&mut doc, name, role, email, None, provider, model)
        .map_err(|e| e.to_string())?;

    // Get the newly added member's ID
    let members = queries::read_all_members(&doc);
    let id = members.last().map(|m| m.id.clone()).unwrap_or_default();

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    Ok(json!({ "ok": true, "id": id }))
}

async fn handle_remove_member(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let member_ref = body.get("memberId")
        .or_else(|| body.get("name"))
        .and_then(|m| m.as_str())
        .unwrap_or("");

    let member = queries::find_member(&doc, member_ref)
        .ok_or("Member not found")?;

    operations::remove_member(&mut doc, &member.id, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_update_member(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let member_ref = body.get("memberId")
        .or_else(|| body.get("name"))
        .and_then(|m| m.as_str())
        .unwrap_or("");

    let member = queries::find_member(&doc, member_ref)
        .ok_or("Member not found")?;

    let empty = json!({});
    let updates = body.get("updates").unwrap_or(&empty);
    let name = updates.get("name").and_then(|n| n.as_str());
    let email = updates.get("email").map(|e| e.as_str());
    let role: Option<MemberRole> = updates.get("role").and_then(|r| r.as_str()).and_then(|r| r.parse().ok());
    let provider: Option<AgentProvider> = updates.get("agentProvider").and_then(|p| p.as_str()).and_then(|p| p.parse().ok());
    let model = updates.get("agentModel").and_then(|m| m.as_str());

    operations::update_member(&mut doc, &member.id, name, email, role, provider, model, None)
        .map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_update_inbox(state: &AppState, body: &Value) -> Result<Value, String> {
    let _ = inbox::ensure_inbox_files(&state.todo_dir);
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("");
    inbox::write_inbox(&state.todo_dir, text).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_bulk(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let ops = body.get("operations")
        .and_then(|o| o.as_array())
        .cloned()
        .unwrap_or_default();

    for op in &ops {
        let action = op.get("action").and_then(|a| a.as_str()).unwrap_or("");
        let number = op.get("number").and_then(|n| n.as_u64()).unwrap_or(0);

        match action {
            "update" => {
                let empty_obj = json!({});
                let updates = op.get("updates").unwrap_or(&empty_obj);
                let fields = parse_update_fields(updates);
                operations::update_todo(&mut doc, number, fields, None)
                    .map_err(|e| e.to_string())?;
            }
            "delete" => {
                operations::delete_todo(&mut doc, number, None)
                    .map_err(|e| e.to_string())?;
            }
            _ => return Err(format!("Unknown bulk operation: {action}")),
        }
    }

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "count": ops.len() }))
}

// ── WebSocket handler ───────────────────────────────────────────────

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(text) => {
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {} // Client messages ignored
                    _ => break,
                }
            }
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn parse_update_fields(updates: &Value) -> UpdateTodoFields<'_> {
    UpdateTodoFields {
        title: updates.get("title").and_then(|t| t.as_str()),
        description: updates.get("description").and_then(|d| d.as_str()),
        status: updates.get("status").and_then(|s| s.as_str()).and_then(|s| s.parse().ok()),
        priority: updates.get("priority").and_then(|p| p.as_str()).and_then(|p| p.parse().ok()),
        difficulty: updates.get("difficulty").and_then(|d| d.as_str()).and_then(|d| d.parse().ok()),
        labels: updates.get("labels").and_then(|l| l.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| s.parse().ok())
                .collect()
        }),
        assignee: updates.get("assignee").map(|a| a.as_str()),
        plan_path: updates.get("planPath").map(|p| p.as_str()),
    }
}

fn slugify(text: &str, max_words: usize) -> String {
    static ARTICLES: &[&str] = &["a", "an", "the"];
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !ARTICLES.contains(w))
        .take(max_words)
        .collect::<Vec<_>>()
        .join("-")
}
