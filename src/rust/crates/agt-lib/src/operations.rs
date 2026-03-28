//! Automerge mutation functions for the Project document.
//! All writes to the CRDT go through these functions.
//!
//! Audit metadata is embedded in each Automerge change's `message` field
//! as a JSON string. This leverages Automerge's built-in history tracking.

use anyhow::{Context, Result, bail};
use automerge::{
    AutoCommit, ObjType, ROOT, ReadDoc, ScalarValue, transaction::CommitOptions,
    transaction::Transactable,
};
use serde_json::json;
use uuid::Uuid;

use crate::schema::*;

// ── Document keys ───────────────────────────────────────────────────

const K_VERSION: &str = "_version";
const K_ID: &str = "id";
const K_PREFIX: &str = "prefix";
const K_NAME: &str = "name";
const K_DESCRIPTION: &str = "description";
const K_COUNTER: &str = "counter";
const K_CREATED_AT: &str = "createdAt";
const K_UPDATED_AT: &str = "updatedAt";
const K_MEMBERS: &str = "members";
const K_TODOS: &str = "todos";
const K_TITLE: &str = "title";
const K_STATUS: &str = "status";
const K_PRIORITY: &str = "priority";
const K_DIFFICULTY: &str = "difficulty";
const K_LABELS: &str = "labels";
const K_ASSIGNEE: &str = "assignee";
const K_BRANCH: &str = "branch";
const K_COMMENTS: &str = "comments";
const K_CREATED_BY: &str = "createdBy";
const K_PLATFORM: &str = "platform";
const K_NUMBER: &str = "number";
const K_EMAIL: &str = "email";
const K_ROLE: &str = "role";
const K_AGENT_PROVIDER: &str = "agentProvider";
const K_AGENT_MODEL: &str = "agentModel";
const K_AUTHOR: &str = "author";
const K_AUTHOR_NAME: &str = "authorName";
const K_TEXT: &str = "text";
const K_PLAN_PATH: &str = "planPath";
const K_WORKTREES: &str = "worktrees";
const K_COMMITS: &str = "commits";
const K_CYCLES: &str = "cycles";
const K_CYCLE_ID: &str = "cycleId";
const K_START_DATE: &str = "startDate";
const K_END_DATE: &str = "endDate";

// ── Change message helper ───────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMessage {
    pub action: String,
    pub target: String,
    pub actor_id: String,
    pub actor_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

fn commit_msg(doc: &mut AutoCommit, msg: String) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    doc.commit_with(CommitOptions::default().with_message(msg).with_time(now));
}

fn build_msg(
    doc: &AutoCommit,
    action: &str,
    actor_id: &str,
    target: &str,
    details: Option<serde_json::Value>,
) -> String {
    let actor_name = find_member_name(doc, actor_id).unwrap_or_else(|| actor_id.to_string());
    let msg = ChangeMessage {
        action: action.to_string(),
        target: target.to_string(),
        actor_id: actor_id.to_string(),
        actor_name,
        details,
    };
    serde_json::to_string(&msg).unwrap_or_default()
}

/// Read a string value from an Automerge object, handling both Scalar::Str
/// and ObjType::Text (JS Automerge stores strings as Text objects).
fn read_str(doc: &AutoCommit, obj: &automerge::ObjId, key: &str) -> Option<String> {
    match doc.get(obj, key) {
        Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
            ScalarValue::Str(s) => Some(s.to_string()),
            ScalarValue::Null => None,
            _ => None,
        },
        Ok(Some((automerge::Value::Object(automerge::ObjType::Text), id))) => doc.text(&id).ok(),
        _ => None,
    }
}

fn resolve_actor(doc: &AutoCommit, actor_id: Option<&str>) -> String {
    if let Some(id) = actor_id {
        return id.to_string();
    }
    if let Ok(Some((_, members_id))) = doc.get(ROOT, K_MEMBERS) {
        let len = doc.length(&members_id);
        if len > 0
            && let Ok(Some((_, member_id))) = doc.get(&members_id, 0usize)
            && let Some(id) = read_str(doc, &member_id, K_ID)
        {
            return id;
        }
    }
    "system".to_string()
}

