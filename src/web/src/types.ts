/** Types mirroring the server's JSON output from toJSON() */

export type Status = "none" | "todo" | "in_progress" | "completed" | "archived" | "wont_do";
export type Priority = "none" | "low" | "medium" | "high" | "urgent";
export type Label = "new_feature" | "bug" | "feature_plus";
export type MemberRole = "owner" | "member" | "agent";

export type Timestamp = number;

export interface Comment {
  id: string;
  author: string;
  authorName: string;
  text: string;
  createdAt: Timestamp;
}

export interface AuditEntry {
  id: string;
  action: string;
  actor: string;
  actorName: string;
  target: string;
  details: string;
  timestamp: Timestamp;
}

export interface Todo {
  id: string;
  ref: string;
  number: number;
  title: string;
  description: string;
  status: Status | null;
  priority: Priority;
  labels: Label[];
  assignee: string | null;
  assigneeName: string | null;
  branch: string | null;
  comments: Comment[];
  createdAt: Timestamp;
  updatedAt: Timestamp;
  createdBy: string;
}

export interface Member {
  id: string;
  name: string;
  email: string | null;
  role: MemberRole;
}

export interface Project {
  id: string;
  prefix: string;
  name: string;
  description: string;
  members: Member[];
  todos: Todo[];
  auditLog: AuditEntry[];
}

export const STATUSES: Status[] = [
  "none",
  "todo",
  "in_progress",
  "completed",
  "archived",
  "wont_do",
];

/** Statuses shown as columns on the board */
export const BOARD_STATUSES: Status[] = [
  "none",
  "todo",
  "in_progress",
  "completed",
  "archived",
  "wont_do",
];

export const PRIORITIES: Priority[] = ["none", "urgent", "high", "medium", "low"];

export const LABELS: Label[] = ["bug", "new_feature", "feature_plus"];

export const LABEL_DISPLAY: Record<Label, string> = {
  new_feature: "New Feature",
  bug: "Bug",
  feature_plus: "Feature++",
};

export const LABEL_COLORS: Record<Label, string> = {
  new_feature: "#10b981",
  bug: "#ef4444",
  feature_plus: "#8b5cf6",
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

export const PRIORITY_COLORS: Record<Priority, string> = {
  none: "#d4d4d8",
  low: "#6b7280",
  medium: "#3b82f6",
  high: "#f59e0b",
  urgent: "#ef4444",
};

export const STATUS_COLORS: Record<Status, string> = {
  none: "#9ca3af",
  todo: "#3b82f6",
  in_progress: "#f59e0b",
  completed: "#10b981",
  archived: "#6b7280",
  wont_do: "#ef4444",
};
