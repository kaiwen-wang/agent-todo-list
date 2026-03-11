/** Types mirroring the server's JSON output from toJSON() */

export type Status = 'backlog' | 'todo' | 'in_progress' | 'done' | 'archived'
export type Priority = 'low' | 'medium' | 'high' | 'urgent'
export type MemberRole = 'owner' | 'member' | 'agent'

export interface Todo {
  id: string
  ref: string
  number: number
  title: string
  description: string
  status: Status
  priority: Priority
  assignee: string | null
  assigneeName: string | null
  tags: string[]
  createdAt: string
  updatedAt: string
  createdBy: string
}

export interface Member {
  id: string
  name: string
  email: string | null
  role: MemberRole
}

export interface Project {
  id: string
  prefix: string
  name: string
  description: string
  members: Member[]
  todos: Todo[]
}

export const STATUSES: Status[] = ['backlog', 'todo', 'in_progress', 'done', 'archived']

/** Statuses shown as columns on the board (exclude archived) */
export const BOARD_STATUSES: Status[] = ['backlog', 'todo', 'in_progress', 'done']

export const PRIORITIES: Priority[] = ['low', 'medium', 'high', 'urgent']

export const STATUS_DISPLAY: Record<Status, string> = {
  backlog: 'Backlog',
  todo: 'Todo',
  in_progress: 'In Progress',
  done: 'Done',
  archived: 'Archived',
}

export const PRIORITY_DISPLAY: Record<Priority, string> = {
  low: 'Low',
  medium: 'Medium',
  high: 'High',
  urgent: 'Urgent',
}

export const PRIORITY_COLORS: Record<Priority, string> = {
  low: '#6b7280',
  medium: '#3b82f6',
  high: '#f59e0b',
  urgent: '#ef4444',
}

export const STATUS_COLORS: Record<Status, string> = {
  backlog: '#9ca3af',
  todo: '#3b82f6',
  in_progress: '#f59e0b',
  done: '#10b981',
  archived: '#6b7280',
}
