<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NModal,
  NCard,
  NTag,
  NSelect,
  NInput,
  NButtonGroup,
  NButton,
  useMessage,
} from 'naive-ui'
import { useProjectStore } from '@/stores/project'
import type { Status, Priority } from '@/types'
import {
  STATUSES,
  PRIORITIES,
  STATUS_DISPLAY,
  PRIORITY_DISPLAY,
} from '@/types'

const store = useProjectStore()
const message = useMessage()

const priorityOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }))
const assigneeOptions = computed(() =>
  store.members.map((m) => ({ label: m.name, value: m.id })),
)

const isOpen = computed(() => store.selectedTodoNumber !== null)
const todo = computed(() =>
  store.selectedTodoNumber !== null
    ? store.todos.find((t) => t.number === store.selectedTodoNumber)
    : undefined,
)

// Local editable copies
const title = ref('')
const description = ref('')
// Sync local state when a different todo is opened
watch(
  () => store.selectedTodoNumber,
  () => {
    if (todo.value) {
      title.value = todo.value.title
      description.value = todo.value.description
    }
  },
)

function close() {
  store.closeTodo()
}

async function saveField(field: string, value: unknown) {
  if (!todo.value) return
  try {
    await store.updateTodo(todo.value.number, { [field]: value })
  } catch {
    message.error('Failed to update')
  }
}

function commitTitle() {
  const trimmed = title.value.trim()
  if (!todo.value || trimmed === todo.value.title) return
  if (!trimmed) {
    title.value = todo.value.title
    return
  }
  saveField('title', trimmed)
}

function commitDescription() {
  const trimmed = description.value.trim()
  if (!todo.value || trimmed === todo.value.description) return
  saveField('description', trimmed)
}

async function changeStatus(status: Status) {
  if (!todo.value) return
  try {
    await store.moveTodo(todo.value.number, status)
  } catch {
    message.error('Failed to update status')
  }
}

async function changePriority(priority: Priority) {
  if (!todo.value || priority === todo.value.priority) return
  saveField('priority', priority)
}

async function changeAssignee(assignee: string | null) {
  if (!todo.value || assignee === todo.value.assignee) return
  saveField('assignee', assignee)
}

async function handleArchive() {
  if (!todo.value) return
  try {
    await store.moveTodo(todo.value.number, 'archived')
    message.success('Todo moved to trash')
    close()
  } catch {
    message.error('Failed to archive todo')
  }
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString()
}
</script>

<template>
  <NModal :show="isOpen" @update:show="(v: boolean) => !v && close()">
    <NCard
      :bordered="true"
      style="width: 680px; max-width: 95vw; min-height: 70vh"
      role="dialog"
    >
      <div v-if="!todo" class="not-found">
        <p>Todo not found.</p>
      </div>

      <div v-else>
        <!-- Header: ref + title -->
        <div class="detail-header">
          <NTag size="small" :bordered="false" style="font-family: monospace; flex-shrink: 0">
            {{ todo.ref }}
          </NTag>
          <NButton
            v-if="todo.status !== 'archived'"
            size="tiny"
            quaternary
            type="error"
            style="flex-shrink: 0"
            @click="handleArchive"
          >
            <template #icon>
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/></svg>
            </template>
          </NButton>
        </div>

        <input
          v-model="title"
          class="inline-title"
          @blur="commitTitle"
          @keydown.enter="($event.target as HTMLInputElement).blur()"
        />

        <!-- Meta row -->
        <div class="meta-grid">
          <div class="meta-item">
            <label class="meta-label">Status</label>
            <NButtonGroup size="tiny">
              <NButton
                v-for="s in STATUSES"
                :key="s"
                :type="todo.status === s ? 'primary' : 'default'"
                :ghost="todo.status !== s"
                @click="changeStatus(s)"
              >
                {{ STATUS_DISPLAY[s] }}
              </NButton>
            </NButtonGroup>
          </div>

          <div class="meta-item">
            <label class="meta-label">Priority</label>
            <NSelect
              :value="todo.priority"
              :options="priorityOptions"
              size="small"
              style="width: 130px"
              @update:value="changePriority"
            />
          </div>

          <div class="meta-item">
            <label class="meta-label">Assignee</label>
            <NSelect
              :value="todo.assignee"
              :options="assigneeOptions"
              size="small"
              clearable
              placeholder="Unassigned"
              style="width: 160px"
              @update:value="changeAssignee"
            />
          </div>

          <div class="meta-item">
            <label class="meta-label">Created</label>
            <span class="meta-value">{{ formatDate(todo.createdAt) }}</span>
          </div>

          <div class="meta-item">
            <label class="meta-label">Updated</label>
            <span class="meta-value">{{ formatDate(todo.updatedAt) }}</span>
          </div>
        </div>

        <!-- Description -->
        <div class="field-group">
          <label class="meta-label">Description</label>
          <NInput
            v-model:value="description"
            type="textarea"
            :autosize="{ minRows: 6, maxRows: 16 }"
            placeholder="Add a description..."
            @blur="commitDescription"
          />
        </div>
      </div>
    </NCard>
  </NModal>
</template>

<style scoped>
.not-found {
  text-align: center;
  padding: 40px;
  opacity: 0.5;
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.inline-title {
  width: 100%;
  font-size: 22px;
  font-weight: 700;
  line-height: 1.3;
  border: none;
  outline: none;
  background: transparent;
  padding: 4px 0;
  margin-bottom: 16px;
  color: inherit;
  font-family: inherit;
  border-bottom: 2px solid transparent;
  transition: border-color 0.15s;
}

.inline-title:focus {
  border-bottom-color: var(--n-primary-color, #18a058);
}

.meta-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px 24px;
  margin-bottom: 16px;
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta-label {
  font-size: 11px;
  font-weight: 600;
  opacity: 0.5;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.meta-value {
  font-size: 13px;
  line-height: 28px; /* align with input heights */
}

.field-group {
  margin-bottom: 16px;
}

.field-group .meta-label {
  margin-bottom: 6px;
  display: block;
}
</style>
