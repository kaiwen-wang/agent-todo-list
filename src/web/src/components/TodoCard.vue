<script setup lang="ts">
import { computed } from 'vue'
import { NCard, NTag, NSpace } from 'naive-ui'
import type { Todo } from '@/types'
import { PRIORITY_DISPLAY, PRIORITY_COLORS } from '@/types'
import { useProjectStore } from '@/stores/project'

const props = defineProps<{
  todo: Todo
}>()

const store = useProjectStore()

const priorityColor = computed(() => PRIORITY_COLORS[props.todo.priority])
const priorityLabel = computed(() => PRIORITY_DISPLAY[props.todo.priority])

function openDetail() {
  store.openTodo(props.todo.number)
}
</script>

<template>
  <NCard
    size="small"
    hoverable
    class="todo-card"
    :class="{ done: todo.status === 'done' }"
    @click="openDetail"
    content-class="card-content"
  >
    <div class="card-header">
      <span class="todo-ref">{{ todo.ref }}</span>
      <span class="priority-dot" :style="{ background: priorityColor }" :title="priorityLabel" />
    </div>
    <div class="card-title">{{ todo.title }}</div>
    <NSpace v-if="todo.tags.length || todo.assigneeName" :size="4" class="card-footer">
      <NTag v-for="tag in todo.tags" :key="tag" size="tiny" round :bordered="false">
        {{ tag }}
      </NTag>
      <span v-if="todo.assigneeName" class="card-assignee">{{ todo.assigneeName }}</span>
    </NSpace>
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
  padding: 10px 12px !important;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
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
  font-size: 13px;
  font-weight: 500;
  line-height: 1.4;
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
