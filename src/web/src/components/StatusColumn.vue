<script setup lang="ts">
import { ref, computed } from 'vue'
import { NButton } from 'naive-ui'
import type { Todo, Status } from '@/types'
import { STATUS_DISPLAY, STATUS_COLORS } from '@/types'
import TodoCard from './TodoCard.vue'
import CreateTodoModal from './CreateTodoModal.vue'

const props = defineProps<{
  status: Status
  todos: Todo[]
}>()

const label = computed(() => STATUS_DISPLAY[props.status])
const color = computed(() => STATUS_COLORS[props.status])

const showCreate = ref(false)
</script>

<template>
  <div class="status-column">
    <div class="column-header">
      <div class="column-indicator" :style="{ background: color }" />
      <span class="column-label">{{ label }}</span>
      <span class="column-count" :style="{ background: color + '22', color: color }">{{
        todos.length
      }}</span>
      <NButton size="tiny" quaternary @click="showCreate = true" class="add-btn">+</NButton>
    </div>
    <div class="column-body">
      <TodoCard v-for="todo in todos" :key="todo.id" :todo="todo" />
      <div v-if="todos.length === 0" class="column-empty">No items</div>
    </div>
    <CreateTodoModal :open="showCreate" :default-status="status" @close="showCreate = false" />
  </div>
</template>

<style scoped>
.status-column {
  flex: 1;
  min-width: 260px;
  max-width: 340px;
  display: flex;
  flex-direction: column;
  background: #fafafa;
  border-radius: 4px;
  border: 1px solid #e8e8e8;
  overflow: hidden;
}

.column-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 14px;
  border-bottom: 1px solid #e8e8e8;
}

.column-indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  flex-shrink: 0;
}

.column-label {
  font-size: 13px;
  font-weight: 600;
  flex: 1;
}

.column-count {
  min-width: 20px;
  height: 20px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  line-height: 1;
  padding: 0 4px;
  box-sizing: border-box;
}

.column-body {
  flex: 1;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
}

.add-btn {
  font-size: 16px;
  font-weight: 600;
  opacity: 0.4;
  transition: opacity 0.15s;
}

.add-btn:hover {
  opacity: 1;
}

.column-empty {
  padding: 24px 8px;
  text-align: center;
  font-size: 12px;
  opacity: 0.35;
}
</style>
