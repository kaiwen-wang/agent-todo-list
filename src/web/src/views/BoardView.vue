<script setup lang="ts">
import { ref } from 'vue'
import { useProjectStore } from '@/stores/project'
import { BOARD_STATUSES } from '@/types'
import StatusColumn from '@/components/StatusColumn.vue'
import CreateTodoModal from '@/components/CreateTodoModal.vue'

const store = useProjectStore()
const showCreate = ref(false)
</script>

<template>
  <div class="board-view">
    <div class="board-toolbar">
      <h2>Board</h2>
      <button class="btn btn-primary" @click="showCreate = true">+ New Todo</button>
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
  </div>
</template>

<style scoped>
.board-view {
  height: calc(100vh - 56px);
  display: flex;
  flex-direction: column;
  overflow: hidden;
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
}
</style>
