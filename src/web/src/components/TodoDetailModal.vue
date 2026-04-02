<script setup lang="ts">
import { ref, computed, watch, h, nextTick, onMounted, onUnmounted, type Component } from "vue";
import { useRouter } from "vue-router";
import {
  NModal,
  NCard,
  NTag,
  NIcon,
  NSelect,
  NInput,
  NButton,
  NTabs,
  NTabPane,
  useMessage,
} from "naive-ui";
import {
  AntennaBars1,
  AntennaBars2,
  AntennaBars3,
  AntennaBars4,
  AntennaBars5,
  ExternalLink,
  FileText,
  GitBranch,
  GitCommit,
  Trash,
  CalendarEvent,
  MessageCircle,
  X,
} from "@vicons/tabler";
import { marked } from "marked";
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

function getInitials(name: string): string {
  return name
    .split(/\s+/)
    .map((w) => w[0])
    .join("")
    .toUpperCase()
    .slice(0, 2);
}

function renderAssigneeLabel(option: { label: string; value: string }) {
  return h("span", { style: "display: flex; align-items: center; gap: 8px" }, [
    h(
      "span",
      {
        style:
          "width: 18px; height: 18px; border-radius: 50%; background: rgba(128,128,128,0.15); display: flex; align-items: center; justify-content: center; font-size: 9px; font-weight: 600; flex-shrink: 0; line-height: 1",
      },
      getInitials(option.label),
    ),
    option.label,
  ]);
}
const sprintOptions = computed(() => store.cycles.map((c) => ({ label: c.name, value: c.id })));

function renderSprintLabel(option: { label: string; value: string }) {
  return h("span", { style: "display: flex; align-items: center; gap: 8px" }, [
    h(
      NIcon,
      { size: 14, style: "opacity: 0.5; flex-shrink: 0" },
      { default: () => h(CalendarEvent) },
    ),
    option.label,
  ]);
}

const isOpen = computed(() => store.selectedTodoNumber !== null);
const todo = computed(() =>
  store.selectedTodoNumber !== null
    ? store.todos.find((t) => t.number === store.selectedTodoNumber)
    : undefined,
);

// Local editable copies
const title = ref("");
const description = ref("");
const commitShaInput = ref("");
const showCommitInput = ref(false);
const branchLoading = ref(false);
const activeTab = ref("details");
// Sync local state when a different todo is opened
watch(
  () => store.selectedTodoNumber,
  () => {
    if (todo.value) {
      title.value = todo.value.title;
      description.value = todo.value.description;
      activeTab.value = "details";
      showCommitInput.value = false;
    }
  },
);

function close() {
  store.closeTodo();
}

function openFullPage() {
  if (!todo.value) return;
  const num = todo.value.number;
  store.closeTodo();
  router.push({ name: "todo-detail", params: { number: num } });
}

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

async function changeSprint(cycleId: string | null) {
  if (!todo.value || cycleId === todo.value.cycleId) return;
  saveField("cycleId", cycleId);
}

async function submitCommitLink() {
  const sha = commitShaInput.value.trim();
  if (!todo.value || !sha) return;
  try {
    await store.linkCommit(todo.value.number, sha);
    commitShaInput.value = "";
    showCommitInput.value = false;
    message.success(`Linked ${sha.slice(0, 8)}`);
  } catch {
    message.error("Failed to link commit");
  }
}

async function handleCreateBranchOnly() {
  if (!todo.value) return;
  branchLoading.value = true;
  try {
    const result = await store.createBranchOnly(todo.value.number);
    message.success(`Branch: ${result.branch}`);
  } catch {
    message.error("Failed to create branch");
  } finally {
    branchLoading.value = false;
  }
}

async function handleCreateBranch() {
  if (!todo.value) return;
  branchLoading.value = true;
  try {
    const result = await store.createBranch(todo.value.number);
    message.success(`Branch: ${result.branch}`);
  } catch {
    message.error("Failed to create branch & worktree");
  } finally {
    branchLoading.value = false;
  }
}

