<script setup lang="ts">
import { ref, computed } from "vue";
import { NButton } from "naive-ui";
import type { Todo, Status } from "@/types";
import { STATUS_DISPLAY, STATUS_COLORS } from "@/types";
import TodoCard from "./TodoCard.vue";
import CreateTodoModal from "./CreateTodoModal.vue";
import { useProjectStore } from "@/stores/project";

const props = defineProps<{
  status: Status;
  todos: Todo[];
}>();

const store = useProjectStore();
const label = computed(() => STATUS_DISPLAY[props.status]);
const color = computed(() => STATUS_COLORS[props.status]);

const showCreate = ref(false);
const isDragOver = ref(false);

function onDragOver(e: DragEvent) {
  e.preventDefault();
  if (e.dataTransfer) {
    e.dataTransfer.dropEffect = "move";
  }
}

function onDragEnter(e: DragEvent) {
  e.preventDefault();
  isDragOver.value = true;
}

function onDragLeave(e: DragEvent) {
  const target = e.currentTarget as HTMLElement;
  if (e.relatedTarget && !target.contains(e.relatedTarget as Node)) {
    isDragOver.value = false;
  }
}

async function onDrop(e: DragEvent) {
  e.preventDefault();
  isDragOver.value = false;

  const isMulti = e.dataTransfer?.getData("application/x-multi-drag");
  const raw = e.dataTransfer?.getData("text/plain");
  if (!raw) return;

  if (isMulti) {
    // Multi-drag: move all selected cards to this column
    await store.bulkUpdateTodos({ status: props.status });
  } else {
    // Single drag
    const todo = store.todos.find((t) => t.id === raw);
    if (todo && todo.status !== props.status) {
      await store.moveTodo(todo.number, props.status);
    }
  }
}
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
    <div
      class="column-body"
      :class="{ 'drag-over': isDragOver }"
      @dragover="onDragOver"
      @dragenter="onDragEnter"
      @dragleave="onDragLeave"
      @drop="onDrop"
    >
      <TodoCard v-for="todo in todos" :key="todo.id" :todo="todo" :column-todos="todos" />
      <!-- <div v-if="todos.length === 0" class="column-empty">No items</div> -->
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
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
  overflow-y: auto;
  overscroll-behavior-y: contain;
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
  padding: 24px;
  text-align: center;
  font-size: 12px;
  user-select: none;
  opacity: 0.35;
}

.drag-over {
  background: rgba(59, 130, 246, 0.08);
}
</style>
