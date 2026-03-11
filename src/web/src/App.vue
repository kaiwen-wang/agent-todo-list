<script setup lang="ts">
import { ref, onMounted, computed, h, type Component } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import {
  NConfigProvider,
  NMessageProvider,
  NLayout,
  NLayoutHeader,
  NLayoutSider,
  NLayoutContent,
  NMenu,
  NTag,
  NSpace,
  NDivider,
  NStatistic,
  NButton,
  type MenuOption,
} from 'naive-ui'
import { useProjectStore } from '@/stores/project'
import { STATUS_COLORS } from '@/types'
import SettingsModal from '@/components/SettingsModal.vue'
import TodoDetailModal from '@/components/TodoDetailModal.vue'

const store = useProjectStore()
const route = useRoute()
const router = useRouter()

const showSettings = ref(false)

onMounted(() => {
  store.load()
})

const activeKey = computed(() => (route.name as string) ?? 'board')

const menuOptions: MenuOption[] = [
  { label: 'Board', key: 'board' },
  { label: 'Table', key: 'list' },
  { label: 'Members', key: 'members' },
]

function handleMenuUpdate(key: string) {
  router.push({ name: key })
}

const themeOverrides = {
  Layout: {
    color: '#f0f0f0',
    siderColor: '#f0f0f0',
  },
}

</script>

<template>
  <NConfigProvider :theme-overrides="themeOverrides">
    <NMessageProvider>
      <NLayout has-sider class="app-layout">
        <!-- Sidebar -->
        <NLayoutSider
          :width="220"
          content-class="sidebar-content"
        >
          <template v-if="store.project">
            <div class="sidebar-header">
              <h1 class="app-title">{{ store.projectName }}</h1>
            </div>

            <NMenu
              :value="activeKey"
              :options="menuOptions"
              @update:value="handleMenuUpdate"
            />
            <NDivider style="margin: 12px 0" />

            <div class="sidebar-section">
              <div class="section-label">Status</div>
              <div class="status-counts">
                <div class="status-row">
                  <span class="status-dot" :style="{ background: STATUS_COLORS.none }" />
                  <span class="status-name">None</span>
                  <span class="status-count">{{ store.statusCounts.none }}</span>
                </div>
                <div class="status-row">
                  <span class="status-dot" :style="{ background: STATUS_COLORS.todo }" />
                  <span class="status-name">To Do</span>
                  <span class="status-count">{{ store.statusCounts.todo }}</span>
                </div>
                <div class="status-row">
                  <span class="status-dot" :style="{ background: STATUS_COLORS.in_progress }" />
                  <span class="status-name">In Progress</span>
                  <span class="status-count">{{ store.statusCounts.in_progress }}</span>
                </div>
                <div class="status-row">
                  <span class="status-dot" :style="{ background: STATUS_COLORS.completed }" />
                  <span class="status-name">Completed</span>
                  <span class="status-count">{{ store.statusCounts.completed }}</span>
                </div>
                <div class="status-row">
                  <span class="status-dot" :style="{ background: STATUS_COLORS.archived }" />
                  <span class="status-name">Archived</span>
                  <span class="status-count">{{ store.statusCounts.archived }}</span>
                </div>
                <div class="status-row">
                  <span class="status-dot" :style="{ background: STATUS_COLORS.wont_do }" />
                  <span class="status-name">Won't Do</span>
                  <span class="status-count">{{ store.statusCounts.wont_do }}</span>
                </div>
              </div>
            </div>

            <NDivider style="margin: 12px 0" />

            <div class="sidebar-section">
              <div class="section-label">Members</div>
              <div class="member-list">
                <div v-for="m in store.members" :key="m.id" class="member-row">
                  <span class="member-avatar">{{ m.name.charAt(0).toUpperCase() }}</span>
                  <span class="member-name">{{ m.name }}</span>
                  <NTag size="tiny" :bordered="false" round>{{ m.role }}</NTag>
                </div>
              </div>
            </div>

            <div class="sidebar-spacer" />
            <div class="sidebar-bottom">
              <NButton
                quaternary
                block
                size="medium"
                class="settings-btn"
                @click="showSettings = true"
              >
                <template #icon>
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>
                </template>
                Settings
              </NButton>
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
      <SettingsModal :open="showSettings" @close="showSettings = false" />
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
    -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
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
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 16px 16px 12px;
}

.app-title {
  font-size: 17px;
  font-weight: 700;
}

.sidebar-section {
  padding: 0 16px;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  opacity: 0.45;
  margin-bottom: 8px;
}

.status-counts {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-name {
  flex: 1;
}

.status-count {
  font-weight: 600;
  font-size: 12px;
  opacity: 0.6;
  font-variant-numeric: tabular-nums;
}

.member-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
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
  background: #e8e8e8;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 600;
  color: #666;
  flex-shrink: 0;
}

.member-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-spacer {
  flex: 1;
}

.sidebar-bottom {
  padding: 12px 16px;
  border-top: 1px solid rgba(0, 0, 0, 0.09);
}

.main-content {
  height: calc(100vh - 20px);
  overflow: auto;
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

/* Kill Naive UI modal/overlay animations */
.n-modal-mask,
.v-binder-follower-content {
  transition-duration: 0s !important;
}

.n-modal-body-wrapper {
  transition-duration: 0s !important;
  animation-duration: 0s !important;
}

.fade-in-scale-up-transition-enter-active,
.fade-in-scale-up-transition-leave-active,
.fade-in-transition-enter-active,
.fade-in-transition-leave-active {
  transition-duration: 0s !important;
}
</style>
