<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import {
  NConfigProvider,
  NMessageProvider,
  NLayout,
  NLayoutSider,
  NLayoutContent,
  NTag,
  NIcon,
} from "naive-ui";
import {
  Inbox,
  LayoutKanban,
  LayoutRows,
  Table,
  Users,
  Settings,
  Search,
  ChartDots,
} from "@vicons/tabler";
import { useProjectStore } from "@/stores/project";
import { STATUSES, STATUS_DISPLAY, STATUS_COLORS, CYCLE_STATUS_COLORS } from "@/types";
import SettingsModal from "@/components/SettingsModal.vue";
import CreateTodoModal from "@/components/CreateTodoModal.vue";
import TodoDetailModal from "@/components/TodoDetailModal.vue";
import SearchModal from "@/components/SearchModal.vue";

const store = useProjectStore();
const route = useRoute();
const router = useRouter();

const showSettings = ref(false);
const showCreate = ref(false);
const showSearch = ref(false);

function isTyping(): boolean {
  const tag = (document.activeElement as HTMLElement)?.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    !!(document.activeElement as HTMLElement)?.isContentEditable
  );
}

function handleGlobalKeydown(e: KeyboardEvent) {
  // Escape — clear multi-select (works even when typing)
  if (e.key === "Escape" && store.hasSelection) {
    e.preventDefault();
    store.clearSelection();
    return;
  }

  // CMD/Ctrl+K — search palette
  if ((e.metaKey || e.ctrlKey) && e.key === "k") {
    e.preventDefault();
    showSearch.value = true;
    return;
  }

  // CMD/Ctrl+A — select all (only on board view)
  if ((e.metaKey || e.ctrlKey) && e.key === "a" && !isTyping() && route.name === "board") {
    e.preventDefault();
    store.selectAll();
    return;
  }

  if (e.metaKey || e.ctrlKey || e.altKey) return;
  if (isTyping()) return;

  // C — create new todo
  if (e.key === "c") {
    e.preventDefault();
    showCreate.value = true;
  }
}

onMounted(() => {
  store.load();
  store.connectWebSocket();
  window.addEventListener("keydown", handleGlobalKeydown);
});

onUnmounted(() => {
  window.removeEventListener("keydown", handleGlobalKeydown);
});

const activeKey = computed(() => (route.name as string) ?? "board");

const navItems = [
  { label: "Inbox", key: "inbox", icon: Inbox },
  { label: "Board", key: "board", icon: LayoutKanban },
  { label: "Sprints", key: "cycles", icon: LayoutRows },
  { label: "Table", key: "list", icon: Table },
  { label: "Members", key: "members", icon: Users },
  { label: "Statistics", key: "statistics", icon: ChartDots },
];

function navigate(key: string) {
  router.push({ name: key });
}

const themeOverrides = {
  Layout: {
    color: "#f0f0f0",
    siderColor: "#f0f0f0",
  },
};
</script>

<template>
  <NConfigProvider :theme-overrides="themeOverrides">
    <NMessageProvider placement="bottom-right">
      <NLayout has-sider class="app-layout">
        <!-- Sidebar -->
        <NLayoutSider :width="220" content-class="sidebar-content">
          <template v-if="store.project">
            <!-- Project title + settings -->
            <div class="sidebar-header">
              <div class="project-badge">{{ store.prefix }}</div>
              <div class="header-text">
                <h1 class="app-title">{{ store.projectName }}</h1>
                <span class="app-subtitle">{{ store.todos.length }} items</span>
              </div>
              <button
                class="icon-btn settings-icon-btn"
                title="Settings"
                @click="showSettings = true"
              >
                <NIcon :size="20"><Settings /></NIcon>
              </button>
            </div>

            <!-- Navigation -->
            <nav class="sidebar-nav">
              <button
                v-for="item in navItems"
                :key="item.key"
                class="nav-item"
                :class="{ active: activeKey === item.key }"
                @click="navigate(item.key)"
              >
                <NIcon :size="18"><component :is="item.icon" /></NIcon>
                {{ item.label }}
              </button>
              <button class="nav-item search-trigger" @click="showSearch = true">
                <NIcon :size="18"><Search /></NIcon>
                Search
                <kbd class="kbd-hint">&#8984;K</kbd>
              </button>
            </nav>

            <!-- Status overview -->
            <div class="sidebar-section">
              <div class="section-label">Status</div>
              <div class="status-counts">
                <div v-for="s in STATUSES" :key="s" class="status-row">
                  <span class="status-dot" :style="{ background: STATUS_COLORS[s] }" />
                  <span class="status-name">{{ STATUS_DISPLAY[s] }}</span>
                  <span class="status-count">{{ store.statusCounts[s] }}</span>
                </div>
              </div>
            </div>

            <!-- Members -->
            <div class="sidebar-section">
              <div class="section-label">Members</div>
              <div class="member-list">
                <div
                  v-for="m in store.members.filter((m) => m.role !== 'agent')"
                  :key="m.id"
                  class="member-row"
                >
                  <span class="member-avatar">{{ m.name.charAt(0).toUpperCase() }}</span>
                  <span class="member-name">{{ m.name }}</span>
                  <NTag size="tiny" :bordered="false" round>{{ m.role }}</NTag>
                </div>
              </div>
            </div>

            <!-- Cycles -->
            <div class="sidebar-section">
              <div class="section-label">Sprints</div>
              <div class="cycle-list">
                <button
                  v-if="store.activeCycleId"
                  class="cycle-row cycle-clear"
                  @click="store.setActiveCycle(null)"
                >
                  <span class="cycle-name">All todos</span>
                </button>
                <button
                  v-for="c in store.cycles"
                  :key="c.id"
                  class="cycle-row"
                  :class="{ active: store.activeCycleId === c.id }"
                  @click="store.setActiveCycle(c.id)"
                >
                  <span class="status-dot" :style="{ background: CYCLE_STATUS_COLORS[c.status] }" />
                  <span class="cycle-name">{{ c.name }}</span>
                  <span class="cycle-count">{{
                    store.todos.filter((t) => t.cycleId === c.id).length
                  }}</span>
                </button>
                <span v-if="store.cycles.length === 0" class="cycle-empty">No cycles yet</span>
              </div>
            </div>
          </template>
        </NLayoutSider>

        <!-- Main content -->
        <NLayout>
          <NLayoutContent class="main-content">
            <div v-if="store.loading && !store.project" class="loading">Loading project...</div>
            <RouterView v-else />
          </NLayoutContent>
        </NLayout>
      </NLayout>
      <TodoDetailModal />
      <CreateTodoModal :open="showCreate" @close="showCreate = false" />
      <SettingsModal :open="showSettings" @close="showSettings = false" />
      <SearchModal :open="showSearch" @close="showSearch = false" />
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family:
    -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  overscroll-behavior: none;
}

