/**
 * Core data types for the agent-todo-list CRDT document.
 * These interfaces describe the shape of the Automerge document.
 */

import type { Counter } from "@automerge/automerge";

export type Status = "none" | "todo" | "in_progress" | "completed" | "archived" | "wont_do";
export type Priority = "low" | "medium" | "high" | "urgent";
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
  "low",
  "medium",
  "high",
  "urgent",
] as const;

export const STATUS_DISPLAY: Record<Status, string> = {
  none: "None",
  todo: "To Do",
  in_progress: "In Progress",
  completed: "Completed",
  archived: "Archived",
  wont_do: "Won't Do",
};

export const PRIORITY_DISPLAY: Record<Priority, string> = {
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