fn find_member_name(doc: &AutoCommit, member_id: &str) -> Option<String> {
    let (_, members_obj) = doc.get(ROOT, K_MEMBERS).ok()??;
    let len = doc.length(&members_obj);
    for i in 0..len {
        let (_, m_obj) = doc.get(&members_obj, i).ok()??;
        if let Some(id) = read_str(doc, &m_obj, K_ID)
            && id == member_id
        {
            return read_str(doc, &m_obj, K_NAME);
        }
    }
    None
}

fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

fn get_prefix(doc: &AutoCommit) -> String {
    read_str(doc, &ROOT, K_PREFIX).unwrap_or_else(|| "TODO".to_string())
}

fn find_todo_obj(doc: &AutoCommit, todo_number: u64) -> Result<(automerge::ObjId, usize)> {
    let (_, todos_id) = doc.get(ROOT, K_TODOS)?.context("todos list not found")?;
    let len = doc.length(&todos_id);
    for i in 0..len {
        let (_, todo_id) = doc.get(&todos_id, i)?.context("todo entry missing")?;
        if let Ok(Some((automerge::Value::Scalar(s), _))) = doc.get(&todo_id, K_NUMBER) {
            let num = match s.as_ref() {
                ScalarValue::Uint(n) => *n,
                ScalarValue::Int(n) => *n as u64,
                ScalarValue::F64(n) => *n as u64,
                _ => continue,
            };
            if num == todo_number {
                return Ok((todo_id, i));
            }
        }
    }
    bail!("Todo #{todo_number} not found");
}

// ── Project operations ──────────────────────────────────────────────

/// Create a brand new empty project document.
pub fn create_project(
    prefix: &str,
    name: &str,
    owner_name: &str,
    owner_email: Option<&str>,
) -> Result<AutoCommit> {
    let mut doc = AutoCommit::new();
    let owner_id = Uuid::new_v4().to_string();
    let project_id = Uuid::new_v4().to_string();
    let now = now_millis();

    doc.put(ROOT, K_VERSION, CURRENT_SCHEMA_VERSION as i64)?;
    doc.put(ROOT, K_ID, project_id)?;
    doc.put(ROOT, K_PREFIX, prefix.to_uppercase())?;
    doc.put(ROOT, K_NAME, name)?;
    doc.put(ROOT, K_DESCRIPTION, "")?;
    doc.put(ROOT, K_COUNTER, ScalarValue::counter(0))?;
    doc.put(ROOT, K_CREATED_AT, now)?;

    let members = doc.put_object(ROOT, K_MEMBERS, ObjType::List)?;
    let member = doc.insert_object(&members, 0, ObjType::Map)?;
    doc.put(&member, K_ID, owner_id)?;
    doc.put(&member, K_NAME, owner_name)?;
    if let Some(email) = owner_email {
        doc.put(&member, K_EMAIL, email)?;
    } else {
        doc.put(&member, K_EMAIL, ScalarValue::Null)?;
    }
    doc.put(&member, K_ROLE, MemberRole::Owner.as_str())?;

    doc.put_object(ROOT, K_TODOS, ObjType::List)?;
    doc.put_object(ROOT, K_CYCLES, ObjType::List)?;

    commit_msg(&mut doc, "project.created".to_string());
    Ok(doc)
}

/// Update project metadata.
pub fn update_project(
    doc: &mut AutoCommit,
    name: Option<&str>,
    description: Option<&str>,
    prefix: Option<&str>,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let mut changed = serde_json::Map::new();

    if let Some(name) = name {
        doc.put(ROOT, K_NAME, name)?;
        changed.insert("name".into(), json!(name));
    }
    if let Some(desc) = description {
        doc.put(ROOT, K_DESCRIPTION, desc)?;
        changed.insert("description".into(), json!(desc));
    }
    if let Some(pfx) = prefix {
        doc.put(ROOT, K_PREFIX, pfx.to_uppercase())?;
        changed.insert("prefix".into(), json!(pfx.to_uppercase()));
    }

    let proj_name = get_prefix(doc);
    let msg = build_msg(
        doc,
        "project.updated",
        &actor,
        &proj_name,
        Some(json!(changed)),
    );
    commit_msg(doc, msg);
    Ok(())
}

