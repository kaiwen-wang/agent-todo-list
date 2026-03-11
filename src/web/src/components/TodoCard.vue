<script setup lang="ts">
import { computed, type Component } from 'vue'
import { NCard, NIcon } from 'naive-ui'
import {
  AntennaBars1,
  AntennaBars2,
  AntennaBars3,
  AntennaBars4,
  AntennaBars5,
} from '@vicons/tabler'
import type { Todo, Priority } from '@/types'
import { PRIORITY_DISPLAY, PRIORITY_COLORS } from '@/types'
import { useProjectStore } from '@/stores/project'

const PRIORITY_ICON: Record<Priority, Component> = {
  none: AntennaBars1,
  low: AntennaBars2,
  medium: AntennaBars3,
  high: AntennaBars4,
  urgent: AntennaBars5,
}

const props = defineProps<{
  todo: Todo
}>()

const store = useProjectStore()

const priorityIcon = computed(() => PRIORITY_ICON[props.todo.priority])
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
    :class="{ done: todo.status === 'completed' }"
    @click="openDetail"
    content-class="card-content"
  >
    <div class="card-header">
      <span class="todo-ref">{{ todo.ref }}</span>
      <NIcon :size="16" :color="priorityColor" :title="priorityLabel">
        <component :is="priorityIcon" />
      </NIcon>
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
