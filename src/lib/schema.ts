/**
 * Core data types for the agent-todo-list CRDT document.
 * These interfaces describe the shape of the Automerge document.
 */

import type { Counter } from "@automerge/automerge";

export type Status = "backlog" | "todo" | "in_progress" | "done" | "archived";
export type Priority = "low" | "medium" | "high" | "urgent";
export type MemberRole = "owner" | "member" | "agent";

export const STATUSES: readonly Status[] = [
  "backlog",
  "todo",
  "in_progress",
  "done",
  "archived",
] as const;

export const PRIORITIES: readonly Priority[] = [
  "low",
  "medium",
  "high",
  "urgent",
] as const;

export const STATUS_DISPLAY: Record<Status, string> = {
  backlog: "Backlog",
  todo: "Todo",
  in_progress: "In Progress",
  done: "Done",
  archived: "Archived",
};

export const PRIORITY_DISPLAY: Record<Priority, string> = {
  low: "Low",
  medium: "Medium",
  high: "High",
  urgent: "Urgent",
};

export interface Todo {
  id: string;
  number: number;
  title: string;
  description: string;
  status: Status;
  priority: Priority;
  assignee: string | null;
  tags: string[];
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
