<script setup lang="ts">
import { ref, computed, watch, h, onMounted, onUnmounted, type Component } from "vue";
import { useRouter } from "vue-router";
import { NTag, NIcon, NSelect, NInput, NButton, useMessage } from "naive-ui";
import {
  AntennaBars1,
  AntennaBars2,
  AntennaBars3,
  AntennaBars4,
  AntennaBars5,
  ChevronLeft,
  Trash,
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
  STATUS_COLORS,
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

const statusOptions = STATUSES.map((s) => ({ label: STATUS_DISPLAY[s], value: s }));
const priorityOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }));
const difficultyOptions = DIFFICULTIES.map((d) => ({ label: DIFFICULTY_DISPLAY[d], value: d }));
const labelOptions = LABELS.map((l) => ({ label: LABEL_DISPLAY[l], value: l }));

function renderStatusLabel(option: { label: string; value: string }) {
  const s = option.value as Status;
  return h("span", { style: "display: flex; align-items: center; gap: 8px" }, [
    h("span", {
      style: `width: 8px; height: 8px; border-radius: 50%; background: ${STATUS_COLORS[s]}; flex-shrink: 0`,
    }),
    option.label,
  ]);
}

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
  return h("span", { style: `display: flex; align-items: center; gap: 6px` }, [
    h("span", {
      style: `width: 8px; height: 8px; border-radius: 50%; background: ${LABEL_COLORS[l]}; flex-shrink: 0`,
    }),
    option.label,
  ]);
}

const assigneeOptions = computed(() => store.members.map((m) => ({ label: m.name, value: m.id })));

const todo = computed(() => store.todos.find((t) => t.number === props.number));

// Local editable copies
const title = ref("");
const description = ref("");

watch(
  () => props.number,
  () => {
    if (todo.value) {
      title.value = todo.value.title;
      description.value = todo.value.description;
    }
  },
  { immediate: true },
);

// Also sync when store data updates (e.g. from WebSocket)
watch(
  () => todo.value?.title,
  (newTitle) => {
    if (
      newTitle !== undefined &&
      document.activeElement?.classList.contains("inline-title") === false
    ) {
      title.value = newTitle;
    }
  },
);

watch(
  () => todo.value?.description,
  (newDesc) => {
    if (newDesc !== undefined) {
      description.value = newDesc;
    }
  },
);

async function saveField(field: string, value: unknown) {
  if (!todo.value) return;
  try {
    await store.updateTodo(todo.value.number, { [field]: value });
  } catch {
    message.error("Failed to update");
  }
}

function commitTitle() {
  const trimmed = title.value.trim();
  if (!todo.value || trimmed === todo.value.title) return;
  if (!trimmed) {
    title.value = todo.value.title;
    return;
  }
  saveField("title", trimmed);
}

function commitDescription() {
  const trimmed = description.value.trim();
  if (!todo.value || trimmed === todo.value.description) return;
  saveField("description", trimmed);
}

async function changeStatus(status: Status) {
  if (!todo.value) return;
  try {
    await store.moveTodo(todo.value.number, status);
  } catch {
    message.error("Failed to update status");
  }
}

async function changePriority(priority: Priority) {
  if (!todo.value || priority === todo.value.priority) return;
  saveField("priority", priority);
}

async function changeDifficulty(difficulty: Difficulty) {
  if (!todo.value || difficulty === todo.value.difficulty) return;
  saveField("difficulty", difficulty);
}

async function changeLabels(labels: Label[]) {
  if (!todo.value) return;
  saveField("labels", labels);
}

async function changeAssignee(assignee: string | null) {
  if (!todo.value || assignee === todo.value.assignee) return;
  saveField("assignee", assignee);
}

async function handleArchive() {
  if (!todo.value) return;
  try {
    await store.moveTodo(todo.value.number, "archived");
    message.success("Todo moved to trash");
    router.push({ name: "board" });
  } catch {
    message.error("Failed to archive todo");
  }
}

function formatDate(ts: number | string): string {
  return new Date(ts).toLocaleString();
}

function goBack() {
  router.back();
}

// ── Keyboard shortcuts (Linear-style) ──
const statusSelectRef = ref<InstanceType<typeof NSelect> | null>(null);
const prioritySelectRef = ref<InstanceType<typeof NSelect> | null>(null);
const difficultySelectRef = ref<InstanceType<typeof NSelect> | null>(null);
const statusPickerOpen = ref(false);
const priorityPickerOpen = ref(false);
const difficultyPickerOpen = ref(false);

const PRIORITY_KEYS: Record<string, Priority> = {
  "0": "none",
  "1": "urgent",
  "2": "high",
  "3": "medium",
  "4": "low",
};

const DIFFICULTY_KEYS: Record<string, Difficulty> = {
  "0": "none",
  "1": "easy",
  "2": "medium",
  "3": "hard",
};

const STATUS_KEYS: Record<string, Status> = {};
STATUSES.forEach((s, i) => {
  STATUS_KEYS[String(i)] = s;
});

