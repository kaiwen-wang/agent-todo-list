//! API route handlers.

use axum::Json;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use serde_json::{Value, json};
use std::fs;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use agt_lib::export::to_json;
use agt_lib::git;
use agt_lib::inbox;
use agt_lib::operations::{self, AddCycleOpts, AddTodoOpts, UpdateCycleFields, UpdateTodoFields};
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

    // Derive project root from todo_dir (parent of .todo/)
    let project_root = state.todo_dir.parent().unwrap_or(&state.todo_dir);
    let remote_url = git::remote_base_url(project_root);

    if let Some(obj) = project.as_object_mut() {
        obj.insert("inboxText".into(), json!(inbox_text));
        obj.insert("inboxProcessed".into(), json!(inbox_processed));
        obj.insert("remoteUrl".into(), json!(remote_url));
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
        "createBranchOnly" => handle_create_branch_only(&state, &body).await,
        "removeBranch" => handle_remove_branch(&state, &body).await,
        "linkCommit" => handle_link_commit(&state, &body).await,
        "updateProject" => handle_update_project(&state, &body).await,
        "addMember" => handle_add_member(&state, &body).await,
        "removeMember" => handle_remove_member(&state, &body).await,
        "updateMember" => handle_update_member(&state, &body).await,
        "updateInbox" => handle_update_inbox(&state, &body).await,
        "processInbox" => handle_process_inbox(&state).await,
        "initPlan" => handle_init_plan(&state, &body).await,
        "researchPlan" => handle_research_plan(&state, &body).await,
        "answerPlan" => handle_answer_plan(&state, &body).await,
        "bulk" => handle_bulk(&state, &body).await,
        "addCycle" => handle_add_cycle(&state, &body).await,
        "updateCycle" => handle_update_cycle(&state, &body).await,
        "deleteCycle" => handle_delete_cycle(&state, &body).await,
        _ => Err(format!("Unknown action: {action}")),
    };

    match result {
        Ok(data) => json_ok(data).into_response(),
        Err(msg) => json_err(&msg, 400).into_response(),
    }
}

