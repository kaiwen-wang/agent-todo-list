//! Parse .todo/workflow.md — YAML front matter + prompt template.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Default allowed tools for the agent (safe restricted set).
pub const DEFAULT_ALLOWED_TOOLS: &[&str] = &[
    "Read",
    "Edit",
    "Write",
    "Glob",
    "Grep",
    "Bash(git:*)",
    "Bash(bun:*)",
    "Bash(agt:*)",
];

const DEFAULT_PROMPT: &str = r#"You are working on a task.

## Task: {{ todo_ref }} — {{ todo_title }}
Priority: {{ todo_priority }}
Difficulty: {{ todo_difficulty }}

{% if todo_description %}
## Description
{{ todo_description }}
{% endif %}

{% if todo_comments %}
## Comments
{{ todo_comments }}
{% endif %}

Work in the current directory. When finished, run:
  agt update {{ todo_ref }} --status completed
"#;

// ── Config types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct AgentConfig {
    pub command: Option<String>,
    pub max_concurrent: Option<usize>,
    pub budget_usd: Option<f64>,
    pub allowed_tools: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrontMatter {
    pub agent: Option<AgentConfig>,
}

/// Resolved workflow configuration with defaults applied.
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub agent_command: String,
    pub max_concurrent: usize,
    pub budget_usd: f64,
    pub allowed_tools: Vec<String>,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            agent_command: "claude".to_string(),
            max_concurrent: 3,
            budget_usd: 5.00,
            allowed_tools: DEFAULT_ALLOWED_TOOLS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Parsed workflow: config + prompt template.
#[derive(Debug, Clone)]
pub struct Workflow {
    pub config: WorkflowConfig,
    pub prompt_template: String,
}

// ── Parsing ─────────────────────────────────────────────────────────

/// Load and parse .todo/workflow.md. Returns defaults if file doesn't exist.
pub fn load_workflow(todo_dir: &Path) -> Result<Workflow> {
    let workflow_path = todo_dir.join("workflow.md");

    if !workflow_path.exists() {
        return Ok(Workflow {
            config: WorkflowConfig::default(),
            prompt_template: DEFAULT_PROMPT.to_string(),
        });
    }

    let content = std::fs::read_to_string(&workflow_path)
        .with_context(|| format!("Failed to read {}", workflow_path.display()))?;

    parse_workflow(&content)
}

/// Parse workflow content string into config + template.
pub fn parse_workflow(content: &str) -> Result<Workflow> {
    let (front_matter_str, prompt_body) = split_front_matter(content);

    let config = if let Some(yaml_str) = front_matter_str {
        let fm: FrontMatter =
            serde_yaml::from_str(&yaml_str).context("Invalid YAML front matter in workflow.md")?;
        resolve_config(fm)
    } else {
        WorkflowConfig::default()
    };

    let prompt_template = if prompt_body.trim().is_empty() {
        DEFAULT_PROMPT.to_string()
    } else {
        prompt_body.trim().to_string()
    };

    Ok(Workflow {
        config,
        prompt_template,
    })
}

/// Split content into optional YAML front matter and markdown body.
fn split_front_matter(content: &str) -> (Option<String>, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content.to_string());
    }

    // Find the closing ---
    let after_first = &trimmed[3..];
    if let Some(end_idx) = after_first.find("\n---") {
        let yaml = after_first[..end_idx].trim().to_string();
        let rest_start = end_idx + 4; // skip past "\n---"
        let body = if rest_start < after_first.len() {
            after_first[rest_start..].to_string()
        } else {
            String::new()
        };
        (Some(yaml), body)
    } else {
        // No closing ---, treat entire content as body
        (None, content.to_string())
    }
}

/// Apply defaults to parsed front matter.
fn resolve_config(fm: FrontMatter) -> WorkflowConfig {
    let defaults = WorkflowConfig::default();

    let agent = fm.agent.unwrap_or(AgentConfig {
        command: None,
        max_concurrent: None,
        budget_usd: None,
        allowed_tools: None,
    });

    WorkflowConfig {
        agent_command: agent.command.unwrap_or(defaults.agent_command),
        max_concurrent: agent.max_concurrent.unwrap_or(defaults.max_concurrent),
        budget_usd: agent.budget_usd.unwrap_or(defaults.budget_usd),
        allowed_tools: agent.allowed_tools.unwrap_or(defaults.allowed_tools),
    }
}

// ── Prompt rendering ────────────────────────────────────────────────

/// Context for rendering the prompt template.
pub struct PromptContext<'a> {
    pub project_name: &'a str,
    pub project_prefix: &'a str,
    pub todo_ref: String,
    pub todo_title: &'a str,
    pub todo_description: &'a str,
    pub todo_priority: &'a str,
    pub todo_difficulty: &'a str,
    pub todo_labels: &'a [String],
    pub todo_comments: Vec<CommentContext<'a>>,
    pub todo_assignee: Option<&'a str>,
    pub attempt: Option<u32>,
}

pub struct CommentContext<'a> {
    pub author: &'a str,
    pub text: &'a str,
}

/// Render the prompt template with todo context.
pub fn render_prompt(template: &str, ctx: &PromptContext) -> Result<String> {
    let mut env = minijinja::Environment::new();
    env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);
    env.add_template("prompt", template)
        .context("Failed to parse prompt template")?;

    let tmpl = env.get_template("prompt").unwrap();

    // Build comments string
    let comments_str = if ctx.todo_comments.is_empty() {
        String::new()
    } else {
        ctx.todo_comments
            .iter()
            .map(|c| format!("**{}**: {}", c.author, c.text))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let labels_str = ctx.todo_labels.join(", ");

    let rendered = tmpl
        .render(minijinja::context! {
            project_name => ctx.project_name,
            project_prefix => ctx.project_prefix,
            todo_ref => &ctx.todo_ref,
            todo_title => ctx.todo_title,
            todo_description => ctx.todo_description,
            todo_priority => ctx.todo_priority,
            todo_difficulty => ctx.todo_difficulty,
            todo_labels => labels_str,
            todo_comments => comments_str,
            todo_assignee => ctx.todo_assignee.unwrap_or("unassigned"),
            attempt => ctx.attempt,
        })
        .context("Failed to render prompt template")?;

    Ok(rendered)
}