async function handleRemoveBranch() {
  if (!todo.value) return;
  try {
    await store.removeBranch(todo.value.number);
    message.success("Branch removed");
  } catch {
    message.error("Failed to remove branch");
  }
}

async function handleArchive() {
  if (!todo.value) return;
  try {
    await store.moveTodo(todo.value.number, "archived");
    message.success("Todo moved to trash");
    close();
  } catch {
    message.error("Failed to archive todo");
  }
}

async function handleDelete() {
  if (!todo.value) return;
  try {
    await store.deleteTodo(todo.value.number);
    message.success("Todo permanently deleted");
    close();
  } catch {
    message.error("Failed to delete todo");
  }
}

function formatDate(ts: number | string): string {
  return new Date(ts).toLocaleString();
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

// Map number keys to statuses: 0=None, 1=Todo, 2=Needs Elaboration, ...
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
  if (!isOpen.value || !todo.value) return;
  if (e.metaKey || e.ctrlKey || e.altKey) return;

  // When status picker is open, number keys select a status
  if (statusPickerOpen.value && STATUS_KEYS[e.key]) {
    e.preventDefault();
    changeStatus(STATUS_KEYS[e.key]!);
    statusPickerOpen.value = false;
    return;
  }

  // When priority picker is open, number keys select a priority
  if (priorityPickerOpen.value && PRIORITY_KEYS[e.key]) {
    e.preventDefault();
    changePriority(PRIORITY_KEYS[e.key]!);
    priorityPickerOpen.value = false;
    return;
  }

  // When difficulty picker is open, number keys select a difficulty
  if (difficultyPickerOpen.value && DIFFICULTY_KEYS[e.key]) {
    e.preventDefault();
    changeDifficulty(DIFFICULTY_KEYS[e.key]!);
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

// ── Plan ──
const planContent = ref<string | null>(null);
const planExists = ref(false);
const planLoading = ref(false);
const planResearching = ref(false);
const planProgressMsg = ref("");
const planAnswerText = ref("");
const planAnswerLoading = ref(false);
const renderedPlan = computed(() => {
  if (!planContent.value) return "";
  return marked.parse(planContent.value) as string;
});

async function loadPlan() {
  if (!todo.value) return;
  try {
    const result = await store.fetchPlan(todo.value.number);
    planContent.value = result.content;
    planExists.value = result.exists;
  } catch {
    // silently fail
  }
}

async function handleResearchPlan() {
  if (!todo.value) return;
  planLoading.value = true;
  planResearching.value = true;
  planProgressMsg.value = "Starting research...";
  try {
    await store.researchPlan(todo.value.number);
    planExists.value = true;
  } catch (e: unknown) {
    message.error(e instanceof Error ? e.message : "Failed to start research");
    planResearching.value = false;
    planProgressMsg.value = "";
  } finally {
    planLoading.value = false;
  }
}

// Watch for plan WebSocket events
watch(
  () => store.planEvents.length,
  () => {
    if (!todo.value) return;
    const events = store.planEvents;
    for (const evt of events) {
      const evtNumber = evt.number as number | undefined;
      if (evtNumber !== undefined && evtNumber !== todo.value.number) continue;

      if (evt.type === "plan:progress") {
        planProgressMsg.value = (evt.message as string) || "Working...";
      } else if (evt.type === "plan:done") {
        planResearching.value = false;
        planProgressMsg.value = "";
        planContent.value = (evt.content as string) || null;
        planExists.value = true;
      } else if (evt.type === "plan:error") {
        planResearching.value = false;
        planProgressMsg.value = "";
        message.error((evt.message as string) || "Research failed");
      }
    }
    // Clear processed events
    store.planEvents.splice(0);
  },
);

async function submitPlanAnswer() {
  if (!todo.value || !planAnswerText.value.trim()) return;
  planAnswerLoading.value = true;
  try {
    await store.answerPlan(todo.value.number, planAnswerText.value.trim());
    planAnswerText.value = "";
    await loadPlan();
  } catch (e: unknown) {
    message.error(e instanceof Error ? e.message : "Failed to add answer");
  } finally {
    planAnswerLoading.value = false;
  }
}

// Load plan when modal opens for a different todo
watch(
  () => store.selectedTodoNumber,
  (num) => {
    if (num !== null) loadPlan();
    else {
      planContent.value = null;
      planExists.value = false;
    }
  },
);

// Reload plan if planPath changes (agent updated it)
watch(
  () => todo.value?.planPath,
  () => {
    if (todo.value) loadPlan();
  },
);

// ── Comments ──
const commentText = ref("");
const replyingTo = ref<string | null>(null);
const replyText = ref("");
const replyInputRef = ref<InstanceType<typeof NInput> | null>(null);

function startReply(commentId: string) {
  if (replyingTo.value === commentId) {
    replyingTo.value = null;
    replyText.value = "";
  } else {
    replyingTo.value = commentId;
    replyText.value = "";
    nextTick(() => {
      const el = replyInputRef.value;
      const input = Array.isArray(el) ? el[0] : el;
      input?.focus();
    });
  }
}

function submitComment() {
  if (!todo.value || !commentText.value.trim()) return;
  const text = commentText.value.trim();
  commentText.value = "";
  store.addComment(todo.value.number, text).catch((e: unknown) => {
    message.error(e instanceof Error ? e.message : "Failed to add comment");
  });
}

function submitReply() {
  if (!todo.value || !replyText.value.trim() || !replyingTo.value) return;
  const text = replyText.value.trim();
  const parentId = replyingTo.value;
  replyText.value = "";
  replyingTo.value = null;
  store.addComment(todo.value.number, text, parentId).catch((e: unknown) => {
    message.error(e instanceof Error ? e.message : "Failed to add reply");
  });
}

interface CommentWithDepth {
  id: string;
  author: string;
  authorName: string;
  text: string;
  createdAt: number;
  parentId?: string | null;
  depth: number;
}

/** Flatten comments into thread order with depth for indentation. */
function threadedComments(): CommentWithDepth[] {
  const comments = todo.value?.comments ?? [];
  const result: CommentWithDepth[] = [];
  const childrenOf = new Map<string, typeof comments>();
  const roots: typeof comments = [];

  for (const c of comments) {
    if (!c.parentId) {
      roots.push(c);
    } else {
      const arr = childrenOf.get(c.parentId) ?? [];
      arr.push(c);
      childrenOf.set(c.parentId, arr);
    }
  }

  function walk(comment: (typeof comments)[0], depth: number) {
    result.push({ ...comment, depth });
    const children = childrenOf.get(comment.id) ?? [];
    children.sort((a, b) => a.createdAt - b.createdAt);
    for (const child of children) walk(child, depth + 1);
  }

  roots.sort((a, b) => a.createdAt - b.createdAt);
  for (const root of roots) walk(root, 0);
  return result;
}
</script>

<template>
  <NModal :show="isOpen" @update:show="(v: boolean) => !v && close()">
    <NCard
      :bordered="true"
      style="width: 960px; max-width: 95vw; height: 85vh"
      role="dialog"
      class="detail-card"
    >
      <div v-if="!todo" class="not-found">
        <p>Todo not found.</p>
      </div>

      <div v-else class="detail-wrapper">
        <!-- Header: ref + outlink + title + trash -->
        <div class="detail-header">
          <button class="header-icon-btn" title="Open full page" @click="openFullPage">
            <NIcon :size="18"><ExternalLink /></NIcon>
          </button>
          <NTag size="small" :bordered="false" class="ref-tag">
            {{ todo.ref }}
          </NTag>
          <input
            v-model="title"
            class="inline-title"
            @blur="commitTitle"
            @keydown.enter="($event.target as HTMLInputElement).blur()"
          />
          <button
            v-if="todo.status !== 'archived'"
            class="header-icon-btn header-icon-btn-danger"
            title="Archive"
            @click="handleArchive"
          >
            <NIcon :size="18"><Trash /></NIcon>
          </button>
          <button
            v-else
            class="header-icon-btn header-icon-btn-danger"
            title="Delete permanently"
            @click="handleDelete"
          >
            <NIcon :size="18"><Trash /></NIcon>
          </button>
        </div>

        <!-- Property pills bar -->
        <div class="prop-bar">
          <div class="prop-row">
            <div class="prop-pill">
              <label class="prop-label">Status</label>
              <NSelect
                ref="statusSelectRef"
                :value="todo.status"
                :options="statusOptions"
                :render-label="renderStatusLabel"
                size="tiny"
                :show="statusPickerOpen"
                @update:value="changeStatus"
                @update:show="(v: boolean) => (statusPickerOpen = v)"
              />
            </div>
            <div class="prop-pill">
              <label class="prop-label">Priority</label>
              <NSelect
                ref="prioritySelectRef"
                :value="todo.priority"
                :options="priorityOptions"
                :render-label="renderPriorityLabel"
                size="tiny"
                :show="priorityPickerOpen"
                @update:value="changePriority"
                @update:show="(v: boolean) => (priorityPickerOpen = v)"
              />
            </div>
            <div class="prop-pill">
              <label class="prop-label">Difficulty</label>
              <NSelect
                ref="difficultySelectRef"
                :value="todo.difficulty"
                :options="difficultyOptions"
                :render-label="renderDifficultyLabel"
                size="tiny"
                :show="difficultyPickerOpen"
                @update:value="changeDifficulty"
                @update:show="(v: boolean) => (difficultyPickerOpen = v)"
              />
            </div>
          </div>
          <div class="prop-row">
            <div class="prop-pill prop-pill-wide">
              <label class="prop-label">Labels</label>
              <NSelect
                :value="todo.labels"
                :options="labelOptions"
                :render-label="renderLabelTag"
                size="tiny"
                multiple
                clearable
                placeholder="None"
                @update:value="changeLabels"
              />
            </div>
            <div class="prop-pill">
              <label class="prop-label">Assignee</label>
              <NSelect
                :value="todo.assignee"
                :options="assigneeOptions"
                :render-label="renderAssigneeLabel"
                size="tiny"
                clearable
                placeholder="None"
                @update:value="changeAssignee"
              />
            </div>
            <div class="prop-pill">
              <label class="prop-label">Sprint</label>
              <NSelect
                :value="todo.cycleId"
                :options="sprintOptions"
                :render-label="renderSprintLabel"
                size="tiny"
                clearable
                placeholder="None"
                @update:value="changeSprint"
              />
            </div>
          </div>
        </div>

        <!-- Tabs + Git sidebar -->
        <div class="tabs-and-git">
          <NTabs v-model:value="activeTab" type="line" size="small" class="detail-tabs">
            <NTabPane name="details" tab="Details">
              <!-- Description -->
              <div class="field-group">
                <NInput
                  v-model:value="description"
                  type="textarea"
                  :autosize="{ minRows: 4, maxRows: 16 }"
                  placeholder="Add a description..."
                  @blur="commitDescription"
                />
              </div>

              <!-- Timestamps footer -->
              <div class="timestamps">
                Created {{ formatDate(todo.createdAt) }} · Updated
                {{ formatDate(todo.updatedAt) }}
              </div>
            </NTabPane>

            <NTabPane name="plan" tab="Plan">
              <div class="plan-tab-content">
                <!-- Researching state -->
                <div v-if="planResearching" class="plan-researching">
                  <div class="plan-spinner" />
                  <span class="plan-progress-msg">{{ planProgressMsg }}</span>
                </div>

                <!-- Plan exists with content -->
                <div v-else-if="planExists && planContent" class="plan-content">
                  <div class="plan-markdown" v-html="renderedPlan" />
                  <div class="plan-answer-input">
                    <NInput
                      v-model:value="planAnswerText"
                      type="textarea"
                      :autosize="{ minRows: 2, maxRows: 4 }"
                      placeholder="Answer questions..."
                      @keydown="
                        (e: KeyboardEvent) => {
                          if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) submitPlanAnswer();
                        }
                      "
                    />
                    <NButton
                      size="small"
                      type="primary"
                      :loading="planAnswerLoading"
                      :disabled="!planAnswerText.trim()"
                      style="align-self: flex-end; margin-top: 8px"
                      @click="submitPlanAnswer"
                    >
                      Answer
                    </NButton>
                  </div>
                </div>

                <!-- No plan yet -->
                <div v-else class="plan-empty">
                  <NButton
                    size="small"
                    quaternary
                    :loading="planLoading"
                    @click="handleResearchPlan"
                  >
                    <template #icon>
                      <NIcon :size="14"><FileText /></NIcon>
                    </template>
                    Research Plan
                  </NButton>
                  <p class="plan-empty-hint">Generate a research plan for this todo using AI.</p>
                </div>
              </div>
            </NTabPane>

            <NTabPane name="comments" :tab="`Comments (${todo.comments?.length ?? 0})`">
              <div class="comments-tab-content">
                <div v-if="todo.comments?.length" class="comments-list">
                  <template v-for="c in threadedComments()" :key="c.id">
                    <div class="comment-item" :style="{ marginLeft: `${c.depth * 28}px` }">
                      <div :class="['comment-avatar', c.depth > 0 && 'comment-avatar-sm']">
                        {{ (c.authorName || "?")[0].toUpperCase() }}
                      </div>
                      <div class="comment-body">
                        <div class="comment-header">
                          <strong>{{ c.authorName }}</strong>
                          <span class="comment-date">{{ formatDate(c.createdAt) }}</span>
                          <NIcon class="comment-reply-btn" :size="18" @click="startReply(c.id)"
                            ><MessageCircle
                          /></NIcon>
                        </div>
                        <div class="comment-text">{{ c.text }}</div>
                      </div>
                    </div>
                    <!-- Reply input appears right after the comment being replied to -->
                    <div
                      v-if="replyingTo === c.id"
                      class="comment-reply-input"
                      :style="{ marginLeft: `${(c.depth + 1) * 28}px` }"
                    >
                      <NInput
                        ref="replyInputRef"
                        v-model:value="replyText"
                        type="textarea"
                        :autosize="{ minRows: 1, maxRows: 4 }"
                        placeholder="Write a reply..."
                        size="small"
                        @keydown="
                          (e: KeyboardEvent) => {
                            if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) submitReply();
                            if (e.key === 'Escape') {
                              replyingTo = null;
                              replyText = '';
                            }
                          }
                        "
                      />
                      <div class="comment-reply-actions">
                        <NButton
                          size="tiny"
                          quaternary
                          @click="
                            replyingTo = null;
                            replyText = '';
                          "
                          >Cancel</NButton
                        >
                        <NButton
                          size="tiny"
                          type="primary"
                          :disabled="!replyText.trim()"
                          @click="submitReply"
                          >Reply</NButton
                        >
                      </div>
                    </div>
                  </template>
                </div>
                <div class="comment-input">
                  <NInput
                    v-model:value="commentText"
                    type="textarea"
                    :autosize="{ minRows: 2, maxRows: 6 }"
                    placeholder="Leave a comment..."
                    @keydown="
                      (e: KeyboardEvent) => {
                        if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) submitComment();
                      }
                    "
                  />
                  <NButton
                    size="small"
                    type="primary"
                    :disabled="!commentText.trim()"
                    style="align-self: flex-end; margin-top: 8px"
                    @click="submitComment"
                  >
                    Comment
                  </NButton>
                </div>
              </div>
            </NTabPane>
          </NTabs>

          <!-- Git sidebar -->
          <div class="git-sidebar">
            <label class="section-label">
              <NIcon :size="12" style="vertical-align: -1px; margin-right: 4px"
                ><GitBranch
              /></NIcon>
              Git
            </label>

            <div class="git-row">
              <span class="git-row-label">Branch</span>
              <div v-if="todo.branch" class="git-row-value">
                <code class="git-ref">{{ todo.branch }}</code>
                <button
                  class="icon-btn icon-btn-danger"
                  title="Remove branch"
                  @click="handleRemoveBranch"
                >
                  <NIcon :size="12"><Trash /></NIcon>
                </button>
              </div>
              <div v-else class="git-row-actions">
                <NButton
                  size="tiny"
                  quaternary
                  :loading="branchLoading"
                  @click="handleCreateBranchOnly"
                  >+ branch</NButton
                >
                <NButton size="tiny" quaternary :loading="branchLoading" @click="handleCreateBranch"
                  >+ worktree</NButton
                >
              </div>
            </div>

            <div v-if="todo.worktrees?.length" class="git-row">
              <span class="git-row-label">Worktrees</span>
              <div class="git-row-list">
                <div v-for="wt in todo.worktrees" :key="wt" class="git-row-value">
                  <code class="git-ref">{{ wt }}</code>
                </div>
              </div>
            </div>

            <div class="git-row">
              <span class="git-row-label">Commits</span>
              <div v-if="todo.commits?.length || showCommitInput" class="git-row-list">
                <div v-for="sha in todo.commits ?? []" :key="sha" class="git-row-value">
                  <NIcon :size="12" style="opacity: 0.4; flex-shrink: 0"><GitCommit /></NIcon>
                  <a
                    v-if="store.remoteUrl"
                    :href="`${store.remoteUrl}/commit/${sha}`"
                    target="_blank"
                    class="git-ref git-link"
                    >{{ sha.slice(0, 8) }}</a
                  >
                  <code v-else class="git-ref">{{ sha.slice(0, 8) }}</code>
                </div>
                <div v-if="showCommitInput" class="git-commit-input">
                  <NInput
                    v-model:value="commitShaInput"
                    size="tiny"
                    placeholder="Paste SHA..."
                    @keyup.enter="submitCommitLink"
                    @keyup.escape="
                      showCommitInput = false;
                      commitShaInput = '';
                    "
                  />
                  <NButton
                    size="tiny"
                    type="primary"
                    :disabled="!commitShaInput.trim()"
                    @click="submitCommitLink"
                    >Link</NButton
                  >
                  <button
                    class="icon-btn"
                    title="Cancel"
                    @click="
                      showCommitInput = false;
                      commitShaInput = '';
                    "
                  >
                    <NIcon :size="12"><X /></NIcon>
                  </button>
                </div>
                <NButton v-else text size="tiny" class="git-add-btn" @click="showCommitInput = true"
                  >+ link commit</NButton
                >
              </div>
              <div v-else class="git-row-actions">
                <NButton size="tiny" quaternary @click="showCommitInput = true"
                  >+ link commit</NButton
                >
              </div>
            </div>
          </div>
        </div>
      </div>
    </NCard>
  </NModal>
