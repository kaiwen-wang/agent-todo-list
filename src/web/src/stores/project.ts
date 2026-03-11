/** Pinia store — single source of truth for the project state in the frontend */

import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import type { Project, Todo, Status, Priority } from '@/types'
import * as api from '@/api'
import { BOARD_STATUSES } from '@/types'

export const useProjectStore = defineStore('project', () => {
  const project = ref<Project | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const todos = computed(() => project.value?.todos ?? [])
  const members = computed(() => project.value?.members ?? [])
  const projectName = computed(() => project.value?.name ?? '')
  const prefix = computed(() => project.value?.prefix ?? '')

  /** Todos grouped by status for the board view */
  const todosByStatus = computed(() => {
    const grouped: Record<Status, Todo[]> = {
      backlog: [],
      todo: [],
      in_progress: [],
      done: [],
      archived: [],
    }
    for (const todo of todos.value) {
      grouped[todo.status].push(todo)
    }
    return grouped
  })

  /** Active todos (not archived) */
  const activeTodos = computed(() => todos.value.filter((t) => t.status !== 'archived'))

  /** Counts per status */
  const statusCounts = computed(() => {
    const counts: Record<Status, number> = {
      backlog: 0,
      todo: 0,
      in_progress: 0,
      done: 0,
      archived: 0,
    }
    for (const todo of todos.value) {
      counts[todo.status]++
    }
    return counts
  })

  // Actions
  async function load() {
    loading.value = true
    error.value = null
    try {
      project.value = await api.fetchProject()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  async function addTodo(params: api.AddTodoParams) {
    error.value = null
    try {
      await api.addTodo(params)
      await load() // Reload to get the full updated state
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  async function updateTodo(number: number, updates: api.UpdateTodoParams) {
    error.value = null
    try {
      await api.updateTodo(number, updates)
      await load()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  async function deleteTodo(number: number) {
    error.value = null
    try {
      await api.deleteTodo(number)
      await load()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  async function moveTodo(number: number, status: Status) {
    return updateTodo(number, { status })
  }

  async function updateProjectSettings(updates: api.UpdateProjectParams) {
    error.value = null
    try {
      await api.updateProjectSettings(updates)
      await load()
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e)
      throw e
    }
  }

  return {
    project,
    loading,
    error,
    todos,
    members,
    projectName,
    prefix,
    todosByStatus,
    activeTodos,
    statusCounts,
    load,
    addTodo,
    updateTodo,
    deleteTodo,
    moveTodo,
    updateProjectSettings,
  }
})
