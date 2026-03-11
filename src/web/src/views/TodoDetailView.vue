<script setup lang="ts">
import { ref, computed, h, type Component } from "vue";
import { useRouter } from "vue-router";
import {
  NButton,
  NIcon,
  NSpace,
  NCard,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NDescriptions,
  NDescriptionsItem,
  NPopconfirm,
  NButtonGroup,
  useMessage,
} from "naive-ui";
import {
  AntennaBars1,
  AntennaBars2,
  AntennaBars3,
  AntennaBars4,
  AntennaBars5,
} from "@vicons/tabler";
import { useProjectStore } from "@/stores/project";
import type { Status, Priority, Difficulty, Label } from "@/types";
import {
  STATUSES,
  PRIORITIES,
  DIFFICULTIES,
  LABELS,
  STATUS_DISPLAY,
  PRIORITY_DISPLAY,
  PRIORITY_COLORS,
  DIFFICULTY_DISPLAY,
  DIFFICULTY_COLORS,
  LABEL_DISPLAY,
  LABEL_COLORS,
} from "@/types";

const PRIORITY_ICON: Record<Priority, Component> = {
  none: AntennaBars1,
  low: AntennaBars2,
  medium: AntennaBars3,
  high: AntennaBars4,
  urgent: AntennaBars5,
};

const props = defineProps<{
  number: number;
}>();

const store = useProjectStore();
const router = useRouter();
const message = useMessage();

const editing = ref(false);
const editTitle = ref("");
const editDescription = ref("");
const editPriority = ref<Priority | null>(null);
const editDifficulty = ref<Difficulty | null>(null);
const editStatus = ref<Status | null>(null);
const editLabels = ref<Label[]>([]);
const editAssignee = ref<string | null>(null);
const saving = ref(false);

const memberOptions = computed(() => store.members.map((m) => ({ label: m.name, value: m.id })));

const statusOptions = STATUSES.map((s) => ({ label: STATUS_DISPLAY[s], value: s }));
const priorityOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }));
const difficultyOptions = DIFFICULTIES.map((d) => ({ label: DIFFICULTY_DISPLAY[d], value: d }));

const labelOptions = LABELS.map((l) => ({ label: LABEL_DISPLAY[l], value: l }));

function renderPriorityLabel(option: { label: string; value: string }) {
  const p = option.value as Priority;
  return h("span", { style: "display: flex; align-items: center; gap: 8px" }, [
    h(NIcon, { size: 16, color: PRIORITY_COLORS[p] }, { default: () => h(PRIORITY_ICON[p]) }),
    option.label,
  ]);
}

function renderDifficultyLabel(option: { label: string; value: string }) {
  const d = option.value as Difficulty;
  return h("span", { style: "display: flex; align-items: center; gap: 8px" }, [
    h("span", {
      style: `width: 8px; height: 8px; border-radius: 50%; background: ${DIFFICULTY_COLORS[d]}; flex-shrink: 0`,
    }),
    option.label,
  ]);
}

function renderLabelTag(option: { label: string; value: string }) {
  const l = option.value as Label;
  return h("span", { style: "display: flex; align-items: center; gap: 6px" }, [
    h("span", {
      style: `width: 8px; height: 8px; border-radius: 50%; background: ${LABEL_COLORS[l]}; flex-shrink: 0`,
    }),
    option.label,
  ]);
}

const todo = computed(() => store.todos.find((t) => t.number === props.number));

function startEdit() {
  if (!todo.value) return;
  editTitle.value = todo.value.title;
  editDescription.value = todo.value.description;
  editPriority.value = todo.value.priority;
  editDifficulty.value = todo.value.difficulty;
  editStatus.value = todo.value.status;
  editLabels.value = [...(todo.value.labels ?? [])];
  editAssignee.value = todo.value.assignee;
  editing.value = true;
}

async function saveEdit() {
  if (!todo.value) return;
  saving.value = true;
  try {
    await store.updateTodo(todo.value.number, {
      title: editTitle.value.trim(),
      description: editDescription.value.trim(),
      priority: editPriority.value || undefined,
      difficulty: editDifficulty.value || undefined,
      status: editStatus.value || undefined,
      labels: editLabels.value,
      assignee: editAssignee.value,
    });
    editing.value = false;
    message.success("Todo updated");
  } catch {
    message.error("Failed to update todo");
  } finally {
    saving.value = false;
  }
}

function cancelEdit() {
  editing.value = false;
}

async function changeStatus(status: Status) {
  if (!todo.value) return;
  try {
    await store.moveTodo(todo.value.number, status);
  } catch {
    message.error("Failed to update status");
  }
}

async function handleDelete() {
  if (!todo.value) return;
  try {
    await store.deleteTodo(todo.value.number);
    message.success("Todo deleted");
    router.back();
  } catch {
    message.error("Failed to delete todo");
  }
}

function goBack() {
  router.back();
}

function formatDate(iso: string | number): string {
  return new Date(iso).toLocaleString();
}
</script>