</template>

<style scoped>
.not-found {
  text-align: center;
  padding: 40px;
  opacity: 0.5;
}

/* ── Header ── */
.detail-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.ref-tag {
  font-family: monospace;
  flex-shrink: 0;
  height: 34px;
  display: inline-flex;
  align-items: center;
}

.header-icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  opacity: 0.35;
  color: inherit;
  flex-shrink: 0;
  transition:
    opacity 0.15s,
    background 0.15s;
}

.header-icon-btn:hover {
  opacity: 0.8;
  background: rgba(128, 128, 128, 0.1);
}

.header-icon-btn-danger:hover {
  opacity: 0.9;
  color: #e06c75;
  background: rgba(224, 108, 117, 0.1);
}

.inline-title {
  flex: 1;
  min-width: 0;
  font-size: 17px;
  font-weight: 600;
  line-height: 1.4;
  height: 34px;
  border: none;
  outline: none;
  background: rgba(128, 128, 128, 0.06);
  padding: 4px 8px;
  color: inherit;
  font-family: inherit;
  border-radius: 0;
  transition: background 0.15s;
}

.inline-title:hover {
  background: rgba(128, 128, 128, 0.09);
}

.inline-title:focus {
  background: rgba(128, 128, 128, 0.11);
}

/* Shared icon button */
.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  opacity: 0.35;
  color: inherit;
  flex-shrink: 0;
  transition:
    opacity 0.15s,
    background 0.15s;
}

