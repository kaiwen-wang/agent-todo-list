//! Read/filter functions for querying the Project document.
//! All functions are pure — they read from an Automerge doc without mutating.

use automerge::{AutoCommit, ReadDoc, ScalarValue, ROOT};

use crate::git_identity::get_git_identity;
use crate::schema::*;

pub struct TodoFilter {
    pub status: Option<Vec<Status>>,
    pub priority: Option<Vec<Priority>>,
    pub difficulty: Option<Vec<Difficulty>>,
    pub assignee: Option<String>,
    pub search: Option<String>,
}

impl Default for TodoFilter {
    fn default() -> Self {
        Self {
            status: None,
            priority: None,
            difficulty: None,
            assignee: None,
            search: None,
        }
    }
}

/// Read all todos from the document as plain Rust structs.
pub fn read_all_todos(doc: &AutoCommit) -> Vec<Todo> {
    let Some((_, todos_id)) = doc.get(ROOT, "todos").ok().flatten() else {
        return vec![];
    };
    let len = doc.length(&todos_id);
    let mut todos = Vec::with_capacity(len as usize);

    for i in 0..len {
        if let Some(todo) = read_todo_at(doc, &todos_id, i as usize) {
            todos.push(todo);
        }
    }
    todos
}

/// Read all members from the document.
pub fn read_all_members(doc: &AutoCommit) -> Vec<Member> {
    let Some((_, members_id)) = doc.get(ROOT, "members").ok().flatten() else {
        return vec![];
    };
    let len = doc.length(&members_id);
    let mut members = Vec::with_capacity(len as usize);

    for i in 0..len {
        if let Some(member) = read_member_at(doc, &members_id, i as usize) {
            members.push(member);
        }
    }
    members
}

fn get_str(doc: &AutoCommit, obj: &automerge::ObjId, key: &str) -> Option<String> {
    match doc.get(obj, key) {
        Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
            ScalarValue::Str(s) => Some(s.to_string()),
            ScalarValue::Null => None,
            _ => None,
        },
        _ => None,
    }
}

fn get_i64(doc: &AutoCommit, obj: &automerge::ObjId, key: &str) -> Option<i64> {
    match doc.get(obj, key) {
        Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
            ScalarValue::Int(n) => Some(*n),
            ScalarValue::Uint(n) => Some(*n as i64),
            ScalarValue::F64(n) => Some(*n as i64),
            ScalarValue::Timestamp(n) => Some(*n),
            ScalarValue::Counter(c) => Some(i64::from(c)),
            _ => None,
        },
        _ => None,
    }
}

fn get_u64(doc: &AutoCommit, obj: &automerge::ObjId, key: &str) -> Option<u64> {
    get_i64(doc, obj, key).map(|n| n as u64)
}

fn get_nullable_str(doc: &AutoCommit, obj: &automerge::ObjId, key: &str) -> Option<String> {
    get_str(doc, obj, key)
}

fn read_todo_at(doc: &AutoCommit, todos_id: &automerge::ObjId, idx: usize) -> Option<Todo> {
    let (_, todo_id) = doc.get(todos_id, idx).ok()??;

    let id = get_str(doc, &todo_id, "id").unwrap_or_default();
    let number = get_u64(doc, &todo_id, "number").unwrap_or(0);
    let title = get_str(doc, &todo_id, "title").unwrap_or_default();
    let description = get_str(doc, &todo_id, "description").unwrap_or_default();
    let status: Status = get_str(doc, &todo_id, "status")
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    let priority: Priority = get_str(doc, &todo_id, "priority")
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    let difficulty: Difficulty = get_str(doc, &todo_id, "difficulty")
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    let platform: Platform = get_str(doc, &todo_id, "platform")
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();
    let assignee = get_nullable_str(doc, &todo_id, "assignee");
    let branch = get_nullable_str(doc, &todo_id, "branch");
    let created_at = get_i64(doc, &todo_id, "createdAt").unwrap_or(0);
    let updated_at = get_i64(doc, &todo_id, "updatedAt").unwrap_or(0);
    let created_by = get_str(doc, &todo_id, "createdBy").unwrap_or_default();

    // Read labels
    let mut labels = Vec::new();
    if let Ok(Some((_, labels_id))) = doc.get(&todo_id, "labels") {
        let labels_len = doc.length(&labels_id);
        for j in 0..labels_len {
            if let Ok(Some((automerge::Value::Scalar(s), _))) = doc.get(&labels_id, j as usize) {
                if let ScalarValue::Str(label_str) = s.as_ref() {
                    if let Ok(label) = label_str.parse::<Label>() {
                        labels.push(label);
                    }
                }
            }
        }
    }

    // Read comments
    let mut comments = Vec::new();
    if let Ok(Some((_, comments_id))) = doc.get(&todo_id, "comments") {
        let comments_len = doc.length(&comments_id);
        for j in 0..comments_len {
            if let Ok(Some((_, c_id))) = doc.get(&comments_id, j as usize) {
                let comment = Comment {
                    id: get_str(doc, &c_id, "id").unwrap_or_default(),
                    author: get_str(doc, &c_id, "author").unwrap_or_default(),
                    author_name: get_str(doc, &c_id, "authorName").unwrap_or_default(),
                    text: get_str(doc, &c_id, "text").unwrap_or_default(),
                    created_at: get_i64(doc, &c_id, "createdAt").unwrap_or(0),
                };
                comments.push(comment);
            }
        }
    }

    Some(Todo {
        id,
        number,
        title,
        description,
        status,
        priority,
        difficulty,
        labels,
        assignee,
        branch,
        comments,
        created_at,
        updated_at,
        created_by,
        platform,
    })
}

