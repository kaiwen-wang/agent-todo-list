<script setup lang="ts">
import { ref, watch, nextTick } from "vue";
import { useMessage } from "naive-ui";
import { useProjectStore } from "@/stores/project";

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
}>();

const store = useProjectStore();
const message = useMessage();

const name = ref("");
const prefix = ref("");
const description = ref("");
const submitting = ref(false);
const nameInputRef = ref<HTMLInputElement | null>(null);

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen && store.project) {
      name.value = store.project.name;
      prefix.value = store.project.prefix;
      description.value = store.project.description ?? "";
      nextTick(() => nameInputRef.value?.focus());
    }
  },
);

async function submit() {
  if (!name.value.trim() || !prefix.value.trim()) return;
  submitting.value = true;
  try {
    const updates: Record<string, string> = {};
    if (name.value.trim() !== store.project?.name) {
      updates.name = name.value.trim();
    }
    if (prefix.value.trim().toUpperCase() !== store.project?.prefix) {
      updates.prefix = prefix.value.trim();
    }
    if (description.value.trim() !== (store.project?.description ?? "")) {
      updates.description = description.value.trim();
    }
    if (Object.keys(updates).length > 0) {
      await store.updateProjectSettings(updates);
      message.success("Settings saved");
    }
    emit("close");
  } catch {
    message.error("Failed to save settings");
  } finally {
    submitting.value = false;
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.preventDefault();
    e.stopPropagation();
    emit("close");
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="settings-overlay" @mousedown.self="emit('close')">
      <div class="settings-container" @keydown="handleKeydown">
        <div class="settings-header">
          <span class="settings-title">Project Settings</span>
          <kbd class="settings-esc" @click="emit('close')">ESC</kbd>
        </div>

        <form class="settings-body" @submit.prevent="submit">
          <div class="settings-field">
            <label class="settings-label">Project Name</label>
            <input
              ref="nameInputRef"
              v-model="name"
              class="settings-input"
              placeholder="My Project"
              spellcheck="false"
              @keydown.enter.prevent="submit"
            />
          </div>

          <div class="settings-field">
            <label class="settings-label">Prefix</label>
            <input
              v-model="prefix"
              class="settings-input settings-input-prefix"
              placeholder="TODO"
              spellcheck="false"
              @keydown.enter.prevent="submit"
            />
            <span class="settings-hint">
              Used for todo references like {{ (prefix || "TODO").toUpperCase() }}-1,
              {{ (prefix || "TODO").toUpperCase() }}-2, etc.
            </span>
          </div>

          <div class="settings-field">
            <label class="settings-label">Description</label>
            <textarea
              v-model="description"
              class="settings-textarea"
              placeholder="Brief project description"
              rows="3"
              spellcheck="false"
            />
          </div>

          <div class="settings-actions">
            <button type="button" class="settings-btn settings-btn-cancel" @click="emit('close')">
              Cancel
            </button>
            <button
              type="submit"
              class="settings-btn settings-btn-save"
              :disabled="!name.trim() || !prefix.trim() || submitting"
            >
              {{ submitting ? "Saving..." : "Save" }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 9999;
  display: flex;
  justify-content: center;
  padding-top: 18vh;
}

.settings-container {
  width: 480px;
  max-width: 95vw;
  background: #fff;
  border-radius: 10px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  height: fit-content;
}

.settings-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 16px;
  border-bottom: 1px solid #e5e5e5;
}

.settings-title {
  font-size: 15px;
  font-weight: 500;
  color: inherit;
}

.settings-esc {
  font-size: 10px;
  font-family: inherit;
  padding: 2px 6px;
  border-radius: 4px;
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #e5e7eb;
  flex-shrink: 0;
  cursor: pointer;
}

.settings-esc:hover {
  background: #e5e7eb;
}

.settings-body {
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.settings-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.settings-label {
  font-size: 12px;
  font-weight: 600;
  color: #6b7280;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}

.settings-input,
.settings-textarea {
  font-family: inherit;
  font-size: 14px;
  padding: 9px 12px;
  border: 1px solid #e5e7eb;
  border-radius: 6px;
  outline: none;
  background: #fafafa;
  color: inherit;
  transition:
    border-color 0.15s,
    background 0.15s;
}

.settings-input:focus,
.settings-textarea:focus {
  border-color: #b0b0b0;
  background: #fff;
}

.settings-input::placeholder,
.settings-textarea::placeholder {
  color: #9ca3af;
}

.settings-input-prefix {
  text-transform: uppercase;
}

.settings-textarea {
  resize: vertical;
  min-height: 60px;
}

.settings-hint {
  font-size: 12px;
  color: #9ca3af;
  line-height: 1.4;
}

.settings-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding-top: 4px;
}

.settings-btn {
  font-family: inherit;
  font-size: 13px;
  font-weight: 500;
  padding: 7px 16px;
  border-radius: 6px;
  border: 1px solid transparent;
  cursor: pointer;
  transition:
    background 0.15s,
    border-color 0.15s;
}

.settings-btn-cancel {
  background: #f3f4f6;
  color: #374151;
  border-color: #e5e7eb;
}

.settings-btn-cancel:hover {
  background: #e5e7eb;
}

.settings-btn-save {
  background: #111;
  color: #fff;
  border-color: #111;
}

.settings-btn-save:hover:not(:disabled) {
  background: #333;
}

.settings-btn-save:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
</style>
