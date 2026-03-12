<script setup lang="ts">
import { ref } from "vue";
import { NButton, NPopselect } from "naive-ui";
import { useProjectStore } from "@/stores/project";
import {
  STATUSES,
  STATUS_DISPLAY,
  STATUS_COLORS,
  PRIORITIES,
  PRIORITY_DISPLAY,
  PRIORITY_COLORS,
} from "@/types";
import type { Status, Priority } from "@/types";

const store = useProjectStore();
const showDeleteConfirm = ref(false);

const statusOptions = STATUSES.map((s) => ({
  label: STATUS_DISPLAY[s],
  value: s,
}));

const priorityOptions = PRIORITIES.map((p) => ({
  label: PRIORITY_DISPLAY[p],
  value: p,
}));

const memberOptions = store.members.map((m) => ({
  label: m.name,
  value: m.id,
}));

async function handleMoveStatus(status: Status) {
  await store.bulkUpdateTodos({ status });
}

async function handleSetPriority(priority: Priority) {
  await store.bulkUpdateTodos({ priority });
}

async function handleAssign(memberId: string) {
  await store.bulkUpdateTodos({ assignee: memberId });
}

async function handleDelete() {
  if (!showDeleteConfirm.value) {
    showDeleteConfirm.value = true;
    return;
  }
  await store.bulkDeleteTodos();
  showDeleteConfirm.value = false;
}

function handleArchive() {
  store.bulkUpdateTodos({ status: "archived" as Status });
}
</script>

<template>
  <div class="batch-bar" v-if="store.hasSelection">
    <span class="batch-count">{{ store.selectionCount }} selected</span>

    <NPopselect
      :options="statusOptions"
      trigger="click"
      @update:value="handleMoveStatus"
      :render-label="
        (option: any) => {
          const s = option.value as Status;
          return option.label;
        }
      "
    >
      <NButton size="small" quaternary>Move to...</NButton>
    </NPopselect>

    <NPopselect :options="priorityOptions" trigger="click" @update:value="handleSetPriority">
      <NButton size="small" quaternary>Priority</NButton>
    </NPopselect>

    <NPopselect
      v-if="memberOptions.length"
      :options="memberOptions"
      trigger="click"
      @update:value="handleAssign"
    >
      <NButton size="small" quaternary>Assign</NButton>
    </NPopselect>

    <NButton size="small" quaternary @click="handleArchive">Archive</NButton>

    <NButton
      size="small"
      :type="showDeleteConfirm ? 'error' : 'default'"
      :quaternary="!showDeleteConfirm"
      @click="handleDelete"
    >
      {{ showDeleteConfirm ? "Confirm Delete" : "Delete" }}
    </NButton>

    <div class="batch-spacer" />

    <NButton size="small" quaternary @click="store.clearSelection()"> Cancel </NButton>
  </div>
</template>

<style scoped>
.batch-bar {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 6px;
  background: #1a1a1a;
  color: #fff;
  padding: 8px 16px;
  border-radius: 10px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.28);
  z-index: 1000;
  font-size: 13px;
  min-width: 420px;
}

.batch-count {
  font-weight: 600;
  font-size: 13px;
  margin-right: 8px;
  white-space: nowrap;
  color: #93c5fd;
}

.batch-bar :deep(.n-button) {
  color: #e5e7eb !important;
  font-size: 12px;
}

.batch-bar :deep(.n-button:hover) {
  color: #fff !important;
  background: rgba(255, 255, 255, 0.1) !important;
}

.batch-spacer {
  flex: 1;
}
</style>