fn read_member_at(
    doc: &AutoCommit,
    members_id: &automerge::ObjId,
    idx: usize,
) -> Option<Member> {
    let (_, m_id) = doc.get(members_id, idx).ok()??;

    Some(Member {
        id: get_str(doc, &m_id, "id").unwrap_or_default(),
        name: get_str(doc, &m_id, "name").unwrap_or_default(),
        email: get_nullable_str(doc, &m_id, "email"),
        role: get_str(doc, &m_id, "role")
            .and_then(|s| s.parse().ok())
            .unwrap_or(MemberRole::Member),
        agent_provider: get_str(doc, &m_id, "agentProvider")
            .and_then(|s| s.parse().ok()),
        agent_model: get_str(doc, &m_id, "agentModel"),
    })
}

/// Get all todos, optionally filtered. Excludes archived/wont_do by default.
pub fn query_todos(doc: &AutoCommit, filter: &TodoFilter) -> Vec<Todo> {
    let mut todos = read_all_todos(doc);

    // Filter by status
    if let Some(statuses) = &filter.status {
        todos.retain(|t| statuses.contains(&t.status));
    } else {
        todos.retain(|t| t.status != Status::Archived && t.status != Status::WontDo);
    }

    if let Some(priorities) = &filter.priority {
        todos.retain(|t| priorities.contains(&t.priority));
    }

    if let Some(difficulties) = &filter.difficulty {
        todos.retain(|t| difficulties.contains(&t.difficulty));
    }

    if let Some(assignee) = &filter.assignee {
        let assignee_lower = assignee.to_lowercase();
        let members = read_all_members(doc);
        todos.retain(|t| {
            if let Some(a) = &t.assignee {
                if a.to_lowercase() == assignee_lower {
                    return true;
                }
                if let Some(member) = members.iter().find(|m| m.id == *a) {
                    return member.name.to_lowercase().contains(&assignee_lower);
                }
            }
            false
        });
    }

    if let Some(search) = &filter.search {
        let search_lower = search.to_lowercase();
        todos.retain(|t| {
            t.title.to_lowercase().contains(&search_lower)
                || t.description.to_lowercase().contains(&search_lower)
        });
    }

    todos
}

/// Find a single todo by its number.
pub fn find_todo_by_number(doc: &AutoCommit, num: u64) -> Option<Todo> {
    read_all_todos(doc).into_iter().find(|t| t.number == num)
}

/// Parse a todo reference like "ABC-1" into the number part.
pub fn parse_todo_ref(reference: &str, prefix: &str) -> Option<u64> {
    let upper = reference.to_uppercase();
    let expected_prefix = format!("{}-", prefix.to_uppercase());

    if upper.starts_with(&expected_prefix) {
        return upper[expected_prefix.len()..].parse().ok();
    }

    reference.parse().ok()
}

/// Find a member by name (case-insensitive), ID, or "me" (git identity).
pub fn find_member(doc: &AutoCommit, name_or_id: &str) -> Option<Member> {
    let members = read_all_members(doc);

    if name_or_id.eq_ignore_ascii_case("me") {
        let git = get_git_identity();
        if let Some(email) = &git.email {
            if let Some(m) = members
                .iter()
                .find(|m| m.email.as_deref().is_some_and(|e| e.eq_ignore_ascii_case(email)))
            {
                return Some(m.clone());
            }
        }
        if let Some(name) = &git.name {
            if let Some(m) = members
                .iter()
                .find(|m| m.name.eq_ignore_ascii_case(name))
            {
                return Some(m.clone());
            }
        }
        return None;
    }

    let lower = name_or_id.to_lowercase();
    members
        .iter()
        .find(|m| m.id == name_or_id)
        .or_else(|| members.iter().find(|m| m.name.to_lowercase() == lower))
        .or_else(|| {
            members
                .iter()
                .find(|m| m.name.to_lowercase().contains(&lower))
        })
        .cloned()
}

/// Get a count of todos grouped by status.
pub fn count_by_status(doc: &AutoCommit) -> std::collections::HashMap<Status, usize> {
    let mut counts = std::collections::HashMap::new();
    for status in Status::ALL {
        counts.insert(*status, 0);
    }
    for todo in read_all_todos(doc) {
        *counts.entry(todo.status).or_insert(0) += 1;
    }
    counts
}

/// Read project-level metadata.
pub fn read_project_meta(doc: &AutoCommit) -> (String, String, String, String) {
    let id = get_str(doc, &ROOT.into(), "id").unwrap_or_default();
    let prefix = get_str(doc, &ROOT.into(), "prefix").unwrap_or_default();
    let name = get_str(doc, &ROOT.into(), "name").unwrap_or_default();
    let description = get_str(doc, &ROOT.into(), "description").unwrap_or_default();
    (id, prefix, name, description)
}