// ── Todo operations ─────────────────────────────────────────────────

pub struct AddTodoOpts<'a> {
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub difficulty: Option<Difficulty>,
    pub labels: Option<Vec<Label>>,
    pub assignee: Option<&'a str>,
    pub created_by: Option<&'a str>,
    pub platform: Option<Platform>,
}

/// Add a new todo and return its number.
pub fn add_todo(doc: &mut AutoCommit, opts: AddTodoOpts<'_>) -> Result<u64> {
    let actor = resolve_actor(doc, opts.created_by);
    let now = now_millis();
    let todo_id_str = Uuid::new_v4().to_string();
    let status = opts.status.unwrap_or(Status::None);
    let priority = opts.priority.unwrap_or(Priority::None);
    let difficulty = opts.difficulty.unwrap_or(Difficulty::None);
    let platform = opts.platform.unwrap_or(Platform::Unknown);

    // Increment counter
    doc.increment(ROOT, K_COUNTER, 1)?;
    let counter_val = match doc.get(ROOT, K_COUNTER)? {
        Some((automerge::Value::Scalar(s), _)) => match s.as_ref() {
            ScalarValue::Counter(c) => i64::from(c),
            _ => bail!("counter is not a Counter type"),
        },
        _ => bail!("counter not found"),
    };
    let todo_number = counter_val as u64;

    let (_, todos_id) = doc.get(ROOT, K_TODOS)?.context("todos list not found")?;
    let len = doc.length(&todos_id);

    let todo_obj = doc.insert_object(&todos_id, len, ObjType::Map)?;
    doc.put(&todo_obj, K_ID, todo_id_str)?;
    doc.put(&todo_obj, K_NUMBER, todo_number as i64)?;
    doc.put(&todo_obj, K_TITLE, opts.title)?;
    doc.put(&todo_obj, K_DESCRIPTION, opts.description.unwrap_or(""))?;
    doc.put(&todo_obj, K_STATUS, status.as_str())?;
    doc.put(&todo_obj, K_PRIORITY, priority.as_str())?;
    doc.put(&todo_obj, K_DIFFICULTY, difficulty.as_str())?;

    let labels_obj = doc.put_object(&todo_obj, K_LABELS, ObjType::List)?;
    if let Some(labels) = &opts.labels {
        for (i, label) in labels.iter().enumerate() {
            doc.insert(&labels_obj, i, label.as_str())?;
        }
    }

    if let Some(assignee) = opts.assignee {
        doc.put(&todo_obj, K_ASSIGNEE, assignee)?;
    } else {
        doc.put(&todo_obj, K_ASSIGNEE, ScalarValue::Null)?;
    }
    doc.put(&todo_obj, K_BRANCH, ScalarValue::Null)?;
    doc.put_object(&todo_obj, K_WORKTREES, ObjType::List)?;
    doc.put_object(&todo_obj, K_COMMITS, ObjType::List)?;
    doc.put_object(&todo_obj, K_COMMENTS, ObjType::List)?;
    doc.put(&todo_obj, K_CREATED_AT, now)?;
    doc.put(&todo_obj, K_UPDATED_AT, now)?;
    doc.put(&todo_obj, K_CREATED_BY, actor.as_str())?;
    doc.put(&todo_obj, K_PLATFORM, platform.as_str())?;
    doc.put(&todo_obj, K_PLAN_PATH, ScalarValue::Null)?;
    doc.put(&todo_obj, K_CYCLE_ID, ScalarValue::Null)?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.created",
        &actor,
        &format!("{prefix}-{todo_number}"),
        Some(json!({
            "title": opts.title,
            "status": status.as_str(),
            "priority": priority.as_str(),
        })),
    );
    commit_msg(doc, msg);
    Ok(todo_number)
}

