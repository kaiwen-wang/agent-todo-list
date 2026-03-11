/** API client — talks to the Bun server's two endpoints */

import type { Project, Status, Priority } from './types'

const BASE = '' // same origin (Vite proxy in dev, Bun.serve in prod)

export async function fetchProject(): Promise<Project> {
  const res = await fetch(`${BASE}/api/project`)
  if (!res.ok) throw new Error(`Failed to fetch project: ${res.statusText}`)
  return res.json()
}

export interface AddTodoParams {
  title: string
  description?: string
  status?: Status
  priority?: Priority
  tags?: string[]
}

export async function addTodo(params: AddTodoParams): Promise<{ ok: boolean; number: number }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ action: 'add', ...params }),
  })
  if (!res.ok) {
    const data = await res.json()
    throw new Error(data.error || 'Failed to add todo')
  }
  return res.json()
}

export interface UpdateTodoParams {
  title?: string
  description?: string
  status?: Status
  priority?: Priority
  assignee?: string | null
  tags?: string[]
}

export async function updateTodo(
  number: number,
  updates: UpdateTodoParams,
): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ action: 'update', number, updates }),
  })
  if (!res.ok) {
    const data = await res.json()
    throw new Error(data.error || 'Failed to update todo')
  }
  return res.json()
}

export async function deleteTodo(number: number): Promise<{ ok: boolean }> {
  const res = await fetch(`${BASE}/api/change`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ action: 'delete', number }),
  })
  if (!res.ok) {
    const data = await res.json()
    throw new Error(data.error || 'Failed to delete todo')
  }
  return res.json()
}