async fn handle_add(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let title = body
        .get("title")
        .and_then(|t| t.as_str())
        .unwrap_or("Untitled");
    let description = body.get("description").and_then(|d| d.as_str());
    let status: Option<Status> = body
        .get("status")
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok());
    let priority: Option<Priority> = body
        .get("priority")
        .and_then(|p| p.as_str())
        .and_then(|p| p.parse().ok());
    let difficulty: Option<Difficulty> = body
        .get("difficulty")
        .and_then(|d| d.as_str())
        .and_then(|d| d.parse().ok());
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
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;

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
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;

    operations::delete_todo(&mut doc, number, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_add_comment(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("");

    operations::add_comment(&mut doc, number, text, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_create_branch(state: &AppState, body: &Value) -> Result<Value, String> {
    let doc = state.doc.lock().await;
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;

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
    let project_path = state
        .data_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or("invalid project path")?;
    let worktree_path = project_path.join(".worktrees").join(&branch_name);

    if worktree_path.exists() {
        return Err("Worktree path already exists".into());
    }

    let output = Command::new("git")
        .args([
            "worktree",
            "add",
            "-b",
            &branch_name,
            worktree_path.to_str().unwrap_or(""),
        ])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to spawn git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create worktree: {}", stderr.trim()));
    }

    let mut doc = state.doc.lock().await;
    operations::set_branch(&mut doc, number, &branch_name, None).map_err(|e| e.to_string())?;
    let wt_rel = format!(".worktrees/{branch_name}");
    operations::link_worktree(&mut doc, number, &wt_rel, None).map_err(|e| e.to_string())?;
    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    Ok(json!({
        "ok": true,
        "branch": branch_name,
        "worktree": wt_rel,
    }))
}

async fn handle_create_branch_only(state: &AppState, body: &Value) -> Result<Value, String> {
    let doc = state.doc.lock().await;
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;

    let todo = queries::find_todo_by_number(&doc, number)
        .ok_or_else(|| format!("Todo #{number} not found"))?;

    if let Some(branch) = &todo.branch {
        return Ok(json!({ "ok": true, "branch": branch, "alreadyExists": true }));
    }

    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let slug = slugify(&todo.title, 5);
    let branch_name = format!("{}-{}-{}", prefix.to_lowercase(), number, slug);

    drop(doc);

    let project_path = state
        .data_path
        .parent()
        .and_then(|p| p.parent())
        .ok_or("invalid project path")?;

    let output = Command::new("git")
        .args(["branch", &branch_name])
        .current_dir(project_path)
        .output()
        .map_err(|e| format!("Failed to spawn git: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create branch: {}", stderr.trim()));
    }

    let mut doc = state.doc.lock().await;
    operations::set_branch(&mut doc, number, &branch_name, None).map_err(|e| e.to_string())?;
    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    Ok(json!({ "ok": true, "branch": branch_name }))
}

async fn handle_link_commit(state: &AppState, body: &Value) -> Result<Value, String> {
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;
    let commit = body
        .get("commit")
        .and_then(|c| c.as_str())
        .ok_or("missing commit")?;

    // Verify commit exists in the repo
    let project_root = state.todo_dir.parent().unwrap_or(&state.todo_dir);
    let full_sha = git::verify_commit(project_root, commit).map_err(|e| e.to_string())?;

    let mut doc = state.doc.lock().await;
    operations::link_commit(&mut doc, number, &full_sha, None).map_err(|e| e.to_string())?;
    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    Ok(json!({ "ok": true }))
}

async fn handle_remove_branch(state: &AppState, body: &Value) -> Result<Value, String> {
    let doc = state.doc.lock().await;
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;

    let todo = queries::find_todo_by_number(&doc, number)
        .ok_or_else(|| format!("Todo #{number} not found"))?;

    let branch = todo
        .branch
        .clone()
        .ok_or_else(|| format!("Todo #{number} has no branch"))?;
    drop(doc);

    let project_path = state
        .data_path
        .parent()
        .and_then(|p| p.parent())
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

    let name = body
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Unnamed");
    let role: MemberRole = body
        .get("role")
        .and_then(|r| r.as_str())
        .and_then(|r| r.parse().ok())
        .unwrap_or(MemberRole::Member);
    let email = body.get("email").and_then(|e| e.as_str());
    let provider: Option<AgentProvider> = body
        .get("agentProvider")
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

    let member_ref = body
        .get("memberId")
        .or_else(|| body.get("name"))
        .and_then(|m| m.as_str())
        .unwrap_or("");

    let member = queries::find_member(&doc, member_ref).ok_or("Member not found")?;

    operations::remove_member(&mut doc, &member.id, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_update_member(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let member_ref = body
        .get("memberId")
        .or_else(|| body.get("name"))
        .and_then(|m| m.as_str())
        .unwrap_or("");

    let member = queries::find_member(&doc, member_ref).ok_or("Member not found")?;

    let empty = json!({});
    let updates = body.get("updates").unwrap_or(&empty);
    let name = updates.get("name").and_then(|n| n.as_str());
    let email = updates.get("email").map(|e| e.as_str());
    let role: Option<MemberRole> = updates
        .get("role")
        .and_then(|r| r.as_str())
        .and_then(|r| r.parse().ok());
    let provider: Option<AgentProvider> = updates
        .get("agentProvider")
        .and_then(|p| p.as_str())
        .and_then(|p| p.parse().ok());
    let model = updates.get("agentModel").and_then(|m| m.as_str());

    operations::update_member(
        &mut doc, &member.id, name, email, role, provider, model, None,
    )
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

async fn handle_process_inbox(state: &AppState) -> Result<Value, String> {
    let _ = inbox::ensure_inbox_files(&state.todo_dir);
    let content = inbox::read_inbox(&state.todo_dir).map_err(|e| e.to_string())?;
    let items = inbox::parse_inbox_items(&content);

    if items.is_empty() {
        return Ok(json!({ "ok": true, "processed": 0, "tasks": [] }));
    }

    let mut doc = state.doc.lock().await;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let mut tasks = Vec::new();

    for item in &items {
        let number = operations::add_todo(
            &mut doc,
            AddTodoOpts {
                title: item,
                description: None,
                status: None,
                priority: None,
                difficulty: None,
                labels: None,
                assignee: None,
                created_by: None,
                platform: Some(Platform::Cli),
            },
        )
        .map_err(|e| e.to_string())?;

        let reference = format!("{}-{}", prefix, number);
        tasks.push(json!({ "ref": reference, "title": item }));
    }

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    // Archive to TODO-PROCESSED.md and clear inbox
    let entries: Vec<inbox::ProcessedEntry> = tasks
        .iter()
        .map(|t| inbox::ProcessedEntry {
            original: t["title"].as_str().unwrap_or("").to_string(),
            reference: t["ref"].as_str().unwrap_or("").to_string(),
            title: t["title"].as_str().unwrap_or("").to_string(),
        })
        .collect();
    inbox::append_processed(&state.todo_dir, &entries).map_err(|e| e.to_string())?;
    inbox::write_inbox(&state.todo_dir, "").map_err(|e| e.to_string())?;

    let count = tasks.len();
    Ok(json!({ "ok": true, "processed": count, "tasks": tasks }))
}

async fn handle_bulk(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let ops = body
        .get("operations")
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
                operations::delete_todo(&mut doc, number, None).map_err(|e| e.to_string())?;
            }
            _ => return Err(format!("Unknown bulk operation: {action}")),
        }
    }

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "count": ops.len() }))
}