function isTyping(): boolean {
  const el = document.activeElement as HTMLElement | null;
  if (!el) return false;
  const tag = el.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || !!el.isContentEditable;
}

function handleDetailKeydown(e: KeyboardEvent) {
  if (!todo.value) return;
  if (e.metaKey || e.ctrlKey || e.altKey) return;

  if (statusPickerOpen.value && STATUS_KEYS[e.key]) {
    e.preventDefault();
    changeStatus(STATUS_KEYS[e.key]);
    statusPickerOpen.value = false;
    return;
  }

  if (priorityPickerOpen.value && PRIORITY_KEYS[e.key]) {
    e.preventDefault();
    changePriority(PRIORITY_KEYS[e.key]);
    priorityPickerOpen.value = false;
    return;
  }

  if (difficultyPickerOpen.value && DIFFICULTY_KEYS[e.key]) {
    e.preventDefault();
    changeDifficulty(DIFFICULTY_KEYS[e.key]);
    difficultyPickerOpen.value = false;
    return;
  }

  if (isTyping()) return;

  if (e.key === "s") {
    e.preventDefault();
    statusPickerOpen.value = true;
  } else if (e.key === "p") {
    e.preventDefault();
    priorityPickerOpen.value = true;
  } else if (e.key === "d") {
    e.preventDefault();
    difficultyPickerOpen.value = true;
  }
}

onMounted(() => window.addEventListener("keydown", handleDetailKeydown));
onUnmounted(() => window.removeEventListener("keydown", handleDetailKeydown));

// ── Comments ──
const commentText = ref("");
const commentLoading = ref(false);

async function submitComment() {
  if (!todo.value || !commentText.value.trim()) return;
  commentLoading.value = true;
  try {
    await store.addComment(todo.value.number, commentText.value.trim());
    commentText.value = "";
  } catch (e: unknown) {
    message.error(e instanceof Error ? e.message : "Failed to add comment");
  } finally {
    commentLoading.value = false;
  }
}

// ── Branch ──
const branchLoading = ref(false);

async function handleCreateBranch() {
  if (!todo.value) return;
  branchLoading.value = true;
  try {
    const result = await store.createBranch(todo.value.number);
    if (result.alreadyExists) {
      message.info(`Branch already exists: ${result.branch}`);
    } else {
      message.success(`Created branch: ${result.branch}`);
    }
  } catch (e: unknown) {
    message.error(e instanceof Error ? e.message : "Failed to create branch");
  } finally {
    branchLoading.value = false;
  }
}

async function handleRemoveBranch() {
  if (!todo.value?.branch) return;
  branchLoading.value = true;
  try {
    await store.removeBranch(todo.value.number);
    message.success("Removed worktree + branch");
  } catch (e: unknown) {
    message.error(e instanceof Error ? e.message : "Failed to remove branch");
  } finally {
    branchLoading.value = false;
  }
}
</script>