#[derive(Default)]
pub struct UpdateTodoFields<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub status: Option<Status>,
    pub priority: Option<Priority>,
    pub difficulty: Option<Difficulty>,
    pub labels: Option<Vec<Label>>,
    pub assignee: Option<Option<&'a str>>,
    pub plan_path: Option<Option<&'a str>>,
    pub cycle_id: Option<Option<&'a str>>,
}

pub fn update_todo(
    doc: &mut AutoCommit,
    todo_number: u64,
    updates: UpdateTodoFields<'_>,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (todo_obj, _) = find_todo_obj(doc, todo_number)?;
    let mut changed = serde_json::Map::new();

    if let Some(title) = updates.title {
        doc.put(&todo_obj, K_TITLE, title)?;
        changed.insert("title".into(), json!(title));
    }
    if let Some(desc) = updates.description {
        doc.put(&todo_obj, K_DESCRIPTION, desc)?;
        changed.insert("description".into(), json!(desc));
    }
    if let Some(status) = updates.status {
        doc.put(&todo_obj, K_STATUS, status.as_str())?;
        changed.insert("status".into(), json!(status.as_str()));
    }
    if let Some(priority) = updates.priority {
        doc.put(&todo_obj, K_PRIORITY, priority.as_str())?;
        changed.insert("priority".into(), json!(priority.as_str()));
    }
    if let Some(difficulty) = updates.difficulty {
        doc.put(&todo_obj, K_DIFFICULTY, difficulty.as_str())?;
        changed.insert("difficulty".into(), json!(difficulty.as_str()));
    }
    if let Some(labels) = &updates.labels {
        let labels_obj = doc.put_object(&todo_obj, K_LABELS, ObjType::List)?;
        for (i, label) in labels.iter().enumerate() {
            doc.insert(&labels_obj, i, label.as_str())?;
        }
        changed.insert(
            "labels".into(),
            json!(labels.iter().map(|l| l.as_str()).collect::<Vec<_>>()),
        );
    }
    if let Some(assignee_opt) = updates.assignee {
        if let Some(assignee) = assignee_opt {
            doc.put(&todo_obj, K_ASSIGNEE, assignee)?;
            changed.insert("assignee".into(), json!(assignee));
        } else {
            doc.put(&todo_obj, K_ASSIGNEE, ScalarValue::Null)?;
            changed.insert("assignee".into(), json!(null));
        }
    }
    if let Some(plan_path_opt) = updates.plan_path {
        if let Some(plan_path) = plan_path_opt {
            doc.put(&todo_obj, K_PLAN_PATH, plan_path)?;
            changed.insert("planPath".into(), json!(plan_path));
        } else {
            doc.put(&todo_obj, K_PLAN_PATH, ScalarValue::Null)?;
            changed.insert("planPath".into(), json!(null));
        }
    }
    if let Some(cycle_id_opt) = updates.cycle_id {
        if let Some(cycle_id) = cycle_id_opt {
            doc.put(&todo_obj, K_CYCLE_ID, cycle_id)?;
            changed.insert("cycleId".into(), json!(cycle_id));
        } else {
            doc.put(&todo_obj, K_CYCLE_ID, ScalarValue::Null)?;
            changed.insert("cycleId".into(), json!(null));
        }
    }

    doc.put(&todo_obj, K_UPDATED_AT, now_millis())?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.updated",
        &actor,
        &format!("{prefix}-{todo_number}"),
        Some(json!(changed)),
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn delete_todo(doc: &mut AutoCommit, todo_number: u64, actor_id: Option<&str>) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (_, idx) = find_todo_obj(doc, todo_number)?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.deleted",
        &actor,
        &format!("{prefix}-{todo_number}"),
        None,
    );

    let (_, todos_id) = doc.get(ROOT, K_TODOS)?.context("todos list not found")?;
    doc.delete(&todos_id, idx)?;

    commit_msg(doc, msg);
    Ok(())
}

