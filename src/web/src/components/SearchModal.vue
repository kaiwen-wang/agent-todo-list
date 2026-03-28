<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { useProjectStore } from "@/stores/project";
import type { Todo, Status } from "@/types";
import { STATUS_DISPLAY, STATUS_COLORS, PRIORITY_COLORS } from "@/types";

const props = defineProps<{ open: boolean }>();
const emit = defineEmits<{ close: [] }>();

const store = useProjectStore();
const query = ref("");
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);
const resultsRef = ref<HTMLDivElement | null>(null);

interface ScoredTodo extends Todo {
  _score: number;
}

function scoreTodos(todos: Todo[], q: string, prefix: string): ScoredTodo[] {
  const lower = q.toLowerCase().trim();
  if (!lower) return [];

  const scored: ScoredTodo[] = [];
  for (const todo of todos) {
    let score = 0;

    // Ref match: "75" or "AGT-75"
    const todoRef = `${prefix}-${todo.number}`.toLowerCase();
    if (todoRef === lower || String(todo.number) === lower) {
      score += 200;
    } else if (todoRef.includes(lower) || lower.includes(String(todo.number))) {
      score += 100;
    }

    // Title match
    const titleLower = todo.title.toLowerCase();
    if (titleLower.startsWith(lower)) {
      score += 100;
    } else if (titleLower.includes(lower)) {
      score += 80;
    }

    // Status display name match
    const statusDisplay = STATUS_DISPLAY[todo.status ?? "none"]?.toLowerCase() ?? "";
    if (statusDisplay.includes(lower)) {
      score += 40;
    }

    // Assignee match
    if (todo.assigneeName?.toLowerCase().includes(lower)) {
      score += 40;
    }

    // Description match
    if (todo.description.toLowerCase().includes(lower)) {
      score += 20;
    }

    // Comment text match
    if (todo.comments?.some((c) => c.text.toLowerCase().includes(lower))) {
      score += 10;
    }

    if (score > 0) {
      scored.push({ ...todo, _score: score });
    }
  }

  return scored.sort((a, b) => b._score - a._score).slice(0, 50);
}

const results = computed(() => scoreTodos(store.todos, query.value, store.prefix));

watch(query, () => {
  selectedIndex.value = 0;
});

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      query.value = "";
      selectedIndex.value = 0;
      nextTick(() => inputRef.value?.focus());
    }
  },
);

function scrollActiveIntoView() {
  nextTick(() => {
    const el = resultsRef.value?.querySelector(".search-result-item.active");
    el?.scrollIntoView({ block: "nearest" });
  });
}

function selectResult(todo: ScoredTodo) {
  store.openTodo(todo.number);
  emit("close");
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    if (selectedIndex.value < results.value.length - 1) {
      selectedIndex.value++;
      scrollActiveIntoView();
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (selectedIndex.value > 0) {
      selectedIndex.value--;
      scrollActiveIntoView();
    }
  } else if (e.key === "Enter") {
    const item = results.value[selectedIndex.value];
    if (!item) return;
    e.preventDefault();
    selectResult(item);
  } else if (e.key === "Escape") {
    e.preventDefault();
    e.stopPropagation();
    emit("close");
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="search-overlay" @mousedown.self="emit('close')">
      <div class="search-container" @keydown="handleKeydown">
        <div class="search-input-wrapper">
          <svg
            class="search-icon"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.3-4.3" />
          </svg>
          <input
            ref="inputRef"
            v-model="query"
            class="search-input"
            placeholder="Search todos..."
            spellcheck="false"
          />
          <kbd class="search-esc">ESC</kbd>
        </div>
        <div ref="resultsRef" class="search-results">
          <button
            v-for="(result, i) in results"
            :key="result.id"
            class="search-result-item"
            :class="{ active: i === selectedIndex }"
            @click="selectResult(result)"
            @mouseenter="selectedIndex = i"
          >
            <span class="result-ref">{{ store.prefix }}-{{ result.number }}</span>
            <span class="result-title">{{ result.title }}</span>
            <span class="result-meta">
              <span
                class="result-status-dot"
                :style="{ background: STATUS_COLORS[result.status ?? 'none'] }"
              />
              <span class="result-status">{{ STATUS_DISPLAY[result.status ?? "none"] }}</span>
              <span
                v-if="result.priority !== 'none'"
                class="result-priority"
                :style="{ color: PRIORITY_COLORS[result.priority] }"
              >
                {{ result.priority }}
              </span>
            </span>
          </button>
          <div v-if="query && results.length === 0" class="search-empty">No results found</div>
          <div v-if="!query" class="search-hint">Search by title, number, status, assignee...</div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.search-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 9999;
  display: flex;
  justify-content: center;
  padding-top: 18vh;
}

.search-container {
  width: 560px;
  max-height: 60vh;
  background: #fff;
  border-radius: 10px;
  box-shadow: 0 16px 48px rgba(0, 0, 0, 0.2);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  height: fit-content;
}

.search-input-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 14px 16px;
  border-bottom: 1px solid #e5e5e5;
}

.search-icon {
  flex-shrink: 0;
  opacity: 0.35;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  font-size: 15px;
  font-family: inherit;
  background: transparent;
  color: inherit;
}

.search-input::placeholder {
  color: #9ca3af;
}

.search-esc {
  font-size: 10px;
  font-family: inherit;
  padding: 2px 6px;
  border-radius: 4px;
  background: #f3f4f6;
  color: #6b7280;
  border: 1px solid #e5e7eb;
  flex-shrink: 0;
}

.search-results {
  overflow-y: auto;
  max-height: calc(60vh - 52px);
  padding: 4px;
}

.search-result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  font-size: 13px;
  color: inherit;
}

.search-result-item.active {
  background: #f3f4f6;
}

.result-ref {
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, monospace;
  font-size: 12px;
  color: #9ca3af;
  flex-shrink: 0;
  min-width: 56px;
}

.result-title {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.result-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.result-status-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}

.result-status {
  font-size: 11px;
  color: #6b7280;
}

.result-priority {
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
}

.search-empty,
.search-hint {
  padding: 24px 16px;
  text-align: center;
  font-size: 13px;
  color: #9ca3af;
}
</style>
