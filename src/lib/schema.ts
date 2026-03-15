/**
 * Core data types for the agent-todo-list CRDT document.
 * These interfaces describe the shape of the Automerge document.
 */

import type { Counter } from "@automerge/automerge";

// ── Branded primitives ──────────────────────────────────────────────

/** Member ID (UUID string) — references a member in the project */
export type MemberId = string;

/** Unix timestamp in milliseconds (Date.now()) */
export type Timestamp = number;

// ── Enums ───────────────────────────────────────────────────────────

export type Status =
  | "none"
  | "todo"
  | "in_progress"
  | "completed"
  | "archived"
  | "wont_do"
  | "needs_elaboration";
export type Priority = "none" | "low" | "medium" | "high" | "urgent";
export type Label = "new_feature" | "bug" | "feature_plus";
export type Difficulty = "none" | "easy" | "medium" | "hard";
export type Platform = "cli" | "web" | "unknown";
export type MemberRole = "owner" | "member" | "agent";
export type AgentProvider = "claude-code" | "opencode" | "custom";

export const AGENT_PROVIDERS: readonly AgentProvider[] = [
  "claude-code",
  "opencode",
  "custom",
] as const;

export const AGENT_PROVIDER_DISPLAY: Record<AgentProvider, string> = {
  "claude-code": "Claude Code",
  opencode: "Opencode",
  custom: "Custom",
};

export const STATUSES: readonly Status[] = [
  "none",
  "todo",
  "needs_elaboration",
  "in_progress",
  "completed",
  "archived",
  "wont_do",
] as const;

export const PRIORITIES: readonly Priority[] = ["none", "urgent", "high", "medium", "low"] as const;

export const DIFFICULTIES: readonly Difficulty[] = ["none", "easy", "medium", "hard"] as const;

export const LABELS: readonly Label[] = ["bug", "new_feature", "feature_plus"] as const;

export const LABEL_DISPLAY: Record<Label, string> = {
  new_feature: "New Feature",
  bug: "Bug",
  feature_plus: "Feature++",
};

export const STATUS_DISPLAY: Record<Status, string> = {
  none: "None",
  todo: "To Do",
  in_progress: "In Progress",
  completed: "Completed",
  archived: "Archived",
  wont_do: "Won't Do",
  needs_elaboration: "Needs Elaboration",
};

export const PRIORITY_DISPLAY: Record<Priority, string> = {
  none: "None",
  low: "Low",
  medium: "Medium",
  high: "High",
  urgent: "Urgent",
};

export const DIFFICULTY_DISPLAY: Record<Difficulty, string> = {
  none: "None",
  easy: "Low",
  medium: "Medium",
  hard: "High",
};

// ── Sub-document types ──────────────────────────────────────────────

export interface Comment {
  id: string;
  author: MemberId;
  authorName: string; // snapshot at creation time (survives member rename/delete)
  text: string;
  createdAt: Timestamp;
}

/**
 * AuditEntry is now reconstructed from Automerge's change history at read time
 * (see src/lib/history.ts). This interface defines the shape returned by
 * getAuditLog() — it is no longer stored on the document itself.
 */
export interface AuditEntry {
  action: string; // e.g. "todo.created", "todo.updated", "todo.deleted"
  actorId: MemberId;
  actorName: string; // snapshot at creation time (embedded in change message)
  target: string; // e.g. "TODO-1" or member name
  details: Record<string, unknown>; // structured details (no longer a JSON string)
  timestamp: Timestamp; // from Automerge change.time (seconds) * 1000
  hash: string; // Automerge change hash — unique identifier
}

// ── Core entities ───────────────────────────────────────────────────

export interface Todo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: Status;
  priority: Priority;
  difficulty: Difficulty;
  labels: Label[];
  assignee: MemberId | null;
  branch: string | null;
  comments: Comment[];
  createdAt: Timestamp;
  updatedAt: Timestamp;
  createdBy: MemberId;
  platform: Platform;
}

export interface Member {
  id: string;
  name: string;
  email: string | null;
  role: MemberRole;
  /** Only relevant when role === "agent" */
  agentProvider?: AgentProvider;
  /** Model identifier, e.g. "claude-sonnet-4-20250514" */
  agentModel?: string;
}

export interface Project extends Record<string, unknown> {
  _version: number;
  id: string;
  prefix: string;
  name: string;
  description: string;
  counter: Counter;
  createdAt: Timestamp;
  members: Member[];
  todos: Todo[];
}

/** Current schema version — increment when making breaking changes */
export const CURRENT_SCHEMA_VERSION = 5;

/** Config stored in .todo/config.toml (committed to git) */
export interface ProjectConfig {
  id: string;
  prefix: string;
  name: string;
}
