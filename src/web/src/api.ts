/** API client — talks to the Bun server's two endpoints */

import type { Project, Status, Priority, Label, MemberRole } from "./types";

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
  labels?: Label[];
  assignee?: string | null;
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

export async function triggerBrainProcess(): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: "processInbox" }),
  });
  if (!res.ok) {
    const data = await res.json();
    throw new Error(data.error || "Failed to start brain processing");
  }
  return res.json();
}
