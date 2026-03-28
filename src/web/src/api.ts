/** API client — talks to the Bun server's two endpoints */

import type {
  Project,
  Status,
  Priority,
  Difficulty,
  Label,
  MemberRole,
  CycleStatus,
} from "./types";

const BASE = ""; // same origin (Vite proxy in dev, Bun.serve in prod)

export async function fetchProject(): Promise<Project> {
  const res = await fetch(`${BASE}/api/project`);
  if (!res.ok) throw new Error(`Failed to fetch project: ${res.statusText}`);
  return res.json();
}

export interface AddTodoParams {
  title: string;
  description?: string;
  status?: Status;
  priority?: Priority;
  difficulty?: Difficulty;
  labels?: Label[];
  assignee?: string | null;
}

export async function addTodo(params: AddTodoParams): Promise<{ ok: boolean; number: number }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "add", ...params }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to add todo");
  }
  return res.json();
}

export interface UpdateTodoParams {
  title?: string;
  description?: string;
  status?: Status;
  priority?: Priority;
  difficulty?: Difficulty;
  labels?: Label[];
  assignee?: string | null;
  cycleId?: string | null;
}

export async function updateTodo(
  number: number,
  updates: UpdateTodoParams,
): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "update", number, updates }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to update todo");
  }
  return res.json();
}

export interface UpdateProjectParams {
  name?: string;
  prefix?: string;
  description?: string;
}

export async function updateProjectSettings(
  updates: UpdateProjectParams,
): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "updateProject", updates }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to update project");
  }
  return res.json();
}

export async function deleteTodo(number: number): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "delete", number }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to delete todo");
  }
  return res.json();
}

// ── Member API ──

export interface AddMemberParams {
  name: string;
  role?: MemberRole;
  email?: string;
  agentProvider?: string;
  agentModel?: string;
}

export async function addMember(params: AddMemberParams): Promise<{ ok: boolean; id: string }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "addMember", ...params }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to add member");
  }
  return res.json();
}

export async function removeMember(memberId: string): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "removeMember", memberId }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to remove member");
  }
  return res.json();
}

export interface UpdateMemberParams {
  name?: string;
  role?: MemberRole;
  email?: string | null;
  agentProvider?: string;
  agentModel?: string;
}

export async function addCommentApi(number: number, text: string): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "addComment", number, text }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to add comment");
  }
  return res.json();
}

export async function createBranchOnlyApi(
  number: number,
): Promise<{ ok: boolean; branch: string; alreadyExists?: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "createBranchOnly", number }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to create branch");
  }
  return res.json();
}

export async function createBranchApi(
  number: number,
): Promise<{ ok: boolean; branch: string; worktree?: string; alreadyExists?: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "createBranch", number }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to create branch");
  }
  return res.json();
}

export async function linkCommitApi(number: number, commit: string): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "linkCommit", number, commit }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to link commit");
  }
  return res.json();
}

export async function removeBranchApi(number: number): Promise<{ ok: boolean; branch: string }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "removeBranch", number }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to remove branch");
  }
  return res.json();
}

export async function updateMemberApi(
  memberId: string,
  updates: UpdateMemberParams,
): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "updateMember", memberId, updates }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to update member");
  }
  return res.json();
}

// ── Bulk API ──

export type BulkOperation =
  | { action: "update"; number: number; updates: UpdateTodoParams }
  | { action: "delete"; number: number };

export async function bulkChange(
  operations: BulkOperation[],
): Promise<{ ok: boolean; count: number }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "bulk", operations }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Bulk operation failed");
  }
  return res.json();
}

// ── Plan API ──

export async function fetchPlan(
  number: number,
): Promise<{ content: string | null; exists: boolean }> {
  const res = await fetch(`${BASE}/api/plan/${number}`);
  if (!res.ok) throw new Error(`Failed to fetch plan: ${res.statusText}`);
  return res.json();
}

export async function initPlan(
  number: number,
): Promise<{ ok: boolean; planPath: string; content: string }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "initPlan", number }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to init plan");
  }
  return res.json();
}

export async function researchPlan(
  number: number,
): Promise<{ ok: boolean; planPath: string; researching: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "researchPlan", number }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to start plan research");
  }
  return res.json();
}

export async function answerPlan(number: number, text: string): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "answerPlan", number, text }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to answer plan");
  }
  return res.json();
}

// ── Inbox API ──

export async function updateInbox(text: string): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "updateInbox", text }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to update inbox");
  }
  return res.json();
}

// ── Cycle API ──

export interface AddCycleParams {
  name: string;
  description?: string;
  status?: CycleStatus;
  startDate?: string;
  endDate?: string;
}

export async function addCycle(params: AddCycleParams): Promise<{ ok: boolean; id: string }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "addCycle", ...params }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to add cycle");
  }
  return res.json();
}

export interface UpdateCycleParams {
  name?: string;
  description?: string;
  status?: CycleStatus;
  startDate?: string | null;
  endDate?: string | null;
}

export async function updateCycle(
  cycleId: string,
  updates: UpdateCycleParams,
): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "updateCycle", cycleId, updates }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to update cycle");
  }
  return res.json();
}

export async function deleteCycle(cycleId: string): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "deleteCycle", cycleId }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to delete cycle");
  }
  return res.json();
}

export async function processInbox(): Promise<{
  ok: boolean;
  processed: number;
  tasks: Array<{ ref: string; title: string }>;
}> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "processInbox" }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to process inbox");
  }
  return res.json();
}
