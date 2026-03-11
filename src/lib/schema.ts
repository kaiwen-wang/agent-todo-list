/**
 * Core data types for the agent-todo-list CRDT document.
 * These interfaces describe the shape of the Automerge document.
 */

import type { Counter } from "@automerge/automerge";

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

export interface Todo {
  id: string;
  ref: string;
  number: number;
  title: string;
  description: string;
  status: Status;
  priority: Priority;
  labels: Label[];
  assignee: string | null;
  assigneeName: string | null;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
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
  createdAt: string;
  members: Member[];
  todos: Todo[];
}

/** Current schema version — increment when making breaking changes */
export const CURRENT_SCHEMA_VERSION = 1;

/** Config stored in .todo/config.toml (committed to git) */
export interface ProjectConfig {
  id: string;
  prefix: string;
  name: string;
}
