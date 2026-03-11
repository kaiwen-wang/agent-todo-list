<script setup lang="ts">
import { computed } from 'vue'
import type { Todo, Status } from '@/types'
import { STATUS_DISPLAY, STATUS_COLORS } from '@/types'
import TodoCard from './TodoCard.vue'

const props = defineProps<{
  status: Status
  todos: Todo[]
}>()

const label = computed(() => STATUS_DISPLAY[props.status])
const color = computed(() => STATUS_COLORS[props.status])
</script>

<template>
  <div class="status-column">
    <div class="column-header">
      <div class="column-indicator" :style="{ background: color }"></div>
      <span class="column-label">{{ label }}</span>
      <span class="column-count">{{ todos.length }}</span>
    </div>
    <div class="column-body">
      <TodoCard v-for="todo in todos" :key="todo.id" :todo="todo" />
      <div v-if="todos.length === 0" class="column-empty">No items</div>
    </div>
  </div>
</template>

<style scoped>
.status-column {
  flex: 1;
  min-width: 260px;
  max-width: 340px;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface);
  border-radius: var(--radius);
  border: 1px solid var(--border);
  overflow: hidden;
}

.column-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
}

.column-indicator {
  width: 10px;
  height: 10px;
  border-radius: 3px;
  flex-shrink: 0;
}

.column-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.column-count {
  font-size: 11px;
  color: var(--text-muted);
  background: var(--bg);
  padding: 1px 6px;
  border-radius: 8px;
  font-weight: 500;
}

.column-body {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
}

.column-empty {
  padding: 24px 8px;
  text-align: center;
  font-size: 12px;
  color: var(--text-muted);
}
</style>
