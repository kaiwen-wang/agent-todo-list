<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import {
  NModal,
  NCard,
  NButton,
  NSpace,
  NTag,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NDescriptions,
  NDescriptionsItem,
  NPopconfirm,
  NButtonGroup,
  useMessage,
} from 'naive-ui'
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

const store = useProjectStore()
const message = useMessage()

const editing = ref(false)
const editTitle = ref('')
const editDescription = ref('')
const editPriority = ref<Priority>('medium')
const editStatus = ref<Status>('todo')
const editTags = ref('')
const saving = ref(false)

const statusOptions = STATUSES.map((s) => ({ label: STATUS_DISPLAY[s], value: s }))
const priorityOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }))

const isOpen = computed(() => store.selectedTodoNumber !== null)
const todo = computed(() =>
  store.selectedTodoNumber !== null
    ? store.todos.find((t) => t.number === store.selectedTodoNumber)
    : undefined,
)

// Reset edit mode when modal opens/closes
watch(isOpen, (open) => {
  if (!open) editing.value = false
})

function close() {
  store.closeTodo()
}

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
    message.success('Todo updated')
  } catch {
    message.error('Failed to update todo')
  } finally {
    saving.value = false
  }
}

function cancelEdit() {
  editing.value = false
}

async function changeStatus(status: Status) {
  if (!todo.value) return
  try {
    await store.moveTodo(todo.value.number, status)
  } catch {
    message.error('Failed to update status')
  }
}

async function handleDelete() {
  if (!todo.value) return
  try {
    await store.deleteTodo(todo.value.number)
    message.success('Todo deleted')
    close()
  } catch {
    message.error('Failed to delete todo')
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
      closable
      @close="close"
      style="width: 680px; max-width: 95vw"
      role="dialog"
    >
      <div v-if="!todo" class="not-found">
        <p>Todo not found.</p>
      </div>

      <div v-else-if="!editing">
        <!-- View mode -->
        <div class="detail-header">
          <div class="detail-header-left">
            <NTag size="small" :bordered="false" style="font-family: monospace">
              {{ todo.ref }}
            </NTag>
            <h1 class="detail-title">{{ todo.title }}</h1>
          </div>
          <NSpace :size="8">
            <NButton size="small" @click="startEdit">Edit</NButton>
            <NPopconfirm @positive-click="handleDelete">
              <template #trigger>
                <NButton size="small" type="error" ghost>Delete</NButton>
              </template>
              Delete this todo permanently?
            </NPopconfirm>
          </NSpace>
        </div>

        <!-- Status buttons -->
        <NCard size="small" class="detail-meta">
          <div class="meta-section">
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

          <NDescriptions :column="4" label-placement="top" size="small" class="meta-descriptions">
            <NDescriptionsItem label="Priority">
              <NSpace :size="6" align="center">
                <span
                  class="priority-dot"
                  :style="{ background: PRIORITY_COLORS[todo.priority] }"
                />
                {{ PRIORITY_DISPLAY[todo.priority] }}
              </NSpace>
            </NDescriptionsItem>
            <NDescriptionsItem label="Assignee">
              {{ todo.assigneeName || 'Unassigned' }}
            </NDescriptionsItem>
            <NDescriptionsItem label="Created">
              {{ formatDate(todo.createdAt) }}
            </NDescriptionsItem>
            <NDescriptionsItem label="Updated">
              {{ formatDate(todo.updatedAt) }}
            </NDescriptionsItem>
          </NDescriptions>

          <div v-if="todo.tags.length > 0" class="meta-section">
            <label class="meta-label">Tags</label>
            <NSpace :size="6">
              <NTag v-for="tag in todo.tags" :key="tag" size="small" round :bordered="false">
                {{ tag }}
              </NTag>
            </NSpace>
          </div>
        </NCard>

        <!-- Description -->
        <NCard size="small" title="Description" class="detail-description">
          <div v-if="todo.description" class="description-body">{{ todo.description }}</div>
          <div v-else class="description-empty">No description provided.</div>
        </NCard>
      </div>

      <!-- Edit mode -->
      <div v-else>
        <h3 style="margin-bottom: 16px">Edit Todo</h3>
        <NForm label-placement="top" @submit.prevent="saveEdit">
          <NFormItem label="Title">
            <NInput v-model:value="editTitle" />
          </NFormItem>

          <NFormItem label="Description">
            <NInput v-model:value="editDescription" type="textarea" :rows="5" />
          </NFormItem>

          <NSpace :size="12">
            <NFormItem label="Status" style="flex: 1">
              <NSelect v-model:value="editStatus" :options="statusOptions" />
            </NFormItem>
            <NFormItem label="Priority" style="flex: 1">
              <NSelect v-model:value="editPriority" :options="priorityOptions" />
            </NFormItem>
          </NSpace>

          <NFormItem label="Tags">
            <NInput v-model:value="editTags" placeholder="comma-separated" />
          </NFormItem>

          <NSpace justify="end" :size="8">
            <NButton @click="cancelEdit">Cancel</NButton>
            <NButton type="primary" @click="saveEdit" :loading="saving">Save Changes</NButton>
          </NSpace>
        </NForm>
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
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 20px;
  gap: 16px;
}

.detail-header-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.detail-title {
  font-size: 22px;
  font-weight: 700;
  line-height: 1.3;
}

.detail-meta {
  margin-bottom: 16px;
}

.meta-section {
  margin-bottom: 12px;
}

.meta-section:last-child {
  margin-bottom: 0;
}

.meta-label {
  display: block;
  font-size: 11px;
  font-weight: 600;
  opacity: 0.5;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.meta-descriptions {
  margin-top: 12px;
}

.priority-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}

.detail-description {
  margin-top: 16px;
}

.description-body {
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
}

.description-empty {
  font-size: 13px;
  opacity: 0.35;
  font-style: italic;
}
</style>
