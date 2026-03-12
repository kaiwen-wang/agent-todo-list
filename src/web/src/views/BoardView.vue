<script setup lang="ts">
import { ref } from "vue";
import { NButton } from "naive-ui";
import { useProjectStore } from "@/stores/project";
import { BOARD_STATUSES } from "@/types";
import StatusColumn from "@/components/StatusColumn.vue";
import CreateTodoModal from "@/components/CreateTodoModal.vue";
import BatchActionBar from "@/components/BatchActionBar.vue";

const store = useProjectStore();
const showCreate = ref(false);
</script>

<template>
  <div class="board-view">
    <div class="board-toolbar">
      <h2>Board</h2>
      <NButton type="primary" size="small" @click="showCreate = true">+ New Todo</NButton>
    </div>
    <div class="board">
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
