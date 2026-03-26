<script setup lang="ts">
import { ref, watch, computed } from "vue";
import { NButton, NSpin } from "naive-ui";
import { useProjectStore } from "@/stores/project";

const store = useProjectStore();

// Track focus state
const isFocused = ref(false);

// Local draft text — synced from store on load, debounced save on edits
const draft = ref("");
const showProcessed = ref(false);

// Sync draft from store when project loads or inbox changes externally
watch(
  () => store.inboxText,
  (newText) => {
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
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  store.updateInbox(draft.value);
}

async function processInbox() {
  // Flush any pending save first
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
    await store.updateInbox(draft.value);
  }
  try {
    await store.processInboxItems();
    draft.value = "";
  } catch {
    // Error already captured in store
  }
}

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
          v-if="store.inboxResult && !store.inboxProcessing"
          size="small"
          quaternary
          @click="store.clearInboxResult()"
        >
          Dismiss
        </NButton>
        <NButton
          type="primary"
          size="small"
          :disabled="store.inboxProcessing || inboxEmpty"
          @click="processInbox"
        >
          <template v-if="store.inboxProcessing">
            <NSpin :size="14" style="margin-right: 6px" />
            Processing...
          </template>
          <template v-else> Process Inbox </template>
        </NButton>
      </div>
    </div>

    <div class="inbox-content">
      <!-- Textarea -->
      <div class="inbox-editor">
        <textarea
          v-model="draft"
          class="inbox-textarea"
          placeholder="Jot down quick thoughts, ideas, bugs, features...&#10;&#10;Separate items with blank lines. Each becomes a task."
          :disabled="store.inboxProcessing"
          @input="onInput"
          @focus="isFocused = true"
          @blur="onBlur"
        />
      </div>

      <!-- Processing result -->
      <div v-if="store.inboxResult" class="result-panel">
        <div v-if="store.inboxResult.processed === 0" class="result-empty">No tasks created.</div>
        <div v-else class="result-success">
          <div class="result-header">
            Created {{ store.inboxResult.processed }} task{{
              store.inboxResult.processed === 1 ? "" : "s"
            }}
          </div>
          <div v-for="task in store.inboxResult.tasks" :key="task.ref" class="result-task">
            <span class="task-ref">{{ task.ref }}</span>
            {{ task.title }}
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

/* -- Textarea -- */

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

/* -- Result Panel -- */

.result-panel {
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  padding: 12px 16px;
  flex-shrink: 0;
}

.result-empty {
  color: #888;
  font-size: 13px;
}

.result-header {
  font-weight: 600;
  font-size: 13px;
  color: #333;
  margin-bottom: 6px;
}

.result-task {
  font-family: "SF Mono", "Fira Code", "Consolas", monospace;
  font-size: 12px;
  line-height: 1.7;
  color: #555;
}

.task-ref {
  color: #18a058;
  font-weight: 600;
  margin-right: 6px;
}

/* -- Processed Archive -- */

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
