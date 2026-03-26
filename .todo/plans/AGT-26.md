# AGT-26: Member section should be management of AI agents

## Research

### Current State of the Member System

The member system already has infrastructure for agents but the pieces aren't connected:

**Schema** (`src/rust/crates/agt-lib/src/schema.rs`): The `Member` struct includes `role: MemberRole` (Owner/Member/Agent), `agent_provider: Option<AgentProvider>` (ClaudeCode/Opencode/Custom), and `agent_model: Option<String>`. These fields are stored in Automerge but **not used at runtime**.

**CLI** (`src/rust/crates/agt-cli/src/commands/member.rs`): `agt member add/list/remove/update` supports `--role agent --provider claude-code --model <model>`. Agents can be assigned to todos via `agt assign`. Full CRUD works.

**Web Dashboard** (`src/web/src/views/MembersView.vue`): The view **explicitly filters out agents** (`members.filter(m => m.role !== "agent")`). There are unused CSS classes for an agent section (`.agent-section`, `.agent-card`, etc.) suggesting this was planned but never built. No UI exists to add, edit, or view agent members.

**API** (`src/rust/crates/agt-server/src/routes.rs`): The `addMember`, `removeMember`, `updateMember` actions already accept `agentProvider` and `agentModel` fields. The backend is ready.

**Run System** (`src/rust/crates/agt-cli/src/commands/run.rs`): `agt run` spawns a subprocess using `agent.command` from workflow config (defaults to `"claude"`). It does **not** look at the assigned member's `agent_provider` or `agent_model`. There's no connection between "which agent member is assigned" and "which agent binary runs."

**Workflow Config** (`.todo/workflow.md` YAML frontmatter): Project-level settings for `agent.command`, `agent.max_concurrent`, `agent.budget_usd`, `agent.allowed_tools`. These are global — not per-agent-member.

### Key Gaps

1. Web UI has no agent management — agents are invisible in the dashboard
2. Agent provider/model stored on members are never used when spawning agent runs
3. No per-agent configuration (command, budget, tools) — everything is project-global
4. No agent status/health monitoring in the UI

## Approach Options

### Option 1: UI-Only — Add Agent Management to Web Dashboard

Add an "Agents" section to MembersView (or a separate view) that displays, creates, edits, and deletes agent members. Keep the existing schema and API unchanged.

**Scope:**
- Split MembersView into tabs or sections: "People" and "Agents"
- Agent cards showing name, provider, model, assigned task count
- Add/Edit modal with agent-specific fields (provider dropdown, model input)
- Remove the filter that hides agents

**Pros:** Minimal backend changes, uses existing API, fast to ship
**Cons:** Agents are still just labels — assigning an agent doesn't change what runs

### Option 2: UI + Wire Agent Config into Run System

Option 1 plus: when `agt run` spawns an agent, look up the assigned member's `agent_provider` and `agent_model` to determine the command and model flag.

**Scope:**
- Everything in Option 1
- Extend `Member` schema with per-agent fields: `agent_command` (override), `agent_budget_usd`, `agent_allowed_tools`
- Modify `run.rs` to resolve the assigned member, check if role=Agent, and use their provider/model/command instead of (or merged with) workflow defaults
- Schema migration to v5 for new fields

**Pros:** Assigning an agent actually means something — different agents can use different providers/models
**Cons:** More complex, schema migration needed, must handle "no agent assigned" fallback

### Option 3: Full Agent Management Platform

Option 2 plus: agent status tracking, run history per agent, health monitoring, and the ability to trigger runs from the web UI.

**Scope:**
- Everything in Option 2
- Agent status indicators (idle, running, errored) derived from `.todo/runs/*.json`
- Per-agent run history view
- "Run" button on todos assigned to agents (calls `agt run` via API)
- Agent activity dashboard (tasks completed, cost spent, etc.)

**Pros:** Complete agent management experience, makes the web dashboard genuinely useful for agent orchestration
**Cons:** Large scope, needs new API endpoints, significant frontend work

## Recommendation

**Option 2** — UI + Wire Agent Config into Run System.

This is the sweet spot. Option 1 is cosmetic — it doesn't make agents functional. Option 3 is ambitious and can be done incrementally after Option 2 ships. Option 2 makes the "Agents" section meaningful: you add an agent with its provider and model, assign it to a todo, and `agt run` uses that configuration.

**Implementation order:**
1. **Web UI: Agent management section** — Split MembersView, add agent CRUD with provider/model fields. The API already supports this.
2. **Schema: Per-agent config fields** — Add `agent_command`, `agent_budget_usd` to Member. Migrate to schema v5.
3. **CLI: Wire run to agent member** — When `agt run PREFIX-N` executes, resolve the assigned member. If it's an agent, use its provider to determine the command (`claude-code` → `claude`, `opencode` → `opencode`, `custom` → `agent_command` field). Pass `agent_model` if set.
4. **Fallback behavior** — If no agent is assigned or assignee isn't an agent, fall back to workflow config defaults (current behavior).

## Questions

- Should "Agents" be a separate nav item/view, or a tab within the existing Members page?
- Should per-agent budget/tools override or merge with the project-level workflow config?
- Are there additional agent providers beyond claude-code, opencode, and custom that should be supported?
- Should the web UI support triggering `agt run` for agent-assigned todos, or is that CLI-only for now?
- Is there a desired UX for what happens when you assign a todo to an agent — should it auto-run, or just mark it as "ready for agent"?

## Answers