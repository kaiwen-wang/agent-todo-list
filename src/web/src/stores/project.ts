/** Pinia store — single source of truth for the project state in the frontend */

import { ref, computed } from "vue";
import { defineStore } from "pinia";
import type { Project, Todo, Status, BrainEvent } from "@/types";
import * as api from "@/api";

export const useProjectStore = defineStore("project", () => {
  const project = ref<Project | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // Getters
  const todos = computed(() => project.value?.todos ?? []);
  const members = computed(() => project.value?.members ?? []);
  const projectName = computed(() => project.value?.name ?? "");
  const prefix = computed(() => project.value?.prefix ?? "");
  const auditLog = computed(() => project.value?.auditLog ?? []);
  const inboxText = computed(() => project.value?.inboxText ?? "");
  const inboxProcessed = computed(() => project.value?.inboxProcessed ?? "");

  // ── Brain state ──
  const brainProcessing = ref(false);
  const brainLogs = ref<BrainEvent[]>([]);

  /** Todos grouped by status for the board view */
  const todosByStatus = computed(() => {
    const grouped: Record<Status, Todo[]> = {
      none: [],
      todo: [],
      in_progress: [],
      completed: [],
      archived: [],
      wont_do: [],
      needs_elaboration: [],
    };
    for (const todo of todos.value) {
      const key = todo.status;
      if (key && grouped[key]) {
        grouped[key].push(todo);
      }
    }
    // Sort each column newest-first by createdAt
    for (const key of Object.keys(grouped) as Status[]) {
      grouped[key].sort((a, b) => {
        const ta = a.createdAt ? new Date(a.createdAt).getTime() : 0;
        const tb = b.createdAt ? new Date(b.createdAt).getTime() : 0;
        return tb - ta;
      });
    }
    return grouped;
  });

  /** Active todos (not archived or won't do) */
  const activeTodos = computed(() =>
    todos.value.filter((t) => t.status !== "archived" && t.status !== "wont_do"),
  );

  /** Counts per status */
  const statusCounts = computed(() => {
    const counts: Record<Status, number> = {
      none: 0,
      todo: 0,
      in_progress: 0,
      completed: 0,
      archived: 0,
      wont_do: 0,
      needs_elaboration: 0,
    };
    for (const todo of todos.value) {
      const key = todo.status;
      if (key && counts[key] !== undefined) {
        counts[key]++;
      }
    }
    return counts;
  });

  // Actions

  /** Deduplicated load — if a fetch is already in-flight, piggyback on it */
  let loadPromise: Promise<void> | null = null;

  async function load() {
    if (loadPromise) return loadPromise;
    loadPromise = doLoad();
    try {
      await loadPromise;
    } finally {
      loadPromise = null;
    }
  }

  async function doLoad() {
    loading.value = true;
    error.value = null;
    try {
      project.value = await api.fetchProject();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  async function addTodo(params: api.AddTodoParams) {
    error.value = null;
    try {
      const result = await api.addTodo(params);
      await load(); // Reload to get the full updated state
      return result;
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function updateTodo(number: number, updates: api.UpdateTodoParams) {
    error.value = null;
    try {
      await api.updateTodo(number, updates);
      await load();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function deleteTodo(number: number) {
    error.value = null;
    try {
      await api.deleteTodo(number);
      await load();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function moveTodo(number: number, status: Status) {
    // Optimistic: update local state immediately
    if (project.value) {
      project.value = {
        ...project.value,
        todos: project.value.todos.map((t) =>
          t.number === number ? { ...t, status, updatedAt: Date.now() } : t,
        ),
      };
    }
    skipNextRefresh = true;
    try {
      await api.updateTodo(number, { status });
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      skipNextRefresh = false;
      await load();
    }
  }

  async function addComment(number: number, text: string) {
    error.value = null;
    try {
      await api.addCommentApi(number, text);
      await load();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function createBranch(number: number) {
    error.value = null;
    try {
      const result = await api.createBranchApi(number);
      await load();
      return result;
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function removeBranch(number: number) {
    error.value = null;
    try {
      const result = await api.removeBranchApi(number);
      await load();
      return result;
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function updateProjectSettings(updates: api.UpdateProjectParams) {
    error.value = null;
    try {
      await api.updateProjectSettings(updates);
      await load();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  // ── Inbox actions ──

  async function updateInbox(text: string) {
    error.value = null;
    try {
      await api.updateInbox(text);
      // Update local state immediately (no full reload needed)
      if (project.value) {
        project.value.inboxText = text;
      }
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function startBrainProcess() {
    error.value = null;
    brainProcessing.value = true;
    brainLogs.value = [];
    try {
      await api.triggerBrainProcess();
      // Brain events will arrive via WebSocket — the API just kicks it off
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      brainProcessing.value = false;
      throw e;
    }
  }

  function clearBrainLogs() {
    brainLogs.value = [];
  }

  // ── Member actions ──

  async function addMember(params: api.AddMemberParams) {
    error.value = null;
    try {
      await api.addMember(params);
      await load();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function removeMember(memberId: string) {
    error.value = null;
    try {
      await api.removeMember(memberId);
      await load();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  async function updateMember(memberId: string, updates: api.UpdateMemberParams) {
    error.value = null;
    try {
      await api.updateMemberApi(memberId, updates);
      await load();
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  // ── WebSocket for live updates ──

  let ws: WebSocket | null = null;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  const WS_RECONNECT_MS = 2000;

  /**
   * Skip the next WebSocket-triggered refresh. Used after optimistic updates
   * to prevent a redundant re-render from our own server write.
   */
  let skipNextRefresh = false;

  function connectWebSocket() {
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) return;

    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    ws = new WebSocket(`${proto}//${location.host}/ws`);

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        switch (msg.type) {
          case "refresh":
            if (skipNextRefresh) {
              skipNextRefresh = false;
            } else {
              load();
            }
            break;
          case "brain:log":
          case "brain:task":
            brainLogs.value.push(msg);
            break;
          case "brain:error":
            brainLogs.value.push(msg);
            brainProcessing.value = false;
            break;
          case "brain:done":
            brainLogs.value.push(msg);
            brainProcessing.value = false;
            load(); // Refresh to see newly created tasks + cleared inbox
            break;
        }
      } catch {
        // Legacy plain-text messages (backward compat)
        if (event.data === "refresh") {
          if (skipNextRefresh) {
            skipNextRefresh = false;
          } else {
            load();
          }
        }
      }
    };

    ws.onclose = () => {
      ws = null;
      // Auto-reconnect after delay
      if (!reconnectTimer) {
        reconnectTimer = setTimeout(() => {
          reconnectTimer = null;
          connectWebSocket();
        }, WS_RECONNECT_MS);
      }
    };

    ws.onerror = () => {
      // onclose will fire after this, triggering reconnect
    };
  }

  /** Currently selected todo number for the detail modal (null = closed) */
  const selectedTodoNumber = ref<number | null>(null);

  function openTodo(number: number) {
    selectedTodoNumber.value = number;
  }

  function closeTodo() {
    selectedTodoNumber.value = null;
  }

  // ── Multi-select state for kanban ──

  /** Set of selected todo IDs (for CMD+click / Shift+click multi-select) */
  const selectedTodoIds = ref<Set<string>>(new Set());
  /** Last individually toggled todo ID — anchor for Shift+click range select */
  const lastSelectedId = ref<string | null>(null);
  /** Whether any selection is active */
  const hasSelection = computed(() => selectedTodoIds.value.size > 0);
  /** Number of selected items */
  const selectionCount = computed(() => selectedTodoIds.value.size);

  function toggleSelect(todoId: string) {
    const next = new Set(selectedTodoIds.value);
    if (next.has(todoId)) {
      next.delete(todoId);
    } else {
      next.add(todoId);
    }
    selectedTodoIds.value = next;
    lastSelectedId.value = todoId;
  }

  /**
   * Shift+click range select: selects all items between the last-selected item
   * and the clicked item within the provided column todo list.
   */
  function rangeSelect(todoId: string, columnTodos: Todo[]) {
    if (!lastSelectedId.value) {
      // No anchor — just toggle this one
      toggleSelect(todoId);
      return;
    }
    const anchorIdx = columnTodos.findIndex((t) => t.id === lastSelectedId.value);
    const targetIdx = columnTodos.findIndex((t) => t.id === todoId);
    if (anchorIdx === -1 || targetIdx === -1) {
      toggleSelect(todoId);
      return;
    }
    const start = Math.min(anchorIdx, targetIdx);
    const end = Math.max(anchorIdx, targetIdx);
    const next = new Set(selectedTodoIds.value);
    for (let i = start; i <= end; i++) {
      next.add(columnTodos[i]!.id);
    }
    selectedTodoIds.value = next;
    // Don't update lastSelectedId so further Shift+clicks extend from original anchor
  }

  function clearSelection() {
    selectedTodoIds.value = new Set();
    lastSelectedId.value = null;
  }

  function selectAll() {
    const next = new Set<string>();
    for (const todo of activeTodos.value) {
      next.add(todo.id);
    }
    selectedTodoIds.value = next;
  }

  // ── Bulk actions (optimistic) ──

  async function bulkUpdateTodos(updates: api.UpdateTodoParams) {
    error.value = null;
    const selected = todos.value.filter((t) => selectedTodoIds.value.has(t.id));
    const ops: api.BulkOperation[] = selected.map((t) => ({
      action: "update" as const,
      number: t.number,
      updates,
    }));

    // Optimistic: apply changes to local state immediately (no await = one Vue render)
    if (project.value) {
      const ids = selectedTodoIds.value;
      project.value = {
        ...project.value,
        todos: project.value.todos.map((t) =>
          ids.has(t.id) ? { ...t, ...updates, updatedAt: Date.now() } : t,
        ),
      };
    }

    // Fire API call — skip the WebSocket refresh it triggers (we already updated locally)
    skipNextRefresh = true;
    try {
      await api.bulkChange(ops);
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      skipNextRefresh = false;
      await load(); // Rollback on error
    }
  }

  async function bulkDeleteTodos() {
    error.value = null;
    const selected = todos.value.filter((t) => selectedTodoIds.value.has(t.id));
    const ops: api.BulkOperation[] = selected.map((t) => ({
      action: "delete" as const,
      number: t.number,
    }));

    // Optimistic: remove items from local state immediately
    if (project.value) {
      const ids = selectedTodoIds.value;
      project.value = {
        ...project.value,
        todos: project.value.todos.filter((t) => !ids.has(t.id)),
      };
    }
    clearSelection(); // Clear for delete since items no longer exist

    // Fire API call — skip the WebSocket refresh it triggers (we already updated locally)
    skipNextRefresh = true;
    try {
      await api.bulkChange(ops);
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : String(e);
      skipNextRefresh = false;
      await load(); // Rollback on error
    }
  }

  return {
    project,
    loading,
    error,
    todos,
    members,
    projectName,
    prefix,
    auditLog,
    todosByStatus,
    activeTodos,
    statusCounts,
    inboxText,
    inboxProcessed,
    brainProcessing,
    brainLogs,
    selectedTodoNumber,
    load,
    addTodo,
    updateTodo,
    deleteTodo,
    moveTodo,
    addComment,
    createBranch,
    removeBranch,
    updateProjectSettings,
    updateInbox,
    startBrainProcess,
    clearBrainLogs,
    addMember,
    removeMember,
    updateMember,
    openTodo,
    closeTodo,
    connectWebSocket,
    selectedTodoIds,
    lastSelectedId,
    hasSelection,
    selectionCount,
    toggleSelect,
    rangeSelect,
    clearSelection,
    selectAll,
    bulkUpdateTodos,
    bulkDeleteTodos,
  };
});