.icon-btn:hover {
  opacity: 0.8;
  background: rgba(128, 128, 128, 0.1);
}

.icon-btn-danger:hover {
  opacity: 0.9;
  color: #e06c75;
  background: rgba(224, 108, 117, 0.1);
}

/* ── Property pills bar ── */
.prop-bar {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
  padding-bottom: 14px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.08);
}

/* ── Tabs + Git sidebar ── */
.tabs-and-git {
  display: flex;
  flex: 1;
  min-height: 0;
}

.git-sidebar {
  width: 200px;
  flex-shrink: 0;
  border-left: 1px solid rgba(128, 128, 128, 0.08);
  padding-left: 20px;
  margin-left: 20px;
}

.prop-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.prop-pill {
  width: 150px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.prop-label {
  font-size: 10px;
  font-weight: 600;
  opacity: 0.4;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  padding-left: 2px;
}

.prop-pill-wide {
  width: 150px;
}

.prop-pill :deep(.n-base-selection) {
  --n-border: 1px solid rgba(128, 128, 128, 0.1) !important;
  --n-border-hover: 1px solid rgba(128, 128, 128, 0.22) !important;
  --n-border-active: 1px solid rgba(128, 128, 128, 0.22) !important;
  --n-border-focus: 1px solid rgba(128, 128, 128, 0.22) !important;
  --n-color: rgba(128, 128, 128, 0.04) !important;
  --n-color-active: rgba(128, 128, 128, 0.07) !important;
  border-radius: 6px !important;
}

/* ── Two-column layout ── */

/* ── Git sidebar ���─ */
.git-sidebar .section-label {
  margin-bottom: 12px;
}

/* Git rows */
.git-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 14px;
}

