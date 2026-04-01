<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useProjectStore } from "@/stores/project";
import {
  fetchStats,
  type ProjectStats,
  type CountEntry,
  type MemberStatsEntry,
  type CycleStatsEntry,
} from "@/api";
import {
  STATUS_COLORS,
  PRIORITY_COLORS,
  DIFFICULTY_COLORS,
  LABEL_COLORS,
  CYCLE_STATUS_COLORS,
  CYCLE_STATUS_DISPLAY,
  type Status,
  type Priority,
  type Difficulty,
  type Label,
  type CycleStatus,
} from "@/types";

const store = useProjectStore();
const stats = ref<ProjectStats | null>(null);
const loading = ref(false);

async function loadStats() {
  loading.value = true;
  try {
    stats.value = await fetchStats();
  } finally {
    loading.value = false;
  }
}

onMounted(loadStats);

// Color lookups by key
function statusColor(key: string): string {
  return STATUS_COLORS[key as Status] ?? "#9ca3af";
}

function priorityColor(key: string): string {
  return PRIORITY_COLORS[key as Priority] ?? "#d4d4d8";
}

function difficultyColor(key: string): string {
  return DIFFICULTY_COLORS[key as Difficulty] ?? "#d4d4d8";
}

function labelColor(key: string): string {
  return LABEL_COLORS[key as Label] ?? "#9ca3af";
}

function cycleStatusColor(status: string): string {
  return CYCLE_STATUS_COLORS[status as CycleStatus] ?? "#9ca3af";
}

function cycleStatusLabel(status: string): string {
  return CYCLE_STATUS_DISPLAY[status as CycleStatus] ?? status;
}

// Compute percentages for bar widths
function pct(count: number, total: number): number {
  return total > 0 ? (count / total) * 100 : 0;
}

// ── Recent activity (still from store since audit log isn't in stats endpoint) ──
const recentActivity = computed(() => {
  return [...store.auditLog]
    .sort((a, b) => b.timestamp - a.timestamp)
    .slice(0, 15)
    .map((entry) => ({
      ...entry,
      timeAgo: formatTimeAgo(entry.timestamp),
    }));
});

function formatTimeAgo(ts: number): string {
  const diff = Date.now() - ts;
  const minutes = Math.floor(diff / 60000);
  if (minutes < 1) return "just now";
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  if (days < 7) return `${days}d ago`;
  return new Date(ts).toLocaleDateString();
}

function formatAction(action: string): string {
  return action.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}
</script>

