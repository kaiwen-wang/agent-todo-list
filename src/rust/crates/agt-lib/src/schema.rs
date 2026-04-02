//! Core data types for the agent-todo-list CRDT document.
//! These types define the shape of the Automerge document.

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Enums ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    None,
    Todo,
    InProgress,
    Paused,
    Completed,
    Archived,
    WontDo,
    NeedsElaboration,
}

impl Status {
    pub const ALL: &[Status] = &[
        Status::None,
        Status::Todo,
        Status::NeedsElaboration,
        Status::InProgress,
        Status::Paused,
        Status::Completed,
        Status::Archived,
        Status::WontDo,
    ];

    /// Rank for sorting: higher number = more actionable.
    /// Used by `--rank` to put the most actionable items at the bottom (visible in terminal).
    pub fn rank(&self) -> u8 {
        match self {
            Status::Completed => 0,
            Status::None => 1,
            Status::NeedsElaboration => 2,
            Status::Todo => 3,
            Status::Paused => 4,
            Status::InProgress => 5,
            // Archived/WontDo are typically filtered out, but rank low if present
            Status::Archived => 0,
            Status::WontDo => 0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Status::None => "None",
            Status::Todo => "To Do",
            Status::InProgress => "In Progress",
            Status::Paused => "Paused",
            Status::Completed => "Completed",
            Status::Archived => "Archived",
            Status::WontDo => "Won't Do",
            Status::NeedsElaboration => "Needs Elaboration",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Status::None => "none",
            Status::Todo => "todo",
            Status::InProgress => "in_progress",
            Status::Paused => "paused",
            Status::Completed => "completed",
            Status::Archived => "archived",
            Status::WontDo => "wont_do",
            Status::NeedsElaboration => "needs_elaboration",
        }
    }
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Status::None),
            "todo" => Ok(Status::Todo),
            "in_progress" => Ok(Status::InProgress),
            "paused" => Ok(Status::Paused),
            "queued" => Ok(Status::Todo), // backwards compat: queued -> todo
            "completed" | "done" => Ok(Status::Completed),
            "archived" => Ok(Status::Archived),
            "wont_do" => Ok(Status::WontDo),
            "needs_elaboration" => Ok(Status::NeedsElaboration),
            _ => Err(format!("unknown status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    #[default]
    None,
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub const ALL: &[Priority] = &[
        Priority::None,
        Priority::Urgent,
        Priority::High,
        Priority::Medium,
        Priority::Low,
    ];

    /// Rank for sorting: higher number = higher priority.
    pub fn rank(&self) -> u8 {
        match self {
            Priority::None => 0,
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
            Priority::Urgent => 4,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Priority::None => "None",
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Urgent => "Urgent",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::None => "none",
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Urgent => "urgent",
        }
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Priority {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Priority::None),
            "low" => Ok(Priority::Low),
            "medium" | "med" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "urgent" => Ok(Priority::Urgent),
            _ => Err(format!("unknown priority: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    #[default]
    None,
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub const ALL: &[Difficulty] = &[
        Difficulty::None,
        Difficulty::Easy,
        Difficulty::Medium,
        Difficulty::Hard,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::None => "None",
            Difficulty::Easy => "Low",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "High",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::None => "none",
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Difficulty {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Difficulty::None),
            "easy" => Ok(Difficulty::Easy),
            "medium" | "med" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            _ => Err(format!("unknown difficulty: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    NewFeature,
    Bug,
    FeaturePlus,
}

impl Label {
    pub const ALL: &[Label] = &[Label::Bug, Label::NewFeature, Label::FeaturePlus];

    pub fn display_name(&self) -> &'static str {
        match self {
            Label::NewFeature => "New Feature",
            Label::Bug => "Bug",
            Label::FeaturePlus => "Feature++",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Label::NewFeature => "new_feature",
            Label::Bug => "bug",
            Label::FeaturePlus => "feature_plus",
        }
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Label {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "new_feature" => Ok(Label::NewFeature),
            "bug" => Ok(Label::Bug),
            "feature_plus" => Ok(Label::FeaturePlus),
            _ => Err(format!("unknown label: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Cli,
    Web,
    #[default]
    Unknown,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Cli => "cli",
            Platform::Web => "web",
            Platform::Unknown => "unknown",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Platform {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cli" => Ok(Platform::Cli),
            "web" => Ok(Platform::Web),
            "unknown" => Ok(Platform::Unknown),
            _ => Err(format!("unknown platform: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CycleStatus {
    #[default]
    Planning,
    Active,
    Completed,
    Cancelled,
}

impl CycleStatus {
    pub const ALL: &[CycleStatus] = &[
        CycleStatus::Planning,
        CycleStatus::Active,
        CycleStatus::Completed,
        CycleStatus::Cancelled,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            CycleStatus::Planning => "Planning",
            CycleStatus::Active => "Active",
            CycleStatus::Completed => "Completed",
            CycleStatus::Cancelled => "Cancelled",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CycleStatus::Planning => "planning",
            CycleStatus::Active => "active",
            CycleStatus::Completed => "completed",
            CycleStatus::Cancelled => "cancelled",
        }
    }
}

impl fmt::Display for CycleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for CycleStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "planning" => Ok(CycleStatus::Planning),
            "active" => Ok(CycleStatus::Active),
            "completed" => Ok(CycleStatus::Completed),
            "cancelled" | "canceled" => Ok(CycleStatus::Cancelled),
            _ => Err(format!("unknown cycle status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberRole {
    Owner,
    Member,
    Agent,
}

impl MemberRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemberRole::Owner => "owner",
            MemberRole::Member => "member",
            MemberRole::Agent => "agent",
        }
    }
}

impl fmt::Display for MemberRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for MemberRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(MemberRole::Owner),
            "member" => Ok(MemberRole::Member),
            "agent" => Ok(MemberRole::Agent),
            _ => Err(format!("unknown role: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentProvider {
    ClaudeCode,
    Opencode,
    Custom,
}

impl AgentProvider {
    pub const ALL: &[AgentProvider] = &[
        AgentProvider::ClaudeCode,
        AgentProvider::Opencode,
        AgentProvider::Custom,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentProvider::ClaudeCode => "Claude Code",
            AgentProvider::Opencode => "Opencode",
            AgentProvider::Custom => "Custom",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentProvider::ClaudeCode => "claude-code",
            AgentProvider::Opencode => "opencode",
            AgentProvider::Custom => "custom",
        }
    }
}

impl fmt::Display for AgentProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for AgentProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "claude-code" => Ok(AgentProvider::ClaudeCode),
            "opencode" => Ok(AgentProvider::Opencode),
            "custom" => Ok(AgentProvider::Custom),
            _ => Err(format!("unknown agent provider: {s}")),
        }
    }
}

// ── Sub-document types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub author_name: String,
    pub text: String,
    pub created_at: i64,
    /// If set, this comment is a reply to the comment with this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// Audit entry reconstructed from Automerge change history at read time.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    pub action: String,
    pub actor_id: String,
    pub actor_name: String,
    pub target: String,
    pub details: serde_json::Value,
    pub timestamp: i64,
    pub hash: String,
}

// ── Core entities ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub description: String,
    pub status: Status,
    pub priority: Priority,
    pub difficulty: Difficulty,
    pub labels: Vec<Label>,
    pub assignee: Option<String>,
    pub branch: Option<String>,
    pub worktrees: Vec<String>,
    pub commits: Vec<String>,
    pub comments: Vec<Comment>,
    pub created_at: i64,
    pub updated_at: i64,
    pub created_by: String,
    pub platform: Platform,
    pub plan_path: Option<String>,
    pub cycle_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cycle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: CycleStatus,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub role: MemberRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_provider: Option<AgentProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_model: Option<String>,
}

/// Current schema version — increment when making breaking changes.
pub const CURRENT_SCHEMA_VERSION: u64 = 10;

/// Config stored in .todo/config.toml (committed to git).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub prefix: String,
    pub name: String,
}
