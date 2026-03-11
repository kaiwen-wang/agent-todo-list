<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useProjectStore } from '@/stores/project'
import type { Status, Priority } from '@/types'
import {
  STATUSES,
  PRIORITIES,
  STATUS_DISPLAY,
  PRIORITY_DISPLAY,
  STATUS_COLORS,
  PRIORITY_COLORS,
} from '@/types'
import CreateTodoModal from '@/components/CreateTodoModal.vue'

const store = useProjectStore()
const router = useRouter()

const showCreate = ref(false)
const filterStatus = ref<Status | ''>('')
const filterPriority = ref<Priority | ''>('')
const searchQuery = ref('')

const filteredTodos = computed(() => {
  let list = store.activeTodos
  if (filterStatus.value) {
    list = list.filter((t) => t.status === filterStatus.value)
  }
  if (filterPriority.value) {
    list = list.filter((t) => t.priority === filterPriority.value)
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter(
      (t) => t.title.toLowerCase().includes(q) || t.description.toLowerCase().includes(q),
    )
  }
  return list
})

function openTodo(number: number) {
  router.push({ name: 'todo-detail', params: { number } })
}

async function quickStatusChange(todoNumber: number, status: Status) {
  await store.moveTodo(todoNumber, status)
}
</script>

<template>
  <div class="list-view">
    <div class="list-toolbar">
      <h2>List</h2>
      <div class="list-filters">
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search..."
          class="search-input"
        />
        <select v-model="filterStatus">
          <option value="">All statuses</option>
          <option v-for="s in STATUSES" :key="s" :value="s">{{ STATUS_DISPLAY[s] }}</option>
        </select>
        <select v-model="filterPriority">
          <option value="">All priorities</option>
          <option v-for="p in PRIORITIES" :key="p" :value="p">{{ PRIORITY_DISPLAY[p] }}</option>
        </select>
      </div>
      <button class="btn btn-primary" @click="showCreate = true">+ New Todo</button>
    </div>

    <div class="list-table-container">
      <table class="list-table">
        <thead>
          <tr>
            <th class="col-ref">Ref</th>
            <th class="col-title">Title</th>
            <th class="col-status">Status</th>
            <th class="col-priority">Priority</th>
            <th class="col-assignee">Assignee</th>
            <th class="col-tags">Tags</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="todo in filteredTodos"
            :key="todo.id"
            class="todo-row"
            @click="openTodo(todo.number)"
          >
            <td class="col-ref">
              <span class="ref-text">{{ todo.ref }}</span>
            </td>
            <td class="col-title">
              <span :class="{ done: todo.status === 'done' }">{{ todo.title }}</span>
            </td>
            <td class="col-status">
              <span
                class="status-badge"
                :style="{ background: STATUS_COLORS[todo.status] + '22', color: STATUS_COLORS[todo.status] }"
              >
                {{ STATUS_DISPLAY[todo.status] }}
              </span>
            </td>
            <td class="col-priority">
              <span class="priority-indicator">
                <span
                  class="priority-dot"
                  :style="{ background: PRIORITY_COLORS[todo.priority] }"
                ></span>
                {{ PRIORITY_DISPLAY[todo.priority] }}
              </span>
            </td>
            <td class="col-assignee">
              <span v-if="todo.assigneeName" class="assignee-text">{{ todo.assigneeName }}</span>
              <span v-else class="no-assignee">&mdash;</span>
            </td>
            <td class="col-tags">
              <span class="tag" v-for="tag in todo.tags" :key="tag">{{ tag }}</span>
            </td>
          </tr>
          <tr v-if="filteredTodos.length === 0">
            <td colspan="6" class="empty-row">No todos found</td>
          </tr>
        </tbody>
      </table>
    </div>

    <CreateTodoModal :open="showCreate" @close="showCreate = false" />
  </div>
</template>

<style scoped>
.list-view {
  height: calc(100vh - 56px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  gap: 16px;
  flex-shrink: 0;
}

.list-toolbar h2 {
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
}

.list-filters {
  display: flex;
  gap: 8px;
  flex: 1;
  justify-content: center;
}

.search-input {
  max-width: 220px;
}

.list-filters select {
  width: auto;
  min-width: 140px;
}

.list-table-container {
  flex: 1;
  overflow: auto;
  padding: 0 24px 24px;
}

.list-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.list-table th {
  text-align: left;
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  border-bottom: 1px solid var(--border);
  position: sticky;
  top: 0;
  background: var(--bg);
  z-index: 1;
}

.list-table td {
  padding: 10px 12px;
  border-bottom: 1px solid var(--border);
  vertical-align: middle;
}

.todo-row {
  cursor: pointer;
  transition: background 0.1s;
}

.todo-row:hover {
  background: var(--bg-surface);
}

.col-ref {
  width: 80px;
}

.ref-text {
  font-family: monospace;
  font-size: 12px;
  color: var(--text-muted);
  font-weight: 500;
}

.col-title {
  min-width: 200px;
}

.col-title .done {
  text-decoration: line-through;
  color: var(--text-dim);
}

.col-status {
  width: 120px;
}

.status-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
  white-space: nowrap;
}

.col-priority {
  width: 100px;
}

.priority-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-dim);
}

.priority-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.col-assignee {
  width: 120px;
}

.assignee-text {
  color: var(--text-dim);
}

.no-assignee {
  color: var(--text-muted);
}

.col-tags {
  width: 150px;
}

.tag {
  display: inline-block;
  font-size: 10px;
  padding: 1px 6px;
  background: var(--bg-hover);
  border-radius: 3px;
  color: var(--text-dim);
  margin-right: 4px;
}

.empty-row {
  text-align: center;
  padding: 40px 12px !important;
  color: var(--text-muted);
}
</style>
