<script setup lang="ts">
import { ref, computed, type Component } from "vue";
import { NCard, NIcon } from "naive-ui";
import {
  AntennaBars1,
  AntennaBars2,
  AntennaBars3,
  AntennaBars4,
  AntennaBars5,
} from "@vicons/tabler";
import type { Todo, Priority } from "@/types";
import {
  PRIORITY_DISPLAY,
  PRIORITY_COLORS,
  DIFFICULTY_DISPLAY,
  DIFFICULTY_COLORS,
  LABEL_DISPLAY,
  LABEL_COLORS,
} from "@/types";
import { useProjectStore } from "@/stores/project";

const PRIORITY_ICON: Record<Priority, Component> = {
  none: AntennaBars1,
  low: AntennaBars2,
  medium: AntennaBars3,
  high: AntennaBars4,
  urgent: AntennaBars5,
};

const props = defineProps<{
  todo: Todo;
  columnTodos?: Todo[];
}>();

const emit = defineEmits<{
  dragStart: [todo: Todo];
}>();

const store = useProjectStore();
const isDragging = ref(false);

const priorityIcon = computed(() => PRIORITY_ICON[props.todo.priority]);
const priorityColor = computed(() => PRIORITY_COLORS[props.todo.priority]);
const priorityLabel = computed(() => PRIORITY_DISPLAY[props.todo.priority]);

const isSelected = computed(() => store.selectedTodoIds.has(props.todo.id));

function handleClick(e: MouseEvent) {
  if (isDragging.value) return;

  const isMod = e.metaKey || e.ctrlKey;

  // CMD/Ctrl+Shift+Click → range select within column
  if (isMod && e.shiftKey && props.columnTodos) {
    e.preventDefault();
    store.rangeSelect(props.todo.id, props.columnTodos);
    return;
  }

  // CMD/Ctrl+Click → toggle selection
  if (isMod) {
    e.preventDefault();
    store.toggleSelect(props.todo.id);
    return;
  }

  // Plain click (or Shift+Click without CMD) → open detail or clear selection
  if (store.hasSelection) {
    store.clearSelection();
    return;
  }
  store.openTodo(props.todo.number);
}

function onDragStart(e: DragEvent) {
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = "move";
    // If this card is part of a multi-selection, encode all selected IDs
    if (store.selectedTodoIds.has(props.todo.id) && store.selectedTodoIds.size > 1) {
      e.dataTransfer.setData("text/plain", JSON.stringify([...store.selectedTodoIds]));
      e.dataTransfer.setData("application/x-multi-drag", "true");
    } else {
      e.dataTransfer.setData("text/plain", props.todo.id);
    }
    emit("dragStart", props.todo);
    isDragging.value = true;
    setTimeout(() => {
      isDragging.value = false;
    }, 100);
  }
}
</script>

<template>
  <NCard
    draggable="true"
    size="small"
    class="todo-card"
    :class="{ done: todo.status === 'completed', selected: isSelected }"
    @click="handleClick"
    @dragstart="onDragStart"
    content-class="card-content"
  >
    <div class="card-header">
      <span class="card-header-left">
        <span class="todo-ref">{{ todo.ref }}</span>
        <span
          v-if="todo.difficulty && todo.difficulty !== 'none'"
          class="difficulty-num"
          :style="{ color: DIFFICULTY_COLORS[todo.difficulty] }"
          :title="'Difficulty: ' + DIFFICULTY_DISPLAY[todo.difficulty]"
          >{{ DIFFICULTY_DISPLAY[todo.difficulty] }}</span
        >
      </span>
      <NIcon :size="16" :color="priorityColor" :title="priorityLabel">
        <component :is="priorityIcon" />
      </NIcon>
    </div>
    <div class="card-title">{{ todo.title }}</div>
    <div v-if="todo.description" class="card-description">{{ todo.description }}</div>
    <div v-if="todo.labels?.length || todo.assigneeName" class="card-footer">
      <div v-if="todo.labels?.length" class="card-labels">
        <span
          v-for="l in todo.labels"
          :key="l"
          class="card-label"
          :style="{ background: LABEL_COLORS[l] + '22', color: LABEL_COLORS[l] }"
          >{{ LABEL_DISPLAY[l] }}</span
        >
      </div>
      <span v-if="todo.assigneeName" class="card-assignee">{{ todo.assigneeName }}</span>
    </div>
  </NCard>
</template>

<style scoped>
.todo-card {
  cursor: pointer;
  border-radius: 0 !important;
  border: none !important;
  box-shadow: none !important;
  border-bottom: 1px solid #e8e8e8 !important;
}

.todo-card :deep(.n-card) {
  border-radius: 0 !important;
  box-shadow: none !important;
  border: none !important;
}

.todo-card :deep(.n-card__content) {
  border-radius: 0 !important;
}

.todo-card.selected {
  background: rgba(59, 130, 246, 0.1) !important;
}

.todo-card:hover .card-title {
  color: #3b82f6;
}

.todo-card.done {
  opacity: 0.55;
}

.card-content {
  padding: 12px 14px !important;
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

.card-header-left {
  display: flex;
  align-items: baseline;
  gap: 5px;
}

.difficulty-num {
  font-size: 10px;
  font-weight: 700;
  font-family: monospace;
  letter-spacing: 0.3px;
  text-transform: uppercase;
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

.card-description {
  font-size: 11px;
  line-height: 1.25;
  color: #888;
  margin-top: 3px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  word-break: break-word;
}

.card-footer {
  margin-top: 8px;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 6px;
}

.card-labels {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.card-label {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 8px;
  white-space: nowrap;
}

.card-assignee {
  font-size: 11px;
  opacity: 0.5;
  margin-left: auto;
}
</style>