<template>
  <div class="stats-view">
    <div class="stats-header">
      <h2>Statistics</h2>
    </div>

    <div v-if="loading && !stats" class="loading">Loading statistics...</div>

    <div v-else-if="stats" class="stats-scroll">
      <!-- Summary cards -->
      <div class="summary-row">
        <div class="summary-card">
          <div class="summary-value">{{ stats.summary.total }}</div>
          <div class="summary-label">Total</div>
        </div>
        <div class="summary-card">
          <div class="summary-value">{{ stats.summary.active }}</div>
          <div class="summary-label">Active</div>
        </div>
        <div class="summary-card">
          <div class="summary-value">{{ stats.summary.inProgress }}</div>
          <div class="summary-label">In Progress</div>
        </div>
        <div class="summary-card">
          <div class="summary-value">{{ stats.summary.completed }}</div>
          <div class="summary-label">Completed</div>
        </div>
        <div class="summary-card">
          <div class="summary-value">{{ stats.summary.unassigned }}</div>
          <div class="summary-label">Unassigned</div>
        </div>
        <div class="summary-card">
          <div class="summary-value">{{ stats.summary.completionRate }}%</div>
          <div class="summary-label">Closed Rate</div>
        </div>
      </div>

      <div class="stats-grid">
        <!-- Status distribution -->
        <div class="stats-card">
          <div class="card-title">Status Distribution</div>
          <div class="bar-list">
            <div v-for="s in stats.byStatus" :key="s.key" class="bar-row">
              <div class="bar-label">
                <span class="bar-dot" :style="{ background: statusColor(s.key) }" />
                {{ s.label }}
              </div>
              <div class="bar-track">
                <div
                  class="bar-fill"
                  :style="{
                    width: pct(s.count, stats.summary.total) + '%',
                    background: statusColor(s.key),
                  }"
                />
              </div>
              <div class="bar-value">{{ s.count }}</div>
            </div>
          </div>
        </div>

        <!-- Priority distribution -->
        <div class="stats-card">
          <div class="card-title">Priority Breakdown</div>
          <div class="bar-list">
            <div v-for="p in stats.byPriority" :key="p.key" class="bar-row">
              <div class="bar-label">
                <span class="bar-dot" :style="{ background: priorityColor(p.key) }" />
                {{ p.label }}
              </div>
              <div class="bar-track">
                <div
                  class="bar-fill"
                  :style="{
                    width: pct(p.count, stats.summary.active) + '%',
                    background: priorityColor(p.key),
                  }"
                />
              </div>
              <div class="bar-value">{{ p.count }}</div>
            </div>
          </div>
        </div>

        <!-- Difficulty distribution -->
        <div class="stats-card">
          <div class="card-title">Difficulty Breakdown</div>
          <div class="bar-list">
            <div v-for="d in stats.byDifficulty" :key="d.key" class="bar-row">
              <div class="bar-label">
                <span class="bar-dot" :style="{ background: difficultyColor(d.key) }" />
                {{ d.label }}
              </div>
              <div class="bar-track">
                <div
                  class="bar-fill"
                  :style="{
                    width: pct(d.count, stats.summary.active) + '%',
                    background: difficultyColor(d.key),
                  }"
                />
              </div>
              <div class="bar-value">{{ d.count }}</div>
            </div>
          </div>
        </div>

        <!-- Labels -->
        <div class="stats-card">
          <div class="card-title">Labels</div>
          <div v-if="stats.byLabel.length" class="bar-list">
            <div v-for="l in stats.byLabel" :key="l.key" class="bar-row">
              <div class="bar-label">
                <span class="bar-dot" :style="{ background: labelColor(l.key) }" />
                {{ l.label }}
              </div>
              <div class="bar-track">
                <div
                  class="bar-fill"
                  :style="{
                    width:
                      pct(
                        l.count,
                        stats.byLabel.reduce((a, b) => a + b.count, 0),
                      ) + '%',
                    background: labelColor(l.key),
                  }"
                />
              </div>
              <div class="bar-value">{{ l.count }}</div>
            </div>
          </div>
          <div v-else class="empty-note">No labels assigned</div>
        </div>

        <!-- Member workload -->
        <div class="stats-card">
          <div class="card-title">Member Workload</div>
          <div v-if="stats.members.length" class="member-workload">
            <div v-for="ms in stats.members" :key="ms.id" class="workload-row">
              <span class="workload-avatar">{{ ms.name.charAt(0).toUpperCase() }}</span>
              <span class="workload-name">{{ ms.name }}</span>
              <div class="workload-bars">
                <span class="workload-chip active">{{ ms.active }} active</span>
                <span class="workload-chip done">{{ ms.completed }} done</span>
              </div>
            </div>
          </div>
          <div v-else class="empty-note">No assigned todos</div>
        </div>

        <!-- Sprint velocity -->
        <div class="stats-card">
          <div class="card-title">Sprint Progress</div>
          <div v-if="stats.cycles.length" class="sprint-list">
            <div v-for="cs in stats.cycles" :key="cs.id" class="sprint-row">
              <div class="sprint-header">
                <span class="sprint-name">{{ cs.name }}</span>
                <span class="sprint-status" :style="{ color: cycleStatusColor(cs.status) }">{{
                  cycleStatusLabel(cs.status)
                }}</span>
              </div>
              <div class="sprint-progress-track">
                <div
                  class="sprint-progress-fill"
                  :style="{ width: cs.pctDone + '%', background: cycleStatusColor(cs.status) }"
                />
              </div>
              <div class="sprint-meta">
                <span>{{ cs.completed }}/{{ cs.total }} completed ({{ cs.pctDone }}%)</span>
                <span v-if="cs.daysTotal !== null">
                  Day {{ cs.daysElapsed }}/{{ cs.daysTotal }}
                </span>
              </div>
            </div>
          </div>
          <div v-else class="empty-note">No sprints created</div>
        </div>
      </div>

      <!-- Recent activity -->
      <div class="stats-card activity-card">
        <div class="card-title">Recent Activity</div>
        <div v-if="recentActivity.length" class="activity-list">
          <div v-for="(entry, i) in recentActivity" :key="i" class="activity-row">
            <span class="activity-time">{{ entry.timeAgo }}</span>
            <span class="activity-actor">{{ entry.actorName }}</span>
            <span class="activity-action">{{ formatAction(entry.action) }}</span>
            <span v-if="entry.target" class="activity-target">{{ entry.target }}</span>
          </div>
        </div>
        <div v-else class="empty-note">No activity yet</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stats-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.stats-header {
  display: flex;
  align-items: center;
  padding: 16px 24px;
  flex-shrink: 0;
}

