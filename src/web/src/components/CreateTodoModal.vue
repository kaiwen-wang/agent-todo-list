<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Status, Priority } from '@/types'
import { STATUSES, PRIORITIES, STATUS_DISPLAY, PRIORITY_DISPLAY } from '@/types'
import { useProjectStore } from '@/stores/project'

const props = defineProps<{
  open: boolean
  defaultStatus?: Status
}>()

const emit = defineEmits<{
  close: []
}>()

const store = useProjectStore()

const title = ref('')
const description = ref('')
const status = ref<Status>(props.defaultStatus ?? 'todo')
const priority = ref<Priority>('medium')
const tagsInput = ref('')
const submitting = ref(false)

// Reset form when opened
watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      title.value = ''
      description.value = ''
      status.value = props.defaultStatus ?? 'todo'
      priority.value = 'medium'
      tagsInput.value = ''
    }
  },
)

async function submit() {
  if (!title.value.trim()) return
  submitting.value = true
  try {
    const tags = tagsInput.value
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean)
    await store.addTodo({
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      status: status.value,
      priority: priority.value,
      tags: tags.length > 0 ? tags : undefined,
    })
    emit('close')
  } catch {
    // error is shown via store.error
  } finally {
    submitting.value = false
  }
}

function onBackdropClick(e: MouseEvent) {
  if ((e.target as HTMLElement).classList.contains('modal-backdrop')) {
    emit('close')
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" @click="onBackdropClick">
      <div class="modal">
        <div class="modal-header">
          <h2>New Todo</h2>
          <button class="modal-close" @click="emit('close')">&times;</button>
        </div>
        <form @submit.prevent="submit" class="modal-body">
          <div class="form-group">
            <label for="title">Title</label>
            <input
              id="title"
              v-model="title"
              type="text"
              placeholder="What needs to be done?"
              autofocus
            />
          </div>

          <div class="form-group">
            <label for="description">Description</label>
            <textarea
              id="description"
              v-model="description"
              placeholder="Add details... (optional)"
              rows="3"
            ></textarea>
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="status">Status</label>
              <select id="status" v-model="status">
                <option v-for="s in STATUSES" :key="s" :value="s">{{ STATUS_DISPLAY[s] }}</option>
              </select>
            </div>

            <div class="form-group">
              <label for="priority">Priority</label>
              <select id="priority" v-model="priority">
                <option v-for="p in PRIORITIES" :key="p" :value="p">
                  {{ PRIORITY_DISPLAY[p] }}
                </option>
              </select>
            </div>
          </div>

          <div class="form-group">
            <label for="tags">Tags</label>
            <input id="tags" v-model="tagsInput" type="text" placeholder="bug, frontend, urgent" />
            <span class="form-hint">Comma-separated</span>
          </div>

          <div class="modal-actions">
            <button type="button" class="btn" @click="emit('close')">Cancel</button>
            <button type="submit" class="btn btn-primary" :disabled="!title.trim() || submitting">
              {{ submitting ? 'Creating...' : 'Create Todo' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  background: var(--bg-surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  width: 100%;
  max-width: 520px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.4);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.modal-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.modal-close {
  background: none;
  border: none;
  color: var(--text-dim);
  font-size: 22px;
  cursor: pointer;
  padding: 0 4px;
  line-height: 1;
}

.modal-close:hover {
  color: var(--text);
}

.modal-body {
  padding: 20px;
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

.form-hint {
  font-size: 11px;
  color: var(--text-muted);
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 8px;
}
</style>