.git-row-label {
  font-size: 11px;
  font-weight: 500;
  opacity: 0.4;
}

.git-row-value {
  display: flex;
  align-items: center;
  gap: 6px;
}

.git-row-list {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.git-row-actions {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.git-ref {
  font-size: 11px;
  background: rgba(128, 128, 128, 0.08);
  padding: 2px 7px;
  border-radius: 4px;
  word-break: break-all;
  font-family: monospace;
  line-height: 1.5;
}

a.git-link {
  color: inherit;
  text-decoration: none;
  cursor: pointer;
}
a.git-link:hover {
  text-decoration: underline;
  color: #63e2b7;
}

.git-commit-input {
  display: flex;
  align-items: center;
  gap: 4px;
}

.git-commit-input .icon-btn {
  width: 22px;
  height: 22px;
  opacity: 0.4;
}

.git-add-btn {
  font-size: 11px;
  opacity: 0.35;
}
.git-add-btn:hover {
  opacity: 1;
}

/* ── Section labels (shared) ── */
.section-label {
  font-size: 11px;
  font-weight: 600;
  opacity: 0.4;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 8px;
  display: block;
}

.field-group {
  margin-bottom: 20px;
}

/* ── Plan ── */
.plan-content {
  border: 1px solid rgba(128, 128, 128, 0.1);
  border-radius: 8px;
  overflow: hidden;
  background: rgba(128, 128, 128, 0.02);
}

.plan-markdown {
  padding: 14px 16px;
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  word-wrap: break-word;
  font-family: inherit;
  max-height: 50vh;
  overflow-y: auto;
}

.plan-markdown :deep(h1) {
  font-size: 15px;
  font-weight: 700;
  margin: 0 0 8px;
}
.plan-markdown :deep(h2) {
  font-size: 14px;
  font-weight: 600;
  margin: 12px 0 6px;
}
.plan-markdown :deep(h3) {
  font-size: 13px;
  font-weight: 600;
  margin: 10px 0 4px;
}
.plan-markdown :deep(p) {
  margin: 0 0 8px;
}
.plan-markdown :deep(ul),
.plan-markdown :deep(ol) {
  margin: 0 0 8px;
  padding-left: 20px;
}
.plan-markdown :deep(li) {
  margin: 2px 0;
}
.plan-markdown :deep(code) {
  font-size: 12px;
  background: rgba(0, 0, 0, 0.05);
  padding: 1px 4px;
  border-radius: 3px;
}
.plan-markdown :deep(pre) {
  background: rgba(0, 0, 0, 0.04);
  padding: 8px 12px;
  border-radius: 4px;
  overflow-x: auto;
  margin: 0 0 8px;
}
.plan-markdown :deep(pre code) {
  background: none;
  padding: 0;
}
.plan-markdown :deep(strong) {
  font-weight: 600;
}
.plan-markdown :deep(blockquote) {
  border-left: 3px solid rgba(128, 128, 128, 0.2);
  padding-left: 12px;
  margin: 0 0 8px;
  opacity: 0.7;
}

.plan-answer-input {
  padding: 12px 16px;
  border-top: 1px solid rgba(128, 128, 128, 0.08);
  display: flex;
  flex-direction: column;
}

.plan-researching {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  border: 1px solid rgba(128, 128, 128, 0.1);
  border-radius: 8px;
  background: rgba(128, 128, 128, 0.02);
}

.plan-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(128, 128, 128, 0.15);
  border-top-color: var(--n-primary-color, #18a058);
  border-radius: 50%;
  animation: plan-spin 0.8s linear infinite;
  flex-shrink: 0;
}

@keyframes plan-spin {
  to {
    transform: rotate(360deg);
  }
}

.plan-progress-msg {
  font-size: 13px;
  opacity: 0.6;
}

.plan-empty {
  padding: 4px 0;
}

/* ── Activity / Comments ── */
.activity-section {
  border-top: 1px solid rgba(128, 128, 128, 0.08);
  padding-top: 16px;
  margin-top: 4px;
}

.comments-list {
  margin-bottom: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.comment-item {
  display: flex;
  gap: 10px;
  align-items: flex-start;
}

.comment-avatar {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: rgba(128, 128, 128, 0.1);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  opacity: 0.6;
  flex-shrink: 0;
  margin-top: 1px;
}

.comment-body {
  flex: 1;
  min-width: 0;
}

.comment-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 2px;
  font-size: 13px;
}

.comment-date {
  font-size: 11px;
  opacity: 0.35;
}

.comment-text {
  font-size: 13px;
  white-space: pre-wrap;
  line-height: 1.5;
}

.comment-reply-btn {
  cursor: pointer;
  opacity: 0.3;
  transition: opacity 0.15s;
  display: flex;
  align-items: center;
}

.comment-reply-btn:hover {
  opacity: 0.7;
}

.comment-avatar-sm {
  width: 22px;
  height: 22px;
  font-size: 10px;
}

.comment-reply-input {
  margin-top: 6px;
}

.comment-reply-actions {
  display: flex;
  justify-content: flex-end;
  gap: 4px;
  margin-top: 4px;
}

.comment-input {
  display: flex;
  flex-direction: column;
}

/* ── Layout: make card content flex so tabs fill remaining space ── */
.detail-card :deep(.n-card__content) {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.detail-wrapper {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

/* ── Tabs ── */
.detail-tabs {
  margin-top: 0;
  flex: 1;
  min-width: 0;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.detail-tabs :deep(.n-tabs-nav) {
  flex-shrink: 0;
}

.detail-tabs :deep(.n-tabs-pane-wrapper) {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.detail-tabs :deep(.n-tab-pane) {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-top: 8px;
}

/* Remove gap between tab buttons */
.detail-tabs :deep(.n-tabs-tab) {
  margin-right: 0 !important;
  margin-left: 0 !important;
}

.detail-tabs :deep(.n-tabs-tab-wrapper) {
  gap: 0 !important;
}

/* ── Plan tab ── */
.plan-tab-content {
  padding-top: 4px;
}

/* ── Comments tab ── */
.comments-tab-content {
  padding-top: 4px;
}

.plan-empty-hint {
  margin-top: 8px;
  font-size: 12px;
  opacity: 0.35;
}

/* ── Timestamps footer ── */
.timestamps {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid rgba(128, 128, 128, 0.06);
  font-size: 11px;
  opacity: 0.3;
}
</style>