<template>
  <div class="detail-view">
    <div class="detail-nav">
      <NButton size="small" @click="goBack">&larr; Back</NButton>
    </div>

    <div v-if="!todo" class="not-found">
      <p>Todo #{{ number }} not found.</p>
      <NButton @click="goBack">Go back</NButton>
    </div>

    <div v-else-if="!editing">
      <!-- View mode -->
      <div class="detail-header">
        <div class="detail-header-left">
          <NTag size="small" :bordered="false" style="font-family: monospace">
            {{ todo.ref }}
          </NTag>
          <h1 class="detail-title">{{ todo.title }}</h1>
        </div>
        <NSpace :size="8">
          <NButton size="small" @click="startEdit">Edit</NButton>
          <NPopconfirm @positive-click="handleDelete">
            <template #trigger>
              <NButton size="small" type="error" ghost>Delete</NButton>
            </template>
            Delete this todo permanently?
          </NPopconfirm>
        </NSpace>
      </div>

      <!-- Status buttons -->
      <NCard size="small" class="detail-meta">
        <div class="meta-section">
          <label class="meta-label">Status</label>
          <NButtonGroup size="tiny">
            <NButton
              v-for="s in STATUSES"
              :key="s"
              :type="todo.status === s ? 'primary' : 'default'"
              :ghost="todo.status !== s"
              @click="changeStatus(s)"
            >
              {{ STATUS_DISPLAY[s] }}
            </NButton>
          </NButtonGroup>
        </div>

        <NDescriptions :column="4" label-placement="top" size="small" class="meta-descriptions">
          <NDescriptionsItem label="Priority">
            <NSpace :size="6" align="center">
              <NIcon :size="18" :color="PRIORITY_COLORS[todo.priority]">
                <component :is="PRIORITY_ICON[todo.priority]" />
              </NIcon>
              {{ PRIORITY_DISPLAY[todo.priority] }}
            </NSpace>
          </NDescriptionsItem>
          <NDescriptionsItem label="Difficulty">
            <NSpace :size="6" align="center">
              <span
                :style="{
                  width: '8px',
                  height: '8px',
                  borderRadius: '50%',
                  background: DIFFICULTY_COLORS[todo.difficulty || 'none'],
                  flexShrink: 0,
                  display: 'inline-block',
                }"
              />
              {{ DIFFICULTY_DISPLAY[todo.difficulty || "none"] }}
            </NSpace>
          </NDescriptionsItem>
          <NDescriptionsItem label="Labels">
            <NSpace v-if="todo.labels?.length" :size="4">
              <span
                v-for="l in todo.labels"
                :key="l"
                :style="{
                  fontSize: '11px',
                  fontWeight: 600,
                  padding: '1px 8px',
                  borderRadius: '8px',
                  background: LABEL_COLORS[l] + '22',
                  color: LABEL_COLORS[l],
                }"
                >{{ LABEL_DISPLAY[l] }}</span
              >
            </NSpace>
            <span v-else style="opacity: 0.35">None</span>
          </NDescriptionsItem>
          <NDescriptionsItem label="Assignee">
            {{ todo.assigneeName || "Unassigned" }}
          </NDescriptionsItem>
          <NDescriptionsItem label="Created">
            {{ formatDate(todo.createdAt) }}
          </NDescriptionsItem>
          <NDescriptionsItem label="Updated">
            {{ formatDate(todo.updatedAt) }}
          </NDescriptionsItem>
        </NDescriptions>
      </NCard>

      <!-- Description -->
      <NCard size="small" title="Description" class="detail-description">
        <div v-if="todo.description" class="description-body">{{ todo.description }}</div>
        <div v-else class="description-empty">No description provided.</div>
      </NCard>
    </div>

    <!-- Edit mode -->
    <NCard v-else title="Edit Todo" size="small">
      <NForm label-placement="top" @submit.prevent="saveEdit">
        <NFormItem label="Title">
          <NInput v-model:value="editTitle" />
        </NFormItem>

        <NFormItem label="Description">
          <NInput v-model:value="editDescription" type="textarea" :rows="5" />
        </NFormItem>

        <NSpace :size="12">
          <NFormItem label="Status" style="flex: 1">
            <NSelect v-model:value="editStatus" :options="statusOptions" />
          </NFormItem>
          <NFormItem label="Priority" style="flex: 1">
            <NSelect
              v-model:value="editPriority"
              :options="priorityOptions"
              :render-label="renderPriorityLabel"
            />
          </NFormItem>
          <NFormItem label="Difficulty" style="flex: 1">
            <NSelect
              v-model:value="editDifficulty"
              :options="difficultyOptions"
              :render-label="renderDifficultyLabel"
            />
          </NFormItem>
        </NSpace>

        <NFormItem label="Labels">
          <NSelect
            v-model:value="editLabels"
            :options="labelOptions"
            :render-label="renderLabelTag"
            multiple
            clearable
            placeholder="Add labels..."
          />
        </NFormItem>

        <NFormItem label="Assignee">
          <NSelect
            v-model:value="editAssignee"
            :options="memberOptions"
            placeholder="Assign to a member..."
            clearable
          />
        </NFormItem>

        <NSpace justify="end" :size="8">
          <NButton @click="cancelEdit">Cancel</NButton>
          <NButton type="primary" @click="saveEdit" :loading="saving">Save Changes</NButton>
        </NSpace>
      </NForm>
    </NCard>
  </div>
</template>

<style scoped>
.detail-view {
  max-width: 800px;
  margin: 0 auto;
  padding: 24px;
}

.detail-nav {
  margin-bottom: 16px;
}

.not-found {
  text-align: center;
  padding: 40px;
  opacity: 0.5;
}

.not-found p {
  margin-bottom: 16px;
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 20px;
  gap: 16px;
}

.detail-header-left {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.detail-title {
  font-size: 22px;
  font-weight: 700;
  line-height: 1.3;
}

.detail-meta {
  margin-bottom: 16px;
}

.meta-section {
  margin-bottom: 12px;
}

.meta-section:last-child {
  margin-bottom: 0;
}

.meta-label {
  display: block;
  font-size: 11px;
  font-weight: 600;
  opacity: 0.5;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 6px;
}

.meta-descriptions {
  margin-top: 12px;
}

.detail-description {
  margin-top: 16px;
}

.description-body {
  font-size: 14px;
  line-height: 1.6;
  white-space: pre-wrap;
}

.description-empty {
  font-size: 13px;
  opacity: 0.35;
  font-style: italic;
}
</style>
