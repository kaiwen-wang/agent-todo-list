<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
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

const props = defineProps<{
  number: number
}>()

const store = useProjectStore()
const router = useRouter()

const editing = ref(false)
const editTitle = ref('')
const editDescription = ref('')
const editPriority = ref<Priority>('medium')
const editStatus = ref<Status>('todo')
const editTags = ref('')
const saving = ref(false)

const todo = computed(() => store.todos.find((t) => t.number === props.number))

function startEdit() {
  if (!todo.value) return
  editTitle.value = todo.value.title
  editDescription.value = todo.value.description
  editPriority.value = todo.value.priority
  editStatus.value = todo.value.status
  editTags.value = todo.value.tags.join(', ')
  editing.value = true
}

async function saveEdit() {
  if (!todo.value) return
  saving.value = true
  try {
    const tags = editTags.value
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean)
    await store.updateTodo(todo.value.number, {
      title: editTitle.value.trim(),
      description: editDescription.value.trim(),
      priority: editPriority.value,
      status: editStatus.value,
      tags,
    })
    editing.value = false
  } catch {
    // error via store
  } finally {
    saving.value = false
  }
}

function cancelEdit() {
  editing.value = false
}

async function changeStatus(status: Status) {
  if (!todo.value) return
  await store.moveTodo(todo.value.number, status)
}

async function handleDelete() {
  if (!todo.value) return
  await store.deleteTodo(todo.value.number)
  router.back()
}

function goBack() {
  router.back()
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString()
}
</script>

<template>
  <div class="detail-view">
    <div class="detail-nav">
      <button class="btn btn-sm" @click="goBack">&larr; Back</button>
    </div>

    <div v-if="!todo" class="not-found">
      <p>Todo #{{ number }} not found.</p>
      <button class="btn" @click="goBack">Go back</button>
    </div>

    <div v-else class="detail-content">
      <!-- View mode -->
      <template v-if="!editing">
        <div class="detail-header">
          <div class="detail-header-left">
            <span class="detail-ref">{{ todo.ref }}</span>
            <h1 class="detail-title">{{ todo.title }}</h1>
          </div>
          <div class="detail-actions">
            <button class="btn btn-sm" @click="startEdit">Edit</button>
            <button class="btn btn-sm btn-danger" @click="handleDelete">Delete</button>
          </div>
        </div>

        <div class="detail-meta">
          <div class="meta-group">
            <label>Status</label>
            <div class="status-selector">
              <button
                v-for="s in STATUSES"
                :key="s"
                class="status-option"
                :class="{ active: todo.status === s }"
                :style="
                  todo.status === s
                    ? { background: STATUS_COLORS[s] + '22', color: STATUS_COLORS[s], borderColor: STATUS_COLORS[s] + '44' }
                    : {}
                "
                @click="changeStatus(s)"
              >
                {{ STATUS_DISPLAY[s] }}
              </button>
            </div>
          </div>

          <div class="meta-row">
            <div class="meta-group">
              <label>Priority</label>
              <span class="priority-indicator">
                <span
                  class="priority-dot"
                  :style="{ background: PRIORITY_COLORS[todo.priority] }"
                ></span>
                {{ PRIORITY_DISPLAY[todo.priority] }}
              </span>
            </div>
            <div class="meta-group">
              <label>Assignee</label>
              <span>{{ todo.assigneeName || 'Unassigned' }}</span>
            </div>
            <div class="meta-group">
              <label>Created</label>
              <span class="date-text">{{ formatDate(todo.createdAt) }}</span>
            </div>
            <div class="meta-group">
              <label>Updated</label>
              <span class="date-text">{{ formatDate(todo.updatedAt) }}</span>
            </div>
          </div>

          <div class="meta-group" v-if="todo.tags.length > 0">
            <label>Tags</label>
            <div class="tags-list">
              <span class="tag" v-for="tag in todo.tags" :key="tag">{{ tag }}</span>
            </div>
          </div>
        </div>

        <div class="detail-description" v-if="todo.description">
          <label>Description</label>
          <div class="description-body">{{ todo.description }}</div>
        </div>
        <div class="detail-description" v-else>
          <label>Description</label>
          <div class="description-empty">No description provided.</div>
        </div>
      </template>

      <!-- Edit mode -->
      <template v-else>
        <form @submit.prevent="saveEdit" class="edit-form">
          <div class="form-group">
            <label for="edit-title">Title</label>
            <input id="edit-title" v-model="editTitle" type="text" />
          </div>

          <div class="form-group">
            <label for="edit-description">Description</label>
            <textarea id="edit-description" v-model="editDescription" rows="5"></textarea>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="edit-status">Status</label>
              <select id="edit-status" v-model="editStatus">
                <option v-for="s in STATUSES" :key="s" :value="s">{{ STATUS_DISPLAY[s] }}</option>
              </select>
            </div>
            <div class="form-group">
              <label for="edit-priority">Priority</label>
              <select id="edit-priority" v-model="editPriority">
                <option v-for="p in PRIORITIES" :key="p" :value="p">
                  {{ PRIORITY_DISPLAY[p] }}
                </option>
              </select>
            </div>
          </div>

          <div class="form-group">
            <label for="edit-tags">Tags</label>
            <input id="edit-tags" v-model="editTags" type="text" placeholder="comma-separated" />
          </div>

          <div class="edit-actions">
            <button type="button" class="btn" @click="cancelEdit">Cancel</button>
            <button type="submit" class="btn btn-primary" :disabled="saving">
              {{ saving ? 'Saving...' : 'Save Changes' }}
            </button>
          </div>
        </form>
      </template>
    </div>
  </div>
</template>

<style scoped>
.detail-view {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

.detail-nav {
  margin-bottom: 20px;
}

.not-found {
  text-align: center;
  padding: 40px;
  color: var(--text-dim);
}

.not-found p {
  margin-bottom: 16px;
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 24px;
  gap: 16px;
}

.detail-header-left {
  flex: 1;
}

.detail-ref {
  font-family: monospace;
  font-size: 13px;
  color: var(--text-muted);
  font-weight: 600;
  margin-bottom: 4px;
  display: block;
}

.detail-title {
  font-size: 24px;
  font-weight: 700;
  line-height: 1.3;
}

.detail-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.detail-meta {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 24px;
  padding: 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.meta-group label {
  margin-bottom: 6px;
}

.meta-row {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}

.status-selector {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.status-option {
  padding: 4px 10px;
  font-size: 12px;
  font-weight: 500;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  transition: all 0.15s;
}

.status-option:hover {
  background: var(--bg-hover);
}

.status-option.active {
  font-weight: 600;
}

.priority-indicator {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}

.priority-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.date-text {
  font-size: 13px;
  color: var(--text-dim);
}

.tags-list {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.tag {
  font-size: 12px;
  padding: 2px 8px;
  background: var(--bg-hover);
  border-radius: var(--radius-sm);
  color: var(--text-dim);
}

.detail-description {
  margin-bottom: 24px;
}

.detail-description label {
  margin-bottom: 8px;
}

.description-body {
  font-size: 14px;
  line-height: 1.6;
  color: var(--text);
  white-space: pre-wrap;
  padding: 16px;
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
}

.description-empty {
  font-size: 13px;
  color: var(--text-muted);
  font-style: italic;
}

/* Edit form */
.edit-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.form-row {
  display: flex;
  gap: 12px;
}

.edit-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 8px;
}
</style>
