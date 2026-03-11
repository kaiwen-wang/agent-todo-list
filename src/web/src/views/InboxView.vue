<script setup lang="ts">
import { ref, watch, nextTick, computed } from "vue";
import { NButton, NSpin } from "naive-ui";
import { useProjectStore } from "@/stores/project";
import type { BrainEvent } from "@/types";

const store = useProjectStore();

// Track focus state — MUST be defined BEFORE the watcher below
const isFocused = ref(false);

// Local draft text — synced from store on load, debounced save on edits
const draft = ref("");
const showProcessed = ref(false);
const logContainer = ref<HTMLElement | null>(null);

// Sync draft from store when project loads or inbox changes externally
watch(
  () => store.inboxText,
  (newText) => {
    // Only update if not currently focused (avoids overwriting while typing)
    if (!isFocused.value) {
      draft.value = newText;
    }
  },
  { immediate: true },
);

// Debounced save
let saveTimer: ReturnType<typeof setTimeout> | null = null;

function onInput() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    store.updateInbox(draft.value);
  }, 1000);
}

function onBlur() {
  isFocused.value = false;
  // Flush any pending save immediately
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  store.updateInbox(draft.value);
}

// Brain processing
async function runBrain() {
  try {
    await store.startBrainProcess();
  } catch {
    // Error already captured in store
  }
}

// Auto-scroll brain log to bottom
watch(
  () => store.brainLogs.length,
  async () => {
    await nextTick();
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  },
);

// Format brain log line for display
function formatLog(event: BrainEvent): string {
  switch (event.type) {
    case "brain:log":
      return event.message;
    case "brain:task":
      return `+ Created ${event.ref}: ${event.title}`;
    case "brain:error":
      return `! ${event.message}`;
    case "brain:done":
      return event.processed > 0
        ? `Done! Created ${event.processed} task${event.processed === 1 ? "" : "s"}.`
        : "No tasks were created.";
    default:
      return "";
  }
}

function logClass(event: BrainEvent): string {
  switch (event.type) {
    case "brain:task":
      return "log-task";
    case "brain:error":
      return "log-error";
    case "brain:done":
      return "log-done";
    default:
      return "log-info";
  }
}

const hasBrainLogs = computed(() => store.brainLogs.length > 0);
const processedText = computed(() => store.inboxProcessed);
const hasProcessed = computed(() => processedText.value.trim().length > 0);
const inboxEmpty = computed(() => !draft.value.trim());
</script>

<template>
  <div class="inbox-view">
    <div class="inbox-toolbar">
      <h2>Inbox</h2>
      <div class="toolbar-actions">
        <NButton
          v-if="hasBrainLogs && !store.brainProcessing"
          size="small"
          quaternary
          @click="store.clearBrainLogs()"
        >
          Clear Log
        </NButton>
        <NButton
          type="primary"
          size="small"
          :disabled="store.brainProcessing || inboxEmpty"
          @click="runBrain"
        >
          <template v-if="store.brainProcessing">
            <NSpin :size="14" style="margin-right: 6px" />
            Processing...
          </template>
          <template v-else> Process with Brain </template>
        </NButton>
      </div>
    </div>

    <div class="inbox-content">
      <!-- Textarea -->
      <div class="inbox-editor">
        <textarea
          v-model="draft"
          class="inbox-textarea"
          placeholder="Jot down quick thoughts, ideas, bugs, features...&#10;&#10;Each item will be expanded into a structured task by the Brain."
          :disabled="store.brainProcessing"
          @input="onInput"
          @focus="isFocused = true"
          @blur="onBlur"
        />
      </div>

      <!-- Brain Log Panel -->
      <div v-if="hasBrainLogs" class="brain-log-panel">
        <div class="panel-header">Brain Log</div>
        <div ref="logContainer" class="log-content">
          <div
            v-for="(event, i) in store.brainLogs"
            :key="i"
            class="log-line"
            :class="logClass(event)"
          >
            {{ formatLog(event) }}
          </div>
          <div v-if="store.brainProcessing" class="log-line log-info log-pending">
            <NSpin :size="12" style="margin-right: 6px" />
            Waiting for Brain...
          </div>
        </div>
      </div>

      <!-- Processed Archive -->
      <div v-if="hasProcessed" class="processed-section">
        <button class="processed-toggle" @click="showProcessed = !showProcessed">
          {{ showProcessed ? "Hide" : "Show" }} Previously Processed
        </button>
        <pre v-if="showProcessed" class="processed-content">{{ processedText }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
.inbox-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.inbox-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  flex-shrink: 0;
}

.inbox-toolbar h2 {
  font-size: 16px;
  font-weight: 600;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.inbox-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 0 24px 24px;
  overflow-y: auto;
  min-height: 0;
}

/* ── Textarea ── */

.inbox-editor {
  flex: 1;
  min-height: 200px;
  display: flex;
}

.inbox-textarea {
  width: 100%;
  height: 100%;
  min-height: 200px;
  padding: 14px 16px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  font-family: "SF Mono", "Fira Code", "Consolas", monospace;
  font-size: 13px;
  line-height: 1.6;
  resize: none;
  outline: none;
  background: #fafafa;
  color: #333;
}

.inbox-textarea:focus {
  border-color: #999;
  background: #fff;
}

.inbox-textarea:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.inbox-textarea::placeholder {
  color: #aaa;
}

/* ── Brain Log Panel ── */

.brain-log-panel {
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  overflow: hidden;
  flex-shrink: 0;
}

.panel-header {
  padding: 8px 14px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #666;
  background: #f5f5f5;
  border-bottom: 1px solid #e5e5e5;
}

.log-content {
  max-height: 300px;
  overflow-y: auto;
  padding: 10px 14px;
  background: #1a1a2e;
  font-family: "SF Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  line-height: 1.7;
}

.log-line {
  white-space: pre-wrap;
  word-break: break-all;
}

.log-info {
  color: #8892b0;
}

.log-task {
  color: #64ffda;
}

.log-error {
  color: #ff6b6b;
}

.log-done {
  color: #c3e88d;
  font-weight: 600;
}

.log-pending {
  display: flex;
  align-items: center;
  opacity: 0.7;
}

/* ── Processed Archive ── */

.processed-section {
  flex-shrink: 0;
}

.processed-toggle {
  background: none;
  border: none;
  font-size: 12px;
  color: #888;
  cursor: pointer;
  padding: 4px 0;
  font-family: inherit;
}

.processed-toggle:hover {
  color: #555;
}

.processed-content {
  margin-top: 8px;
  padding: 12px 14px;
  background: #f9f9f9;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  font-family: "SF Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  line-height: 1.6;
  color: #666;
  white-space: pre-wrap;
  max-height: 200px;
  overflow-y: auto;
}
</style>