// ── Cycle handlers ──────────────────────────────────────────────────

async fn handle_add_cycle(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let name = body
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("Untitled Cycle");
    let description = body.get("description").and_then(|d| d.as_str());
    let status: Option<CycleStatus> = body
        .get("status")
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok());
    let start_date = body.get("startDate").and_then(|d| d.as_str());
    let end_date = body.get("endDate").and_then(|d| d.as_str());

    let id = operations::add_cycle(
        &mut doc,
        AddCycleOpts {
            name,
            description,
            status,
            start_date,
            end_date,
            created_by: None,
        },
    )
    .map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "id": id }))
}

async fn handle_update_cycle(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let cycle_id = body
        .get("cycleId")
        .and_then(|c| c.as_str())
        .ok_or("missing cycleId")?;

    let empty = json!({});
    let updates = body.get("updates").unwrap_or(&empty);

    let fields = UpdateCycleFields {
        name: updates.get("name").and_then(|n| n.as_str()),
        description: updates.get("description").and_then(|d| d.as_str()),
        status: updates
            .get("status")
            .and_then(|s| s.as_str())
            .and_then(|s| s.parse().ok()),
        start_date: updates.get("startDate").map(|d| d.as_str()),
        end_date: updates.get("endDate").map(|d| d.as_str()),
    };

    operations::update_cycle(&mut doc, cycle_id, fields, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

async fn handle_delete_cycle(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;

    let cycle_id = body
        .get("cycleId")
        .and_then(|c| c.as_str())
        .ok_or("missing cycleId")?;

    operations::delete_cycle(&mut doc, cycle_id, None).map_err(|e| e.to_string())?;

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true }))
}

// ── Plan handlers ───────────────────────────────────────────────────

/// GET /api/plan/:number — read the plan markdown content
pub async fn get_plan(State(state): State<AppState>, Path(number): Path<u64>) -> impl IntoResponse {
    let doc = state.doc.lock().await;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);
    let todo = queries::find_todo_by_number(&doc, number);
    drop(doc);

    let Some(todo) = todo else {
        return json_err("Todo not found", 404).into_response();
    };

    let plan_file = if let Some(plan_path) = &todo.plan_path {
        state.todo_dir.join(plan_path)
    } else {
        state
            .todo_dir
            .join(format!("plans/{}-{}.md", prefix, number))
    };

    if !plan_file.exists() {
        return Json(json!({ "content": null, "exists": false })).into_response();
    }

    match fs::read_to_string(&plan_file) {
        Ok(content) => Json(json!({ "content": content, "exists": true })).into_response(),
        Err(e) => json_err(&format!("Failed to read plan: {e}"), 500).into_response(),
    }
}

async fn handle_init_plan(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;
    let (_, prefix, _, _) = queries::read_project_meta(&doc);

    let todo = queries::find_todo_by_number(&doc, number)
        .ok_or_else(|| format!("Todo #{number} not found"))?;

    let relative = format!("plans/{}-{}.md", prefix, number);
    let absolute = state.todo_dir.join(&relative);

    // Create plans directory
    let plans_dir = state.todo_dir.join("plans");
    fs::create_dir_all(&plans_dir).map_err(|e| e.to_string())?;

    if !absolute.exists() {
        let todo_ref = format!("{}-{}", prefix, number);
        let content = format!("# {}: {}\n\n", todo_ref, todo.title);
        fs::write(&absolute, &content).map_err(|e| e.to_string())?;
    }

    // Set planPath on the todo
    if todo.plan_path.is_none() {
        operations::update_todo(
            &mut doc,
            number,
            UpdateTodoFields {
                plan_path: Some(Some(&relative)),
                ..Default::default()
            },
            None,
        )
        .map_err(|e| e.to_string())?;
    }

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    let content = fs::read_to_string(&absolute).map_err(|e| e.to_string())?;
    Ok(json!({ "ok": true, "planPath": relative, "content": content }))
}

