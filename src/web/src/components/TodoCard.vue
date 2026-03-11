<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import type { Todo } from '@/types'
import { PRIORITY_DISPLAY, PRIORITY_COLORS } from '@/types'
import { useProjectStore } from '@/stores/project'

const props = defineProps<{
  todo: Todo
}>()

const router = useRouter()
const store = useProjectStore()

const priorityColor = computed(() => PRIORITY_COLORS[props.todo.priority])
const priorityLabel = computed(() => PRIORITY_DISPLAY[props.todo.priority])

function openDetail() {
  router.push({ name: 'todo-detail', params: { number: props.todo.number } })
}
</script>

<template>
  <div class="todo-card" @click="openDetail" :class="{ done: todo.status === 'done' }">
    <div class="card-header">
      <span class="todo-ref">{{ todo.ref }}</span>
      <span
        class="priority-dot"
        :style="{ background: priorityColor }"
        :title="priorityLabel"
      ></span>
    </div>
    <div class="card-title">{{ todo.title }}</div>
    <div class="card-footer">
      <div class="card-tags" v-if="todo.tags.length">
        <span class="tag" v-for="tag in todo.tags" :key="tag">{{ tag }}</span>
      </div>
      <span class="card-assignee" v-if="todo.assigneeName">{{ todo.assigneeName }}</span>
    </div>
  </div>
</template>

<style scoped>
.todo-card {
  padding: 12px;
  background: var(--bg-card);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  cursor: pointer;
  transition: all 0.15s;
}

.todo-card:hover {
  border-color: var(--text-muted);
  transform: translateY(-1px);
  box-shadow: var(--shadow);
}

.todo-card.done {
  opacity: 0.6;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.todo-ref {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  font-family: monospace;
}

.priority-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.card-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text);
  line-height: 1.4;
  word-break: break-word;
}

.todo-card.done .card-title {
  text-decoration: line-through;
  color: var(--text-dim);
}

.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 8px;
  gap: 8px;
}

.card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.tag {
  font-size: 10px;
  padding: 1px 6px;
  background: var(--bg-hover);
  border-radius: 3px;
  color: var(--text-dim);
}

.card-assignee {
  font-size: 11px;
  color: var(--text-dim);
  white-space: nowrap;
}
</style>
