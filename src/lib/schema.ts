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

export type Status = "none" | "todo" | "in_progress" | "completed" | "archived" | "wont_do";
export type Priority = "none" | "low" | "medium" | "high" | "urgent";
export type Label = "new_feature" | "bug" | "feature_plus";
export type MemberRole = "owner" | "member" | "agent";

export const STATUSES: readonly Status[] = [
  "none",
  "todo",
  "in_progress",
  "completed",
  "archived",
  "wont_do",
] as const;

export const PRIORITIES: readonly Priority[] = [
  "none",
  "urgent",
  "high",
  "medium",
  "low",
] as const;

export const LABELS: readonly Label[] = [
  "bug",
  "new_feature",
  "feature_plus",
] as const;

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
};

export const PRIORITY_DISPLAY: Record<Priority, string> = {
  none: "None",
  low: "Low",
  medium: "Medium",
  high: "High",
  urgent: "Urgent",
};

// ── Sub-document types ──────────────────────────────────────────────

export interface Comment {
  id: string;
  author: MemberId;
  authorName: string;     // snapshot at creation time (survives member rename/delete)
  text: string;
  createdAt: Timestamp;
}

export interface AuditEntry {
  id: string;
  action: string;         // e.g. "todo.created", "todo.updated", "todo.deleted"
  actor: MemberId;
  actorName: string;      // snapshot at creation time
  target: string;         // e.g. "TODO-1" or member name
  details: string;        // JSON string describing what changed
  timestamp: Timestamp;
}

// ── Core entities ───────────────────────────────────────────────────

export interface Todo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: Status;
  priority: Priority;
  labels: Label[];
  assignee: MemberId | null;
  branch: string | null;
  comments: Comment[];
  createdAt: Timestamp;
  updatedAt: Timestamp;
  createdBy: MemberId;
}

export interface Member {
  id: string;
  name: string;
  email: string | null;
  role: MemberRole;
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
  auditLog: AuditEntry[];
}

/** Current schema version — increment when making breaking changes */
export const CURRENT_SCHEMA_VERSION = 2;

/** Config stored in .todo/config.toml (committed to git) */
export interface ProjectConfig {
  id: string;
  prefix: string;
  name: string;
}