async fn handle_answer_plan(state: &AppState, body: &Value) -> Result<Value, String> {
    let doc = state.doc.lock().await;
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("");
    let (_, prefix, _, _) = queries::read_project_meta(&doc);

    let todo = queries::find_todo_by_number(&doc, number)
        .ok_or_else(|| format!("Todo #{number} not found"))?;
    drop(doc);

    let plan_file = if let Some(plan_path) = &todo.plan_path {
        state.todo_dir.join(plan_path)
    } else {
        state
            .todo_dir
            .join(format!("plans/{}-{}.md", prefix, number))
    };

    if !plan_file.exists() {
        return Err(format!(
            "No plan file for {}-{}. Init the plan first.",
            prefix, number
        ));
    }

    let mut content = fs::read_to_string(&plan_file).map_err(|e| e.to_string())?;

    if content.contains("\n## Answers\n") || content.contains("\n## Answers\r\n") {
        content.push_str(&format!("\n> {}\n", text));
    } else {
        content.push_str(&format!("\n## Answers\n\n> {}\n", text));
    }

    fs::write(&plan_file, &content).map_err(|e| e.to_string())?;

    Ok(json!({ "ok": true }))
}

async fn handle_research_plan(state: &AppState, body: &Value) -> Result<Value, String> {
    let mut doc = state.doc.lock().await;
    let number = body
        .get("number")
        .and_then(|n| n.as_u64())
        .ok_or("missing number")?;
    let (_, prefix, project_name, _) = queries::read_project_meta(&doc);

    let todo = queries::find_todo_by_number(&doc, number)
        .ok_or_else(|| format!("Todo #{number} not found"))?;

    let todo_ref = format!("{}-{}", prefix, number);
    let relative = format!("plans/{}.md", todo_ref);
    let absolute = state.todo_dir.join(&relative);

    // Create plans dir + file if needed
    let plans_dir = state.todo_dir.join("plans");
    fs::create_dir_all(&plans_dir).map_err(|e| e.to_string())?;

    if !absolute.exists() {
        let content = format!("# {}: {}\n\n", todo_ref, todo.title);
        fs::write(&absolute, &content).map_err(|e| e.to_string())?;
    }

    // Set planPath
    if todo.plan_path.is_none() {
        operations::update_todo(
            &mut doc,
            number,
            UpdateTodoFields {
                plan_path: Some(Some(&relative)),
                ..Default::default()
            },
            None,
        )
        .map_err(|e| e.to_string())?;
    }

    drop(doc);
    state.save().await.map_err(|e| e.to_string())?;

    // Build the research prompt
    let description_section = if todo.description.is_empty() {
        String::new()
    } else {
        format!("\n## Description\n{}\n", todo.description)
    };

    let comments_section = if todo.comments.is_empty() {
        String::new()
    } else {
        let comments: Vec<String> = todo
            .comments
            .iter()
            .map(|c| format!("**{}**: {}", c.author_name, c.text))
            .collect();
        format!("\n## Comments\n{}\n", comments.join("\n"))
    };

    // Read existing plan content to include as context
    let existing_content = fs::read_to_string(&absolute).unwrap_or_default();
    let existing_section = if existing_content.trim().lines().count() > 1 {
        format!(
            "\n## Existing Plan Content\n```markdown\n{}\n```\n\nBuild on this existing content. Keep what's useful, replace what needs updating.\n",
            existing_content.trim()
        )
    } else {
        String::new()
    };

    let prompt = format!(
        r#"You are researching a task for the project "{project_name}".

## Task: {todo_ref} — {title}
Priority: {priority}
Difficulty: {difficulty}
{description_section}{comments_section}{existing_section}
Research this task by reading relevant code and files in the project.

Output the complete plan as markdown, starting with `# {todo_ref}: {title}`.

Structure it with these sections:
- **## Research** — What you found, relevant context, prior art
- **## Approach Options** — Numbered options with pros/cons
- **## Recommendation** — Your suggested approach
- **## Questions** — Bullet-pointed questions for the user (if any)

Output ONLY the markdown plan content. Do not use any write tools."#,
        project_name = project_name,
        todo_ref = todo_ref,
        title = todo.title,
        priority = todo.priority.as_str(),
        difficulty = todo.difficulty.as_str(),
        description_section = description_section,
        comments_section = comments_section,
        existing_section = existing_section,
    );

    // Spawn agent in background
    let tx = state.tx.clone();
    let plan_path = absolute.clone();
    let todo_dir = state.todo_dir.clone();

    tokio::spawn(async move {
        let _ = tx.send(
            json!({
                "type": "plan:start",
                "todoRef": todo_ref,
                "number": number,
            })
            .to_string(),
        );

        let tx_blocking = tx.clone();
        let result = tokio::task::spawn_blocking(move || {
            let tx = tx_blocking;
            let mut cmd = Command::new("claude");
            cmd.arg("-p")
                .arg(&prompt)
                .arg("--output-format")
                .arg("stream-json")
                .arg("--verbose")
                .arg("--allowedTools")
                .arg("Read Glob Grep")
                .arg("--permission-mode")
                .arg("bypassPermissions")
                .current_dir(&todo_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

            let mut child = match cmd.spawn() {
                Ok(c) => c,
                Err(e) => return Err(format!("Failed to spawn claude: {e}")),
            };

            let stdout = child.stdout.take().unwrap();
            let reader = BufReader::new(stdout);
            let mut result_text = String::new();
            let mut is_error = false;

            for line in reader.lines() {
                let line = match line {
                    Ok(l) => l,
                    Err(_) => break,
                };

                if let Ok(event) = serde_json::from_str::<serde_json::Value>(&line) {
                    let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
                    match event_type {
                        "assistant" => {
                            if let Some(content) = event
                                .get("message")
                                .and_then(|m| m.get("content"))
                                .and_then(|c| c.as_array())
                            {
                                for block in content {
                                    let block_type =
                                        block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                    if block_type == "tool_use" {
                                        let tool_name = block
                                            .get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let _ = tx.send(
                                            json!({
                                                "type": "plan:progress",
                                                "message": format!("Using {tool_name}..."),
                                            })
                                            .to_string(),
                                        );
                                    }
                                }
                            }
                        }
                        "result" => {
                            is_error = event
                                .get("is_error")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            result_text = event
                                .get("result")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                        }
                        _ => {}
                    }
                }
            }

            let status = child.wait().map_err(|e| e.to_string())?;
            if status.success() && !is_error {
                Ok(result_text)
            } else {
                Err(result_text)
            }
        })
        .await;

        match result {
            Ok(Ok(content)) => {
                // Agent outputs the plan — we write it to the file
                if !content.is_empty() {
                    let _ = fs::write(&plan_path, &content);
                }
                let _ = tx.send(
                    json!({
                        "type": "plan:done",
                        "number": number,
                        "content": content,
                    })
                    .to_string(),
                );
            }
            Ok(Err(e)) => {
                let _ = tx.send(
                    json!({
                        "type": "plan:error",
                        "number": number,
                        "message": e,
                    })
                    .to_string(),
                );
            }
            Err(e) => {
                let _ = tx.send(
                    json!({
                        "type": "plan:error",
                        "number": number,
                        "message": format!("{e}"),
                    })
                    .to_string(),
                );
            }
        }
    });

    Ok(json!({ "ok": true, "planPath": relative, "researching": true }))
}

// ── WebSocket handler ───────────────────────────────────────────────

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
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
        status: updates
            .get("status")
            .and_then(|s| s.as_str())
            .and_then(|s| s.parse().ok()),
        priority: updates
            .get("priority")
            .and_then(|p| p.as_str())
            .and_then(|p| p.parse().ok()),
        difficulty: updates
            .get("difficulty")
            .and_then(|d| d.as_str())
            .and_then(|d| d.parse().ok()),
        labels: updates.get("labels").and_then(|l| l.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| s.parse().ok())
                .collect()
        }),
        assignee: updates.get("assignee").map(|a| a.as_str()),
        plan_path: updates.get("planPath").map(|p| p.as_str()),
        cycle_id: updates.get("cycleId").map(|c| c.as_str()),
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