pub fn unassign_todo(doc: &mut AutoCommit, todo_number: u64, actor_id: Option<&str>) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (todo_obj, _) = find_todo_obj(doc, todo_number)?;

    doc.put(&todo_obj, K_ASSIGNEE, ScalarValue::Null)?;
    doc.put(&todo_obj, K_UPDATED_AT, now_millis())?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.unassigned",
        &actor,
        &format!("{prefix}-{todo_number}"),
        None,
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn add_comment(
    doc: &mut AutoCommit,
    todo_number: u64,
    text: &str,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (todo_obj, _) = find_todo_obj(doc, todo_number)?;

    let actor_name = find_member_name(doc, &actor).unwrap_or_else(|| actor.clone());

    let comments_id = match doc.get(&todo_obj, K_COMMENTS)? {
        Some((_, id)) => id,
        None => doc.put_object(&todo_obj, K_COMMENTS, ObjType::List)?,
    };

    let len = doc.length(&comments_id);
    let comment_obj = doc.insert_object(&comments_id, len, ObjType::Map)?;

    doc.put(&comment_obj, K_ID, Uuid::new_v4().to_string())?;
    doc.put(&comment_obj, K_AUTHOR, actor.as_str())?;
    doc.put(&comment_obj, K_AUTHOR_NAME, actor_name.as_str())?;
    doc.put(&comment_obj, K_TEXT, text)?;
    doc.put(&comment_obj, K_CREATED_AT, now_millis())?;
    doc.put(&todo_obj, K_UPDATED_AT, now_millis())?;

    let prefix = get_prefix(doc);
    let truncated = if text.len() > 100 {
        format!("{}...", &text[..100])
    } else {
        text.to_string()
    };
    let msg = build_msg(
        doc,
        "todo.commented",
        &actor,
        &format!("{prefix}-{todo_number}"),
        Some(json!({ "text": truncated })),
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn set_branch(
    doc: &mut AutoCommit,
    todo_number: u64,
    branch_name: &str,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (todo_obj, _) = find_todo_obj(doc, todo_number)?;

    doc.put(&todo_obj, K_BRANCH, branch_name)?;
    doc.put(&todo_obj, K_UPDATED_AT, now_millis())?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.branched",
        &actor,
        &format!("{prefix}-{todo_number}"),
        Some(json!({ "branch": branch_name })),
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn clear_branch(doc: &mut AutoCommit, todo_number: u64, actor_id: Option<&str>) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (todo_obj, _) = find_todo_obj(doc, todo_number)?;

    doc.put(&todo_obj, K_BRANCH, ScalarValue::Null)?;
    doc.put(&todo_obj, K_UPDATED_AT, now_millis())?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.unbranched",
        &actor,
        &format!("{prefix}-{todo_number}"),
        None,
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn link_worktree(
    doc: &mut AutoCommit,
    todo_number: u64,
    worktree_path: &str,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (todo_obj, _) = find_todo_obj(doc, todo_number)?;

    let worktrees_id = match doc.get(&todo_obj, K_WORKTREES)? {
        Some((_, id)) => id,
        None => doc.put_object(&todo_obj, K_WORKTREES, ObjType::List)?,
    };

    let len = doc.length(&worktrees_id);
    for i in 0..len {
        if let Ok(Some((automerge::Value::Scalar(s), _))) = doc.get(&worktrees_id, i)
            && let ScalarValue::Str(existing) = s.as_ref()
            && AsRef::<str>::as_ref(existing) == worktree_path
        {
            return Ok(());
        }
    }

    doc.insert(&worktrees_id, len, worktree_path)?;
    doc.put(&todo_obj, K_UPDATED_AT, now_millis())?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.worktree_linked",
        &actor,
        &format!("{prefix}-{todo_number}"),
        Some(json!({ "worktree": worktree_path })),
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn link_commit(
    doc: &mut AutoCommit,
    todo_number: u64,
    commit_sha: &str,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (todo_obj, _) = find_todo_obj(doc, todo_number)?;

    let commits_id = match doc.get(&todo_obj, K_COMMITS)? {
        Some((_, id)) => id,
        None => doc.put_object(&todo_obj, K_COMMITS, ObjType::List)?,
    };

    let len = doc.length(&commits_id);
    for i in 0..len {
        if let Ok(Some((automerge::Value::Scalar(s), _))) = doc.get(&commits_id, i)
            && let ScalarValue::Str(existing) = s.as_ref()
            && AsRef::<str>::as_ref(existing) == commit_sha
        {
            return Ok(());
        }
    }

    doc.insert(&commits_id, len, commit_sha)?;
    doc.put(&todo_obj, K_UPDATED_AT, now_millis())?;

    let prefix = get_prefix(doc);
    let msg = build_msg(
        doc,
        "todo.commit_linked",
        &actor,
        &format!("{prefix}-{todo_number}"),
        Some(json!({ "commit": commit_sha })),
    );
    commit_msg(doc, msg);
    Ok(())
}

// ── Member operations ───────────────────────────────────────────────

pub fn add_member(
    doc: &mut AutoCommit,
    name: &str,
    role: MemberRole,
    email: Option<&str>,
    actor_id: Option<&str>,
    agent_provider: Option<AgentProvider>,
    agent_model: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);

    let (_, members_id) = doc
        .get(ROOT, K_MEMBERS)?
        .context("members list not found")?;
    let len = doc.length(&members_id);

    let member_obj = doc.insert_object(&members_id, len, ObjType::Map)?;
    doc.put(&member_obj, K_ID, Uuid::new_v4().to_string())?;
    doc.put(&member_obj, K_NAME, name)?;
    if let Some(email) = email {
        doc.put(&member_obj, K_EMAIL, email)?;
    } else {
        doc.put(&member_obj, K_EMAIL, ScalarValue::Null)?;
    }
    doc.put(&member_obj, K_ROLE, role.as_str())?;

    if role == MemberRole::Agent {
        if let Some(provider) = agent_provider {
            doc.put(&member_obj, K_AGENT_PROVIDER, provider.as_str())?;
        }
        if let Some(model) = agent_model {
            doc.put(&member_obj, K_AGENT_MODEL, model)?;
        }
    }

    let msg = build_msg(
        doc,
        "member.added",
        &actor,
        name,
        Some(json!({ "role": role.as_str() })),
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn remove_member(doc: &mut AutoCommit, member_id: &str, actor_id: Option<&str>) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);

    let (_, members_id) = doc
        .get(ROOT, K_MEMBERS)?
        .context("members list not found")?;
    let members_len = doc.length(&members_id);

    let mut member_idx = None;
    let mut member_name = String::new();
    for i in 0..members_len {
        let (_, m_obj) = doc.get(&members_id, i)?.context("member missing")?;
        if let Some(id) = read_str(doc, &m_obj, K_ID)
            && id == member_id
        {
            member_idx = Some(i);
            member_name = read_str(doc, &m_obj, K_NAME).unwrap_or_default();
            break;
        }
    }

    let idx = member_idx.context(format!("Member \"{member_id}\" not found"))?;

    // Unassign any todos assigned to this member
    let (_, todos_id) = doc.get(ROOT, K_TODOS)?.context("todos list not found")?;
    let todos_len = doc.length(&todos_id);
    for i in 0..todos_len {
        let (_, t_obj) = doc.get(&todos_id, i)?.context("todo missing")?;
        if let Some(assignee) = read_str(doc, &t_obj, K_ASSIGNEE)
            && assignee == member_id
        {
            doc.put(&t_obj, K_ASSIGNEE, ScalarValue::Null)?;
        }
    }

    let msg = build_msg(doc, "member.removed", &actor, &member_name, None);
    doc.delete(&members_id, idx)?;
    commit_msg(doc, msg);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update_member(
    doc: &mut AutoCommit,
    member_id: &str,
    name: Option<&str>,
    email: Option<Option<&str>>,
    role: Option<MemberRole>,
    agent_provider: Option<AgentProvider>,
    agent_model: Option<&str>,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);

    let (_, members_id) = doc
        .get(ROOT, K_MEMBERS)?
        .context("members list not found")?;
    let members_len = doc.length(&members_id);

    let mut member_obj = None;
    let mut current_name = String::new();
    for i in 0..members_len {
        let (_, m_obj) = doc.get(&members_id, i)?.context("member missing")?;
        if let Some(id) = read_str(doc, &m_obj, K_ID)
            && id == member_id
        {
            current_name = read_str(doc, &m_obj, K_NAME).unwrap_or_default();
            member_obj = Some(m_obj);
            break;
        }
    }

    let m_obj = member_obj.context(format!("Member \"{member_id}\" not found"))?;
    let mut changed = serde_json::Map::new();

    if let Some(name) = name {
        doc.put(&m_obj, K_NAME, name)?;
        changed.insert("name".into(), json!(name));
    }
    if let Some(email_opt) = email {
        if let Some(email) = email_opt {
            doc.put(&m_obj, K_EMAIL, email)?;
        } else {
            doc.put(&m_obj, K_EMAIL, ScalarValue::Null)?;
        }
    }
    if let Some(role) = role {
        doc.put(&m_obj, K_ROLE, role.as_str())?;
        changed.insert("role".into(), json!(role.as_str()));
    }
    if let Some(provider) = agent_provider {
        doc.put(&m_obj, K_AGENT_PROVIDER, provider.as_str())?;
    }
    if let Some(model) = agent_model {
        doc.put(&m_obj, K_AGENT_MODEL, model)?;
    }

    let target = name.unwrap_or(&current_name);
    let msg = build_msg(doc, "member.updated", &actor, target, Some(json!(changed)));
    commit_msg(doc, msg);
    Ok(())
}

// ── Cycle operations ───────────────────────────────────────────────

fn find_cycle_obj(doc: &AutoCommit, cycle_id: &str) -> Result<(automerge::ObjId, usize)> {
    let (_, cycles_id) = doc.get(ROOT, K_CYCLES)?.context("cycles list not found")?;
    let len = doc.length(&cycles_id);
    for i in 0..len {
        let (_, c_id) = doc.get(&cycles_id, i)?.context("cycle entry missing")?;
        if let Some(id) = read_str(doc, &c_id, K_ID)
            && id == cycle_id
        {
            return Ok((c_id, i));
        }
    }
    bail!("Cycle \"{cycle_id}\" not found");
}

pub struct AddCycleOpts<'a> {
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub status: Option<CycleStatus>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
    pub created_by: Option<&'a str>,
}

/// Add a new cycle and return its UUID.
pub fn add_cycle(doc: &mut AutoCommit, opts: AddCycleOpts<'_>) -> Result<String> {
    let actor = resolve_actor(doc, opts.created_by);
    let now = now_millis();
    let cycle_id = Uuid::new_v4().to_string();
    let status = opts.status.unwrap_or(CycleStatus::Planning);

    let (_, cycles_id) = doc.get(ROOT, K_CYCLES)?.context("cycles list not found")?;
    let len = doc.length(&cycles_id);

    let cycle_obj = doc.insert_object(&cycles_id, len, ObjType::Map)?;
    doc.put(&cycle_obj, K_ID, cycle_id.as_str())?;
    doc.put(&cycle_obj, K_NAME, opts.name)?;
    doc.put(&cycle_obj, K_DESCRIPTION, opts.description.unwrap_or(""))?;
    doc.put(&cycle_obj, K_STATUS, status.as_str())?;
    if let Some(start) = opts.start_date {
        doc.put(&cycle_obj, K_START_DATE, start)?;
    } else {
        doc.put(&cycle_obj, K_START_DATE, ScalarValue::Null)?;
    }
    if let Some(end) = opts.end_date {
        doc.put(&cycle_obj, K_END_DATE, end)?;
    } else {
        doc.put(&cycle_obj, K_END_DATE, ScalarValue::Null)?;
    }
    doc.put(&cycle_obj, K_CREATED_AT, now)?;
    doc.put(&cycle_obj, K_UPDATED_AT, now)?;
    doc.put(&cycle_obj, K_CREATED_BY, actor.as_str())?;

    let msg = build_msg(
        doc,
        "cycle.created",
        &actor,
        opts.name,
        Some(json!({ "status": status.as_str() })),
    );
    commit_msg(doc, msg);
    Ok(cycle_id)
}

#[derive(Default)]
pub struct UpdateCycleFields<'a> {
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub status: Option<CycleStatus>,
    pub start_date: Option<Option<&'a str>>,
    pub end_date: Option<Option<&'a str>>,
}

pub fn update_cycle(
    doc: &mut AutoCommit,
    cycle_id: &str,
    updates: UpdateCycleFields<'_>,
    actor_id: Option<&str>,
) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (cycle_obj, _) = find_cycle_obj(doc, cycle_id)?;
    let mut changed = serde_json::Map::new();

    if let Some(name) = updates.name {
        doc.put(&cycle_obj, K_NAME, name)?;
        changed.insert("name".into(), json!(name));
    }
    if let Some(desc) = updates.description {
        doc.put(&cycle_obj, K_DESCRIPTION, desc)?;
        changed.insert("description".into(), json!(desc));
    }
    if let Some(status) = updates.status {
        doc.put(&cycle_obj, K_STATUS, status.as_str())?;
        changed.insert("status".into(), json!(status.as_str()));
    }
    if let Some(start_opt) = updates.start_date {
        if let Some(start) = start_opt {
            doc.put(&cycle_obj, K_START_DATE, start)?;
            changed.insert("startDate".into(), json!(start));
        } else {
            doc.put(&cycle_obj, K_START_DATE, ScalarValue::Null)?;
            changed.insert("startDate".into(), json!(null));
        }
    }
    if let Some(end_opt) = updates.end_date {
        if let Some(end) = end_opt {
            doc.put(&cycle_obj, K_END_DATE, end)?;
            changed.insert("endDate".into(), json!(end));
        } else {
            doc.put(&cycle_obj, K_END_DATE, ScalarValue::Null)?;
            changed.insert("endDate".into(), json!(null));
        }
    }

    doc.put(&cycle_obj, K_UPDATED_AT, now_millis())?;

    let cycle_name = read_str(doc, &cycle_obj, K_NAME).unwrap_or_default();
    let msg = build_msg(
        doc,
        "cycle.updated",
        &actor,
        &cycle_name,
        Some(json!(changed)),
    );
    commit_msg(doc, msg);
    Ok(())
}

pub fn delete_cycle(doc: &mut AutoCommit, cycle_id: &str, actor_id: Option<&str>) -> Result<()> {
    let actor = resolve_actor(doc, actor_id);
    let (cycle_obj, idx) = find_cycle_obj(doc, cycle_id)?;
    let cycle_name = read_str(doc, &cycle_obj, K_NAME).unwrap_or_default();

    // Unset cycleId on any todos referencing this cycle
    let (_, todos_id) = doc.get(ROOT, K_TODOS)?.context("todos list not found")?;
    let todos_len = doc.length(&todos_id);
    for i in 0..todos_len {
        let (_, t_obj) = doc.get(&todos_id, i)?.context("todo missing")?;
        if let Some(cid) = read_str(doc, &t_obj, K_CYCLE_ID)
            && cid == cycle_id
        {
            doc.put(&t_obj, K_CYCLE_ID, ScalarValue::Null)?;
        }
    }

    let msg = build_msg(doc, "cycle.deleted", &actor, &cycle_name, None);
    let (_, cycles_id) = doc.get(ROOT, K_CYCLES)?.context("cycles list not found")?;
    doc.delete(&cycles_id, idx)?;
    commit_msg(doc, msg);
    Ok(())
}
