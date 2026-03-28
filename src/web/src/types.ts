/** Types mirroring the server's JSON output from toJSON() */

export type Status =
  | "none"
  | "todo"
  | "in_progress"
  | "paused"
  | "completed"
  | "archived"
  | "wont_do"
  | "needs_elaboration";
export type Priority = "none" | "low" | "medium" | "high" | "urgent";
export type Difficulty = "none" | "easy" | "medium" | "hard";
export type Label = "new_feature" | "bug" | "feature_plus";
export type Platform = "cli" | "web" | "unknown";
export type MemberRole = "owner" | "member" | "agent";
export type AgentProvider = "claude-code" | "opencode" | "custom";

export const AGENT_PROVIDERS: AgentProvider[] = ["claude-code", "opencode", "custom"];

export const AGENT_PROVIDER_DISPLAY: Record<AgentProvider, string> = {
  "claude-code": "Claude Code",
  opencode: "Opencode",
  custom: "Custom",
};

export type Timestamp = number;

export interface Comment {
  id: string;
  author: string;
  authorName: string;
  text: string;
  createdAt: Timestamp;
}

export interface AuditEntry {
  action: string;
  actorId: string;
  actorName: string;
  target: string;
  details: Record<string, unknown>;
  timestamp: Timestamp;
  hash: string;
}

export interface Todo {
  id: string;
  ref: string;
  number: number;
  title: string;
  description: string;
  status: Status | null;
  priority: Priority;
  difficulty: Difficulty;
  labels: Label[];
  assignee: string | null;
  assigneeName: string | null;
  branch: string | null;
  worktrees: string[];
  commits: string[];
  planPath: string | null;
  comments: Comment[];
  createdAt: Timestamp;
  updatedAt: Timestamp;
  createdBy: string;
  platform: Platform;
}

export interface Member {
  id: string;
  name: string;
  email: string | null;
  role: MemberRole;
  agentProvider?: AgentProvider;
  agentModel?: string;
}

export interface Project {
  id: string;
  prefix: string;
  name: string;
  description: string;
  members: Member[];
  todos: Todo[];
  auditLog: AuditEntry[];
  /** Inbox text from .todo/TODO.md */
  inboxText?: string;
  /** Processed inbox archive from .todo/TODO-PROCESSED.md */
  inboxProcessed?: string;
}

export const STATUSES: Status[] = [
  "none",
  "todo",
  "needs_elaboration",
  "in_progress",
  "paused",
  "completed",
  "archived",
  "wont_do",
];

/** Statuses shown as columns on the board */
export const BOARD_STATUSES: Status[] = [
  "none",
  "todo",
  "needs_elaboration",
  "in_progress",
  "paused",
  "completed",
  "archived",
  "wont_do",
];

export const PRIORITIES: Priority[] = ["none", "urgent", "high", "medium", "low"];

export const DIFFICULTIES: Difficulty[] = ["none", "easy", "medium", "hard"];

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
  paused: "Paused",
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

export const PRIORITY_COLORS: Record<Priority, string> = {
  none: "#d4d4d8",
  low: "#6b7280",
  medium: "#3b82f6",
  high: "#f59e0b",
  urgent: "#ef4444",
};

export const DIFFICULTY_COLORS: Record<Difficulty, string> = {
  none: "#d4d4d8",
  easy: "#10b981",
  medium: "#f59e0b",
  hard: "#ef4444",
};

export const STATUS_COLORS: Record<Status, string> = {
  none: "#9ca3af",
  todo: "#3b82f6",
  in_progress: "#f59e0b",
  paused: "#f97316",
  completed: "#10b981",
  archived: "#6b7280",
  wont_do: "#ef4444",
  needs_elaboration: "#a855f7",
};
