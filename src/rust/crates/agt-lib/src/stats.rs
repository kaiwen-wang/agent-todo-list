//! Compute project statistics from the Automerge document.

use automerge::AutoCommit;
use serde::Serialize;
use std::collections::HashMap;

use crate::queries;
use crate::schema::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStats {
    pub summary: Summary,
    pub by_status: Vec<CountEntry>,
    pub by_priority: Vec<CountEntry>,
    pub by_difficulty: Vec<CountEntry>,
    pub by_label: Vec<CountEntry>,
    pub members: Vec<MemberStats>,
    pub cycles: Vec<CycleStats>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub total: usize,
    pub active: usize,
    pub in_progress: usize,
    pub completed: usize,
    pub unassigned: usize,
    pub completion_rate: u8,
}

#[derive(Debug, Serialize)]
pub struct CountEntry {
    pub key: String,
    pub label: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberStats {
    pub id: String,
    pub name: String,
    pub role: String,
    pub active: usize,
    pub completed: usize,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CycleStats {
    pub id: String,
    pub name: String,
    pub status: String,
    pub total: usize,
    pub completed: usize,
    pub in_progress: usize,
    pub pct_done: u8,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub days_total: Option<i64>,
    pub days_elapsed: Option<i64>,
}

/// Compute all project statistics from the document.
pub fn compute_stats(doc: &AutoCommit) -> ProjectStats {
    let todos = queries::read_all_todos(doc);
    let members = queries::read_all_members(doc);
    let cycles = queries::read_all_cycles(doc);

    let total = todos.len();

    // Active = not archived/wont_do
    let active_todos: Vec<&Todo> = todos
        .iter()
        .filter(|t| t.status != Status::Archived && t.status != Status::WontDo)
        .collect();
    let active = active_todos.len();

    let in_progress = todos
        .iter()
        .filter(|t| t.status == Status::InProgress)
        .count();
    let completed = todos
        .iter()
        .filter(|t| t.status == Status::Completed)
        .count();
    let unassigned = active_todos.iter().filter(|t| t.assignee.is_none()).count();

    let closed = todos
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                Status::Completed | Status::Archived | Status::WontDo
            )
        })
        .count();
    let completion_rate = if total > 0 {
        ((closed as f64 / total as f64) * 100.0).round() as u8
    } else {
        0
    };

    // ── By status ──
    let status_counts = queries::count_by_status(doc);
    let by_status: Vec<CountEntry> = Status::ALL
        .iter()
        .filter(|s| *status_counts.get(s).unwrap_or(&0) > 0)
        .map(|s| CountEntry {
            key: s.as_str().to_string(),
            label: s.display_name().to_string(),
            count: *status_counts.get(s).unwrap_or(&0),
        })
        .collect();

    // ── By priority ──
    let mut priority_counts: HashMap<Priority, usize> = HashMap::new();
    for t in &active_todos {
        *priority_counts.entry(t.priority).or_default() += 1;
    }
    let by_priority: Vec<CountEntry> = Priority::ALL
        .iter()
        .filter(|p| *priority_counts.get(p).unwrap_or(&0) > 0)
        .map(|p| CountEntry {
            key: p.as_str().to_string(),
            label: p.display_name().to_string(),
            count: *priority_counts.get(p).unwrap_or(&0),
        })
        .collect();

    // ── By difficulty ──
    let mut difficulty_counts: HashMap<Difficulty, usize> = HashMap::new();
    for t in &active_todos {
        *difficulty_counts.entry(t.difficulty).or_default() += 1;
    }
    let by_difficulty: Vec<CountEntry> = Difficulty::ALL
        .iter()
        .filter(|d| *difficulty_counts.get(d).unwrap_or(&0) > 0)
        .map(|d| CountEntry {
            key: d.as_str().to_string(),
            label: d.display_name().to_string(),
            count: *difficulty_counts.get(d).unwrap_or(&0),
        })
        .collect();

    // ── By label ──
    let mut label_counts: HashMap<Label, usize> = HashMap::new();
    for t in &active_todos {
        for l in &t.labels {
            *label_counts.entry(*l).or_default() += 1;
        }
    }
    let by_label: Vec<CountEntry> = Label::ALL
        .iter()
        .filter(|l| *label_counts.get(l).unwrap_or(&0) > 0)
        .map(|l| CountEntry {
            key: l.as_str().to_string(),
            label: l.display_name().to_string(),
            count: *label_counts.get(l).unwrap_or(&0),
        })
        .collect();

    // ── Member workload ──
    let member_stats: Vec<MemberStats> = members
        .iter()
        .map(|m| {
            let assigned: Vec<&Todo> = todos
                .iter()
                .filter(|t| t.assignee.as_deref() == Some(&m.id))
                .collect();
            let member_active = assigned
                .iter()
                .filter(|t| {
                    t.status != Status::Archived
                        && t.status != Status::WontDo
                        && t.status != Status::Completed
                })
                .count();
            let member_completed = assigned
                .iter()
                .filter(|t| t.status == Status::Completed)
                .count();
            MemberStats {
                id: m.id.clone(),
                name: m.name.clone(),
                role: m.role.as_str().to_string(),
                active: member_active,
                completed: member_completed,
                total: assigned.len(),
            }
        })
        .filter(|s| s.total > 0)
        .collect();

    // ── Cycle progress ──
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let cycle_stats: Vec<CycleStats> = cycles
        .iter()
        .map(|c| {
            let cycle_todos: Vec<&Todo> = todos
                .iter()
                .filter(|t| t.cycle_id.as_deref() == Some(&c.id))
                .collect();
            let c_total = cycle_todos.len();
            let c_completed = cycle_todos
                .iter()
                .filter(|t| t.status == Status::Completed)
                .count();
            let c_in_progress = cycle_todos
                .iter()
                .filter(|t| t.status == Status::InProgress)
                .count();
            let pct_done = if c_total > 0 {
                ((c_completed as f64 / c_total as f64) * 100.0).round() as u8
            } else {
                0
            };

            let (days_total, days_elapsed) = match (&c.start_date, &c.end_date) {
                (Some(start), Some(end)) => {
                    if let (Ok(s), Ok(e)) = (parse_date_ms(start), parse_date_ms(end)) {
                        let ms_per_day = 86_400_000i64;
                        let dt = ((e - s) / ms_per_day).max(0);
                        let de = ((now_ms - s) / ms_per_day).clamp(0, dt);
                        (Some(dt), Some(de))
                    } else {
                        (None, None)
                    }
                }
                _ => (None, None),
            };

            CycleStats {
                id: c.id.clone(),
                name: c.name.clone(),
                status: c.status.as_str().to_string(),
                total: c_total,
                completed: c_completed,
                in_progress: c_in_progress,
                pct_done,
                start_date: c.start_date.clone(),
                end_date: c.end_date.clone(),
                days_total,
                days_elapsed,
            }
        })
        .collect();

    ProjectStats {
        summary: Summary {
            total,
            active,
            in_progress,
            completed,
            unassigned,
            completion_rate,
        },
        by_status,
        by_priority,
        by_difficulty,
        by_label,
        members: member_stats,
        cycles: cycle_stats,
    }
}

/// Parse a "YYYY-MM-DD" string into epoch milliseconds.
fn parse_date_ms(date_str: &str) -> Result<i64, ()> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err(());
    }
    let y: i32 = parts[0].parse().map_err(|_| ())?;
    let m: u32 = parts[1].parse().map_err(|_| ())?;
    let d: u32 = parts[2].parse().map_err(|_| ())?;

    // Simple days-since-epoch calculation (good enough for day granularity)
    // Using a simplified algorithm
    let days = days_from_civil(y, m, d);
    Ok(days as i64 * 86_400_000)
}

/// Convert a civil date to days since Unix epoch.
/// Algorithm from Howard Hinnant.
fn days_from_civil(y: i32, m: u32, d: u32) -> i32 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe as i32 - 719468
}