html,
body {
  height: 100%;
  overflow: hidden;
}

.app-layout {
  height: 100vh;
  background: #f0f0f0;
}

.sidebar-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #f0f0f0;
  gap: 14px;
  overflow-y: auto;
  padding: 0 0 0 10px;
}

/* ── Project header ── */

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 0 0 4px;
}

.project-badge {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: #333;
  color: #fff;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.3px;
  flex-shrink: 0;
}

.header-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 0;
}

.app-title {
  font-size: 14px;
  font-weight: 700;
  line-height: 1;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.app-subtitle {
  font-size: 11px;
  line-height: 1;
  margin-top: 0;
  opacity: 0.45;
}

.icon-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0.35;
  transition:
    opacity 0.15s,
    background 0.15s;
  flex-shrink: 0;
  color: inherit;
}

.icon-btn:hover {
  opacity: 0.7;
  background: rgba(0, 0, 0, 0.06);
}

/* ── Navigation ── */

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border: none;
  background: transparent;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  color: inherit;
  opacity: 0.6;
  transition:
    background 0.12s,
    opacity 0.12s;
  text-align: left;
  font-family: inherit;
}

.nav-item:hover {
  background: rgba(0, 0, 0, 0.05);
  opacity: 0.85;
}

.nav-item.active {
  background: rgba(0, 0, 0, 0.08);
  opacity: 1;
  font-weight: 600;
}

.search-trigger .kbd-hint {
  margin-left: auto;
  font-size: 10px;
  font-family: inherit;
  padding: 1px 5px;
  border-radius: 3px;
  background: rgba(0, 0, 0, 0.08);
  color: inherit;
  opacity: 0.5;
}

.nav-icon {
  width: 20px;
  text-align: center;
  font-size: 14px;
  flex-shrink: 0;
}

/* ── Sections ── */

.sidebar-section {
  padding: 0 0 0 4px;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  opacity: 0.35;
  margin-bottom: 4px;
}

/* ── Status counts ── */

.status-counts {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding-left: 4px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  padding: 2px 0;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-name {
  flex: 1;
  opacity: 0.7;
}

.status-count {
  font-weight: 600;
  font-size: 12px;
  opacity: 0.4;
  font-variant-numeric: tabular-nums;
  min-width: 16px;
  text-align: right;
}

/* ── Members ── */

.member-list {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.member-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.member-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: #ddd;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  color: #666;
  flex-shrink: 0;
}

.agent-avatar {
  background: #8b5cf622 !important;
  color: #8b5cf6 !important;
  font-size: 9px !important;
  letter-spacing: 0.3px;
}

.member-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Cycles ── */

.cycle-list {
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.cycle-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px;
  border: none;
  background: transparent;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  color: inherit;
  opacity: 0.7;
  text-align: left;
  font-family: inherit;
  width: 100%;
}

.cycle-row:hover {
  background: rgba(0, 0, 0, 0.05);
  opacity: 0.9;
}

.cycle-row.active {
  background: rgba(0, 0, 0, 0.08);
  opacity: 1;
  font-weight: 600;
}

.cycle-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cycle-empty {
  font-size: 12px;
  opacity: 0.35;
  padding: 2px 8px;
}

.cycle-clear {
  opacity: 0.45;
  font-style: italic;
}

.cycle-count {
  font-weight: 600;
  font-size: 12px;
  opacity: 0.4;
  font-variant-numeric: tabular-nums;
  min-width: 16px;
  text-align: right;
}

/* ── Main content ── */

.main-content {
  height: calc(100vh - 20px);
  overflow-x: hidden !important;
  overflow-y: auto !important;
  background: #ffffff;
  margin: 10px;
  border-radius: 6px;
  border: 1px solid #d9d9d9;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  opacity: 0.5;
  font-size: 14px;
}

/* Nuke every transition and animation */
*,
*::before,
*::after {
  transition: none !important;
  animation: none !important;
}

/* Force modal content visible immediately — overrides JS inline styles */
.n-modal-body-wrapper {
  opacity: 1 !important;
  transform: none !important;
  transform-origin: center center !important;
}

.n-modal-mask {
  opacity: 1 !important;
}

.v-binder-follower-content {
  opacity: 1 !important;
}
</style>