<template>
  <div class="detail-page">
    <div class="detail-nav">
      <NButton size="small" quaternary @click="goBack">
        <template #icon>
          <NIcon :size="16"><ChevronLeft /></NIcon>
        </template>
        Back
      </NButton>
    </div>

    <div v-if="!todo" class="not-found">
      <p>Todo #{{ number }} not found.</p>
      <NButton @click="goBack">Go back</NButton>
    </div>

    <div v-else class="detail-content">
      <!-- Header: ref + archive -->
      <div class="detail-header">
        <NTag size="small" :bordered="false" style="font-family: monospace; flex-shrink: 0">
          {{ todo.ref }}
        </NTag>
        <NButton
          v-if="todo.status !== 'archived'"
          size="tiny"
          quaternary
          type="error"
          style="flex-shrink: 0"
          @click="handleArchive"
        >
          <template #icon>
            <NIcon :size="14"><Trash /></NIcon>
          </template>
        </NButton>
      </div>

      <input
        v-model="title"
        class="inline-title"
        @blur="commitTitle"
        @keydown.enter="($event.target as HTMLInputElement).blur()"
      />

      <!-- Meta row -->
      <div class="meta-grid">
        <div class="meta-item">
          <label class="meta-label">Status</label>
          <NSelect
            ref="statusSelectRef"
            :value="todo.status"
            :options="statusOptions"
            :render-label="renderStatusLabel"
            size="small"
            :show="statusPickerOpen"
            style="width: 160px"
            @update:value="changeStatus"
            @update:show="(v: boolean) => (statusPickerOpen = v)"
          />
        </div>

        <div class="meta-item">
          <label class="meta-label">Priority</label>
          <NSelect
            ref="prioritySelectRef"
            :value="todo.priority"
            :options="priorityOptions"
            :render-label="renderPriorityLabel"
            size="small"
            :show="priorityPickerOpen"
            style="width: 160px"
            @update:value="changePriority"
            @update:show="(v: boolean) => (priorityPickerOpen = v)"
          />
        </div>

        <div class="meta-item">
          <label class="meta-label">Difficulty</label>
          <NSelect
            ref="difficultySelectRef"
            :value="todo.difficulty"
            :options="difficultyOptions"
            :render-label="renderDifficultyLabel"
            size="small"
            :show="difficultyPickerOpen"
            style="width: 160px"
            @update:value="changeDifficulty"
            @update:show="(v: boolean) => (difficultyPickerOpen = v)"
          />
        </div>

        <div class="meta-item">
          <label class="meta-label">Assignee</label>
          <NSelect
            :value="todo.assignee"
            :options="assigneeOptions"
            size="small"
            clearable
            placeholder="Unassigned"
            style="width: 160px"
            @update:value="changeAssignee"
          />
        </div>

        <div class="meta-item">
          <label class="meta-label">Labels</label>
          <NSelect
            :value="todo.labels"
            :options="labelOptions"
            :render-label="renderLabelTag"
            size="small"
            multiple
            clearable
            placeholder="Add labels..."
            style="min-width: 200px"
            @update:value="changeLabels"
          />
        </div>

        <div class="meta-item">
          <label class="meta-label">Created</label>
          <span class="meta-value">{{ formatDate(todo.createdAt) }}</span>
        </div>

        <div class="meta-item">
          <label class="meta-label">Updated</label>
          <span class="meta-value">{{ formatDate(todo.updatedAt) }}</span>
        </div>
      </div>

      <!-- Branch -->
      <div class="meta-grid" style="margin-top: 8px">
        <div class="meta-item">
          <label class="meta-label">Branch</label>
          <div v-if="todo.branch" class="branch-name">
            <NTag size="small" :bordered="false" style="font-family: monospace">
              {{ todo.branch }}
            </NTag>
            <NButton
              size="tiny"
              quaternary
              type="error"
              :loading="branchLoading"
              @click="handleRemoveBranch"
            >
              Remove
            </NButton>
          </div>
          <NButton
            v-else
            size="small"
            secondary
            :loading="branchLoading"
            @click="handleCreateBranch"
          >
            Create Worktree + Branch
          </NButton>
        </div>
      </div>

      <!-- Description -->
      <div class="field-group">
        <label class="meta-label">Description</label>
        <NInput
          v-model:value="description"
          type="textarea"
          :autosize="{ minRows: 8, maxRows: 24 }"
          placeholder="Add a description..."
          @blur="commitDescription"
        />
      </div>

      <!-- Comments -->
      <div class="field-group">
        <label class="meta-label">Comments ({{ todo.comments?.length ?? 0 }})</label>
        <div v-if="todo.comments?.length" class="comments-list">
          <div v-for="c in todo.comments" :key="c.id" class="comment-item">
            <div class="comment-header">
              <strong>{{ c.authorName }}</strong>
              <span class="comment-date">{{ formatDate(c.createdAt) }}</span>
            </div>
            <div class="comment-text">{{ c.text }}</div>
          </div>
        </div>
        <div class="comment-input">
          <NInput
            v-model:value="commentText"
            type="textarea"
            :autosize="{ minRows: 2, maxRows: 6 }"
            placeholder="Write a comment..."
            @keydown.meta.enter="submitComment"
            @keydown.ctrl.enter="submitComment"
          />
          <NButton
            size="small"
            type="primary"
            :loading="commentLoading"
            :disabled="!commentText.trim()"
            style="align-self: flex-end; margin-top: 8px"
            @click="submitComment"
          >
            Comment
          </NButton>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.detail-page {
  max-width: 880px;
  margin: 0 auto;
  padding: 24px 32px;
  overflow-x: hidden;
}

.detail-nav {
  margin-bottom: 20px;
}

.not-found {
  text-align: center;
  padding: 40px;
  opacity: 0.5;
}

.not-found p {
  margin-bottom: 16px;
}

.detail-content {
  /* Full page — no height constraints like the modal */
}

.detail-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.inline-title {
  width: 100%;
  font-size: 22px;
  font-weight: 700;
  line-height: 1.3;
  border: none;
  outline: none;
  background: transparent;
  padding: 4px 0;
  margin-bottom: 16px;
  color: inherit;
  font-family: inherit;
  border-bottom: 2px solid transparent;
}

.inline-title:focus {
  border-bottom-color: var(--n-primary-color, #18a058);
}

.meta-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px 24px;
  margin-bottom: 16px;
}

.meta-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.meta-label {
  font-size: 11px;
  font-weight: 600;
  opacity: 0.5;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.meta-value {
  font-size: 13px;
  line-height: 28px;
}

.field-group {
  margin-bottom: 16px;
}

.field-group .meta-label {
  margin-bottom: 6px;
  display: block;
}

.comments-list {
  margin-bottom: 12px;
}

.comment-item {
  padding: 10px 12px;
  border: 1px solid rgba(128, 128, 128, 0.15);
  border-radius: 6px;
  margin-bottom: 8px;
}

.comment-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
  font-size: 13px;
}

.comment-date {
  font-size: 11px;
  opacity: 0.5;
}

.comment-text {
  font-size: 13px;
  white-space: pre-wrap;
}

.comment-input {
  display: flex;
  flex-direction: column;
}

.branch-name {
  display: flex;
  align-items: center;
  gap: 6px;
}
</style>
