<script setup lang="ts">
import { ref, onUnmounted } from "vue";
import { NButton } from "naive-ui";
import { useProjectStore } from "@/stores/project";
import { BOARD_STATUSES } from "@/types";
import StatusColumn from "@/components/StatusColumn.vue";
import CreateTodoModal from "@/components/CreateTodoModal.vue";
import BatchActionBar from "@/components/BatchActionBar.vue";

const store = useProjectStore();
const showCreate = ref(false);
const boardRef = ref<HTMLElement | null>(null);

function onBoardClick(e: MouseEvent) {
  if (!store.hasSelection) return;
  // If click originated inside a todo card, let the card handle it
  const target = e.target as HTMLElement;
  if (target.closest(".todo-card")) return;
  store.clearSelection();
}

// Auto-scroll horizontally when dragging near the edges of the board
const EDGE_ZONE = 80; // px from edge to trigger scroll
const SCROLL_SPEED = 6; // px per frame
let scrollRaf = 0;

function onBoardDragOver(e: DragEvent) {
  const board = boardRef.value;
  if (!board) return;
  const rect = board.getBoundingClientRect();
  const x = e.clientX;

  cancelAnimationFrame(scrollRaf);

  if (x < rect.left + EDGE_ZONE) {
    const intensity = 1 - (x - rect.left) / EDGE_ZONE;
    autoScroll(board, -SCROLL_SPEED * Math.max(intensity, 0.2));
  } else if (x > rect.right - EDGE_ZONE) {
    const intensity = 1 - (rect.right - x) / EDGE_ZONE;
    autoScroll(board, SCROLL_SPEED * Math.max(intensity, 0.2));
  }
}

function autoScroll(el: HTMLElement, delta: number) {
  el.scrollLeft += delta;
  scrollRaf = requestAnimationFrame(() => autoScroll(el, delta));
}

function onBoardDragEnd() {
  cancelAnimationFrame(scrollRaf);
}

onUnmounted(() => cancelAnimationFrame(scrollRaf));
</script>

<template>
  <div class="board-view">
    <div class="board-toolbar">
      <h2>Board</h2>
      <NButton type="primary" size="small" @click="showCreate = true">+ New Todo</NButton>
    </div>
    <div
      ref="boardRef"
      class="board"
      @click="onBoardClick"
      @dragover="onBoardDragOver"
      @dragend="onBoardDragEnd"
      @drop="onBoardDragEnd"
      @dragleave="onBoardDragEnd"
    >
      <StatusColumn
        v-for="status in BOARD_STATUSES"
        :key="status"
        :status="status"
        :todos="store.todosByStatus[status]"
      />
    </div>
    <CreateTodoModal :open="showCreate" @close="showCreate = false" />
    <BatchActionBar />
  </div>
</template>

<style scoped>
.board-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  overscroll-behavior: contain;
}

.board-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  flex-shrink: 0;
}

.board-toolbar h2 {
  font-size: 16px;
  font-weight: 600;
}

.board {
  flex: 1;
  display: flex;
  gap: 12px;
  padding: 0 24px 24px;
  overflow-x: auto;
  overscroll-behavior: contain;
}
</style>
