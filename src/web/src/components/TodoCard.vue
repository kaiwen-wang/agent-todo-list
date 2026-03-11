<script setup lang="ts">
import { computed } from 'vue'
import { NCard } from 'naive-ui'
import type { Todo } from '@/types'
import { PRIORITY_DISPLAY, PRIORITY_COLORS } from '@/types'
import { useProjectStore } from '@/stores/project'

const props = defineProps<{
  todo: Todo
}>()

const store = useProjectStore()

const priorityColor = computed(() => props.todo.priority ? PRIORITY_COLORS[props.todo.priority] : '#6b7280')
const priorityLabel = computed(() => props.todo.priority ? PRIORITY_DISPLAY[props.todo.priority] : 'None')

function openDetail() {
  store.openTodo(props.todo.number)
}
</script>

<template>
  <NCard
    size="small"
    hoverable
    class="todo-card"
    :class="{ done: todo.status === 'completed' }"
    @click="openDetail"
    content-class="card-content"
  >
    <div class="card-header">
      <span class="todo-ref">{{ todo.ref }}</span>
      <span class="priority-dot" :style="{ background: priorityColor }" :title="priorityLabel" />
    </div>
    <div class="card-title">{{ todo.title }}</div>
    <div v-if="todo.assigneeName" class="card-footer">
      <span class="card-assignee">{{ todo.assigneeName }}</span>
    </div>
  </NCard>
</template>

<style scoped>
.todo-card {
  cursor: pointer;
  transition: transform 0.1s;
}

.todo-card:hover {
  transform: translateY(-1px);
}

.todo-card.done {
  opacity: 0.55;
}

.card-content {
  padding: 14px 14px !important;
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
  opacity: 0.45;
  font-family: monospace;
}

.priority-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.card-title {
  font-size: 14px;
  font-weight: 500;
  line-height: 1.5;
  word-break: break-word;
}

.todo-card.done .card-title {
  text-decoration: line-through;
  opacity: 0.7;
}

.card-footer {
  margin-top: 8px;
}

.card-assignee {
  font-size: 11px;
  opacity: 0.5;
}
</style>