.stats-header h2 {
  font-size: 16px;
  font-weight: 600;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  opacity: 0.5;
  font-size: 14px;
}

.stats-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0 24px 24px;
}

/* ── Summary row ── */

.summary-row {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.summary-card {
  flex: 1;
  padding: 14px 16px;
  border-radius: 8px;
  background: #f8f8f8;
  border: 1px solid #eee;
  text-align: center;
}

.summary-value {
  font-size: 24px;
  font-weight: 700;
  line-height: 1.2;
  font-variant-numeric: tabular-nums;
}

.summary-label {
  font-size: 11px;
  font-weight: 500;
  opacity: 0.45;
  margin-top: 2px;
  text-transform: uppercase;
  letter-spacing: 0.3px;
}

/* ── Stats grid ── */

.stats-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 16px;
}

.stats-card {
  padding: 16px;
  border-radius: 8px;
  background: #f8f8f8;
  border: 1px solid #eee;
}

.card-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  opacity: 0.45;
  margin-bottom: 12px;
}

.empty-note {
  font-size: 13px;
  opacity: 0.35;
  padding: 8px 0;
}

/* ── Bar charts ── */

.bar-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bar-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.bar-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  width: 120px;
  flex-shrink: 0;
}

.bar-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.bar-track {
  flex: 1;
  height: 6px;
  border-radius: 3px;
  background: #e8e8e8;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  border-radius: 3px;
  min-width: 2px;
}

.bar-value {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.5;
  width: 28px;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

/* ── Member workload ── */

.member-workload {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.workload-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.workload-avatar {
  width: 26px;
  height: 26px;
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

.workload-name {
  font-size: 13px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.workload-bars {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.workload-chip {
  font-size: 11px;
  font-weight: 500;
  padding: 2px 8px;
  border-radius: 10px;
}

.workload-chip.active {
  background: #f59e0b22;
  color: #b45309;
}

.workload-chip.done {
  background: #10b98122;
  color: #047857;
}

/* ── Sprint progress ── */

.sprint-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.sprint-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.sprint-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.sprint-name {
  font-size: 13px;
  font-weight: 600;
}

.sprint-status {
  font-size: 11px;
  font-weight: 600;
}

.sprint-progress-track {
  height: 6px;
  border-radius: 3px;
  background: #e8e8e8;
  overflow: hidden;
}

.sprint-progress-fill {
  height: 100%;
  border-radius: 3px;
  min-width: 2px;
}

.sprint-meta {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  opacity: 0.45;
}

/* ── Activity feed ── */

.activity-card {
  margin-bottom: 0;
}

.activity-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.activity-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  padding: 4px 0;
  border-bottom: 1px solid #f0f0f0;
}

.activity-row:last-child {
  border-bottom: none;
}

.activity-time {
  width: 64px;
  flex-shrink: 0;
  opacity: 0.4;
  font-variant-numeric: tabular-nums;
}

.activity-actor {
  font-weight: 600;
  flex-shrink: 0;
}

.activity-action {
  opacity: 0.6;
  flex-shrink: 0;
}

.activity-target {
  font-family: monospace;
  font-size: 11px;
  opacity: 0.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
