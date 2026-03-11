/** Types mirroring the server's JSON output from toJSON() */

export type Status = 'none' | 'todo' | 'in_progress' | 'completed' | 'archived' | 'wont_do'
export type Priority = 'low' | 'medium' | 'high' | 'urgent'
export type MemberRole = 'owner' | 'member' | 'agent'

export interface Todo {
  id: string
  ref: string
  number: number
  title: string
  description: string
  status: Status | null
  priority: Priority | null
  assignee: string | null
  assigneeName: string | null
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

export const STATUSES: Status[] = ['none', 'todo', 'in_progress', 'completed', 'archived', 'wont_do']

/** Statuses shown as columns on the board */
export const BOARD_STATUSES: Status[] = ['none', 'todo', 'in_progress', 'completed', 'archived', 'wont_do']

export const PRIORITIES: Priority[] = ['urgent', 'high', 'medium', 'low']

export const STATUS_DISPLAY: Record<Status, string> = {
  none: 'None',
  todo: 'To Do',
  in_progress: 'In Progress',
  completed: 'Completed',
  archived: 'Archived',
  wont_do: "Won't Do",
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
  none: '#9ca3af',
  todo: '#3b82f6',
  in_progress: '#f59e0b',
  completed: '#10b981',
  archived: '#6b7280',
  wont_do: '#ef4444',
}
