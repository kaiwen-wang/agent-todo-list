<script setup lang="ts">
import { onMounted } from 'vue'
import { RouterView, RouterLink, useRoute } from 'vue-router'
import { useProjectStore } from '@/stores/project'

const store = useProjectStore()
const route = useRoute()

onMounted(() => {
  store.load()
})
</script>

<template>
  <div class="app">
    <header class="app-header">
      <div class="header-left">
        <h1 class="app-title">{{ store.projectName || 'agt' }}</h1>
        <span class="prefix-badge" v-if="store.prefix">{{ store.prefix }}</span>
      </div>
      <nav class="header-nav">
        <RouterLink to="/board" class="nav-link" :class="{ active: route.name === 'board' }">
          Board
        </RouterLink>
        <RouterLink to="/list" class="nav-link" :class="{ active: route.name === 'list' }">
          List
        </RouterLink>
      </nav>
      <div class="header-right">
        <div class="status-summary" v-if="store.project">
          <span class="count-badge todo">{{ store.statusCounts.todo }} todo</span>
          <span class="count-badge in-progress">{{ store.statusCounts.in_progress }} active</span>
          <span class="count-badge done">{{ store.statusCounts.done }} done</span>
        </div>
      </div>
    </header>

    <main class="app-main">
      <div v-if="store.loading && !store.project" class="loading">Loading project...</div>
      <div v-else-if="store.error && !store.project" class="error-banner">{{ store.error }}</div>
      <RouterView v-else />
    </main>
  </div>
</template>

<style>
/* Reset & base */
*,
*::before,
*::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

:root {
  --bg: #0f1117;
  --bg-surface: #1a1d27;
  --bg-card: #232631;
  --bg-hover: #2a2e3d;
  --border: #2e3345;
  --text: #e4e6ed;
  --text-dim: #8b8fa3;
  --text-muted: #5c6078;
  --accent: #6366f1;
  --accent-hover: #818cf8;
  --success: #10b981;
  --warning: #f59e0b;
  --danger: #ef4444;
  --info: #3b82f6;
  --radius: 8px;
  --radius-sm: 4px;
  --shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
}

body {
  font-family:
    -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: var(--bg);
  color: var(--text);
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
}

.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

/* Header */
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  height: 56px;
  background: var(--bg-surface);
  border-bottom: 1px solid var(--border);
  position: sticky;
  top: 0;
  z-index: 100;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.app-title {
  font-size: 18px;
  font-weight: 700;
  color: var(--text);
}

.prefix-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  background: var(--accent);
  color: white;
  border-radius: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.header-nav {
  display: flex;
  gap: 2px;
}

.nav-link {
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-dim);
  text-decoration: none;
  border-radius: var(--radius-sm);
  transition: all 0.15s;
}

.nav-link:hover {
  color: var(--text);
  background: var(--bg-hover);
}

.nav-link.active {
  color: var(--text);
  background: var(--bg-card);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-summary {
  display: flex;
  gap: 8px;
  font-size: 12px;
}

.count-badge {
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: 500;
}

.count-badge.todo {
  background: rgba(59, 130, 246, 0.15);
  color: #60a5fa;
}

.count-badge.in-progress {
  background: rgba(245, 158, 11, 0.15);
  color: #fbbf24;
}

.count-badge.done {
  background: rgba(16, 185, 129, 0.15);
  color: #34d399;
}

/* Main */
.app-main {
  flex: 1;
  overflow: hidden;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-dim);
  font-size: 14px;
}

.error-banner {
  margin: 16px;
  padding: 12px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: var(--radius);
  color: #fca5a5;
  font-size: 14px;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  font-size: 13px;
  font-weight: 500;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg-card);
  color: var(--text);
  cursor: pointer;
  transition: all 0.15s;
  white-space: nowrap;
}

.btn:hover {
  background: var(--bg-hover);
  border-color: var(--text-muted);
}

.btn-primary {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
}

.btn-primary:hover {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
}

.btn-sm {
  padding: 4px 10px;
  font-size: 12px;
}

.btn-danger {
  color: var(--danger);
}

.btn-danger:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: var(--danger);
}

/* Form inputs */
input[type='text'],
textarea,
select {
  width: 100%;
  padding: 8px 12px;
  font-size: 13px;
  background: var(--bg);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  color: var(--text);
  outline: none;
  transition: border-color 0.15s;
}

input[type='text']:focus,
textarea:focus,
select:focus {
  border-color: var(--accent);
}

textarea {
  resize: vertical;
  min-height: 80px;
  font-family: inherit;
}

select {
  cursor: pointer;
}

label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-dim);
  margin-bottom: 4px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
</style>
