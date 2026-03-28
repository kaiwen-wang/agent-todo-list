<script setup lang="ts">
import { ref, computed, h } from "vue";
import {
  NButton,
  NCard,
  NDataTable,
  NEmpty,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NPopconfirm,
  NSelect,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from "naive-ui";
import { useProjectStore } from "@/stores/project";
import type { Cycle, CycleStatus, Todo } from "@/types";
import {
  CYCLE_STATUS_COLORS,
  CYCLE_STATUS_DISPLAY,
  CYCLE_STATUSES,
  STATUS_DISPLAY,
  STATUS_COLORS,
} from "@/types";

const store = useProjectStore();
const message = useMessage();

const showAddModal = ref(false);
const showEditModal = ref(false);
const selectedSprint = ref<Cycle | null>(null);
const editingSprint = ref<Cycle | null>(null);

// Add form — simplified: just pick a length
const addLength = ref("1w");
const addSubmitting = ref(false);

const lengthOptions = [
  { label: "1 week", value: "1w" },
  { label: "2 weeks", value: "2w" },
  { label: "3 weeks", value: "3w" },
  { label: "4 weeks", value: "4w" },
  { label: "6 weeks", value: "6w" },
];

function nextSprintName() {
  return `Sprint ${store.cycles.length + 1}`;
}

function addDays(date: Date, days: number): string {
  const d = new Date(date);
  d.setDate(d.getDate() + days);
  return d.toISOString().slice(0, 10);
}

function lengthToDays(len: string): number {
  const weeks = parseInt(len);
  return weeks * 7;
}

// Edit form
const editName = ref("");
const editDescription = ref("");
const editStatus = ref<CycleStatus>("planning");
const editStartDate = ref("");
const editEndDate = ref("");
const editSubmitting = ref(false);

const statusOptions = CYCLE_STATUSES.map((s) => ({
  label: CYCLE_STATUS_DISPLAY[s],
  value: s,
}));

const columns: DataTableColumns<Cycle> = [
  {
    title: "Name",
    key: "name",
    render(row) {
      return h("span", { style: "font-weight: 600" }, row.name);
    },
  },
  {
    title: "Status",
    key: "status",
    width: 120,
    render(row) {
      const color = CYCLE_STATUS_COLORS[row.status];
      return h(
        NTag,
        {
          size: "small",
          round: true,
          bordered: false,
          color: { color: color + "22", textColor: color },
        },
        () => CYCLE_STATUS_DISPLAY[row.status],
      );
    },
  },
  {
    title: "Todos",
    key: "todos",
    width: 80,
    render(row) {
      const count = store.todos.filter((t) => t.cycleId === row.id).length;
      return count > 0 ? String(count) : "";
    },
  },
  {
    title: "Dates",
    key: "dates",
    width: 200,
    render(row) {
      if (!row.startDate && !row.endDate) return "";
      const parts: string[] = [];
      if (row.startDate) parts.push(row.startDate);
      parts.push("\u2192");
      if (row.endDate) parts.push(row.endDate);
      return h("span", { style: "font-size: 12px; opacity: 0.6" }, parts.join(" "));
    },
  },
  {
    title: "Description",
    key: "description",
    ellipsis: { tooltip: true },
    render(row) {
      return row.description
        ? h("span", { style: "font-size: 12px; opacity: 0.6" }, row.description)
        : "";
    },
  },
];

const sprintTodos = computed(() => {
  if (!selectedSprint.value) return [];
  return store.todos.filter(
    (t) =>
      t.cycleId === selectedSprint.value!.id && t.status !== "archived" && t.status !== "wont_do",
  );
});

const sprintTodoColumns: DataTableColumns<Todo> = [
  {
    title: "Ref",
    key: "ref",
    width: 70,
    render(row) {
      return h("span", { style: "font-family: monospace; font-size: 11px; opacity: 0.5" }, row.ref);
    },
  },
  {
    title: "Title",
    key: "title",
    ellipsis: { tooltip: true },
  },
  {
    title: "Status",
    key: "status",
    width: 110,
    render(row) {
      return h("span", { style: "display: flex; align-items: center; gap: 6px; font-size: 12px" }, [
        h("span", {
          style: `width: 8px; height: 8px; border-radius: 50%; background: ${STATUS_COLORS[row.status ?? "none"]}; flex-shrink: 0`,
        }),
        STATUS_DISPLAY[row.status ?? "none"],
      ]);
    },
  },
];

function selectSprint(sprint: Cycle) {
  selectedSprint.value = sprint;
}

const rowProps = (row: Cycle) => ({
  style: "cursor: pointer",
  onClick: () => selectSprint(row),
});

function openTodo(row: Todo) {
  selectedSprint.value = null;
  store.openTodo(row.number);
}

const todoRowProps = (row: Todo) => ({
  style: "cursor: pointer",
  onClick: () => openTodo(row),
});

function openAdd() {
  addLength.value = "1w";
  showAddModal.value = true;
}

function openEdit(sprint: Cycle) {
  editingSprint.value = sprint;
  editName.value = sprint.name;
  editDescription.value = sprint.description;
  editStatus.value = sprint.status;
  editStartDate.value = sprint.startDate ?? "";
  editEndDate.value = sprint.endDate ?? "";
  showEditModal.value = true;
}

async function handleAdd() {
  addSubmitting.value = true;
  try {
    const today = new Date();
    const startDate = today.toISOString().slice(0, 10);
    const endDate = addDays(today, lengthToDays(addLength.value));
    await store.addCycle({
      name: nextSprintName(),
      status: "active",
      startDate,
      endDate,
    });
    message.success("Sprint created");
    showAddModal.value = false;
  } catch {
    message.error("Failed to create sprint");
  } finally {
    addSubmitting.value = false;
  }
}

async function handleEdit() {
  if (!editingSprint.value || !editName.value.trim()) return;
  editSubmitting.value = true;
  try {
    const updates: Record<string, string | null> = {};
    if (editName.value.trim() !== editingSprint.value.name) {
      updates.name = editName.value.trim();
    }
    if (editDescription.value.trim() !== editingSprint.value.description) {
      updates.description = editDescription.value.trim();
    }
    if (editStatus.value !== editingSprint.value.status) {
      updates.status = editStatus.value;
    }
    const newStart = editStartDate.value || null;
    if (newStart !== editingSprint.value.startDate) {
      updates.startDate = newStart;
    }
    const newEnd = editEndDate.value || null;
    if (newEnd !== editingSprint.value.endDate) {
      updates.endDate = newEnd;
    }
    if (Object.keys(updates).length > 0) {
      await store.updateCycle(editingSprint.value.id, updates);
      message.success("Sprint updated");
    }
    showEditModal.value = false;
  } catch {
    message.error("Failed to update sprint");
  } finally {
    editSubmitting.value = false;
  }
}

async function handleDelete(sprint: Cycle) {
  try {
    await store.deleteCycle(sprint.id);
    message.success(`Deleted ${sprint.name}`);
    if (selectedSprint.value?.id === sprint.id) {
      selectedSprint.value = null;
    }
  } catch {
    message.error("Failed to delete sprint");
  }
}
</script>

<template>
  <div class="sprints-view">
    <div class="sprints-toolbar">
      <h2>Sprints</h2>
      <NButton type="primary" size="small" @click="openAdd">+ New Sprint</NButton>
    </div>

    <div class="sprints-table-container">
      <NDataTable
        v-if="store.cycles.length"
        :columns="columns"
        :data="store.cycles"
        :row-props="rowProps"
        :bordered="false"
        size="small"
      />
      <NEmpty v-else description="No sprints yet" style="padding: 48px 0">
        <template #extra>
          <NButton size="small" @click="openAdd">Create your first sprint</NButton>
        </template>
      </NEmpty>
    </div>

    <!-- Sprint Detail Modal -->
    <NModal :show="!!selectedSprint" @update:show="(v: boolean) => !v && (selectedSprint = null)">
      <NCard
        v-if="selectedSprint"
        :bordered="true"
        closable
        @close="selectedSprint = null"
        style="width: 640px; max-width: 95vw"
        role="dialog"
      >
        <template #header>
          <div class="sprint-detail-header">
            <div>
              <div class="sprint-detail-name">{{ selectedSprint.name }}</div>
              <div class="sprint-detail-meta">
                <NTag
                  size="small"
                  round
                  :bordered="false"
                  :color="{
                    color: CYCLE_STATUS_COLORS[selectedSprint.status] + '22',
                    textColor: CYCLE_STATUS_COLORS[selectedSprint.status],
                  }"
                  >{{ CYCLE_STATUS_DISPLAY[selectedSprint.status] }}</NTag
                >
                <span
                  v-if="selectedSprint.startDate || selectedSprint.endDate"
                  class="sprint-detail-dates"
                >
                  {{ selectedSprint.startDate ?? "?" }} &rarr; {{ selectedSprint.endDate ?? "?" }}
                </span>
              </div>
            </div>
          </div>
        </template>

        <p v-if="selectedSprint.description" class="sprint-detail-desc">
          {{ selectedSprint.description }}
        </p>

        <div class="sprint-detail-actions">
          <NButton size="small" quaternary @click="openEdit(selectedSprint)">Edit</NButton>
          <NPopconfirm @positive-click="handleDelete(selectedSprint)">
            <template #trigger>
              <NButton size="small" quaternary type="error">Delete</NButton>
            </template>
            Delete {{ selectedSprint.name }}? Todos will be unassigned from this sprint.
          </NPopconfirm>
        </div>

        <div class="sprint-detail-section">
          <div class="section-title">Todos ({{ sprintTodos.length }})</div>
          <NDataTable
            v-if="sprintTodos.length"
            :columns="sprintTodoColumns"
            :data="sprintTodos"
            :row-props="todoRowProps"
            :bordered="false"
            size="small"
          />
          <NEmpty
            v-else
            description="No todos in this sprint"
            size="small"
            style="padding: 24px 0"
          />
        </div>
      </NCard>
    </NModal>

    <!-- Add Sprint Modal -->
    <NModal :show="showAddModal" @update:show="(v: boolean) => !v && (showAddModal = false)">
      <NCard
        title="New Sprint"
        :bordered="true"
        closable
        @close="showAddModal = false"
        style="width: 360px; max-width: 95vw"
        role="dialog"
      >
        <NForm @submit.prevent="handleAdd" label-placement="top">
          <NFormItem label="Length">
            <NSelect v-model:value="addLength" :options="lengthOptions" />
          </NFormItem>
          <div class="add-preview">
            Starts today, ends {{ addDays(new Date(), lengthToDays(addLength)) }}
          </div>
          <NSpace justify="end" :size="8" style="margin-top: 16px">
            <NButton @click="showAddModal = false">Cancel</NButton>
            <NButton type="primary" @click="handleAdd" :loading="addSubmitting">
              Create Sprint
            </NButton>
          </NSpace>
        </NForm>
      </NCard>
    </NModal>

    <!-- Edit Sprint Modal -->
    <NModal :show="showEditModal" @update:show="(v: boolean) => !v && (showEditModal = false)">
      <NCard
        title="Edit Sprint"
        :bordered="true"
        closable
        @close="showEditModal = false"
        style="width: 440px; max-width: 95vw"
        role="dialog"
      >
        <NForm @submit.prevent="handleEdit" label-placement="top">
          <NFormItem label="Name">
            <NInput
              v-model:value="editName"
              placeholder="Sprint name"
              autofocus
              @keydown.enter.prevent="handleEdit"
            />
          </NFormItem>
          <NFormItem label="Description">
            <NInput
              v-model:value="editDescription"
              type="textarea"
              placeholder="Optional description"
              :autosize="{ minRows: 2, maxRows: 4 }"
            />
          </NFormItem>
          <NFormItem label="Status">
            <NSelect v-model:value="editStatus" :options="statusOptions" />
          </NFormItem>
          <div class="date-row">
            <NFormItem label="Start Date">
              <NInput v-model:value="editStartDate" placeholder="YYYY-MM-DD" />
            </NFormItem>
            <NFormItem label="End Date">
              <NInput v-model:value="editEndDate" placeholder="YYYY-MM-DD" />
            </NFormItem>
          </div>
          <NSpace justify="end" :size="8">
            <NButton @click="showEditModal = false">Cancel</NButton>
            <NButton
              type="primary"
              @click="handleEdit"
              :loading="editSubmitting"
              :disabled="!editName.trim()"
            >
              Save Changes
            </NButton>
          </NSpace>
        </NForm>
      </NCard>
    </NModal>
  </div>
</template>

<style scoped>
.sprints-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sprints-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  flex-shrink: 0;
}

.sprints-toolbar h2 {
  font-size: 16px;
  font-weight: 600;
}

.sprints-table-container {
  flex: 1;
  padding: 0 24px 24px;
  overflow: auto;
}

.add-preview {
  font-size: 12px;
  opacity: 0.5;
  margin-top: -8px;
}

.date-row {
  display: flex;
  gap: 12px;
}

.date-row .n-form-item {
  flex: 1;
}

.sprint-detail-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sprint-detail-name {
  font-size: 16px;
  font-weight: 700;
}

.sprint-detail-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.sprint-detail-dates {
  font-size: 12px;
  opacity: 0.5;
}

.sprint-detail-desc {
  font-size: 13px;
  opacity: 0.7;
  margin-bottom: 12px;
}

.sprint-detail-actions {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
}

.sprint-detail-section {
  margin-top: 4px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  opacity: 0.45;
  margin-bottom: 8px;
}
</style>
