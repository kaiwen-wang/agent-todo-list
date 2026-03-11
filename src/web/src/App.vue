<script setup lang="ts">
import { onMounted, h, computed } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'
import {
  NConfigProvider,
  NMessageProvider,
  NLayout,
  NLayoutHeader,
  NSpace,
  NTag,
  NMenu,
  darkTheme,
  type MenuOption,
} from 'naive-ui'
import { useProjectStore } from '@/stores/project'

const store = useProjectStore()
const route = useRoute()
const router = useRouter()

onMounted(() => {
  store.load()
})

const activeKey = computed(() => (route.name as string) ?? 'board')

const menuOptions: MenuOption[] = [
  { label: 'Board', key: 'board' },
  { label: 'List', key: 'list' },
]

function handleMenuUpdate(key: string) {
  router.push({ name: key })
}
</script>

<template>
  <NConfigProvider :theme="darkTheme">
    <NMessageProvider>
      <NLayout class="app-layout">
        <NLayoutHeader class="app-header" bordered>
          <div class="header-left">
            <h1 class="app-title">{{ store.projectName || 'agt' }}</h1>
            <NTag v-if="store.prefix" type="info" size="small" round>
              {{ store.prefix }}
            </NTag>
          </div>

          <NMenu
            mode="horizontal"
            :value="activeKey"
            :options="menuOptions"
            @update:value="handleMenuUpdate"
            class="header-menu"
          />

          <NSpace class="header-right" :size="8" align="center">
            <NTag v-if="store.project" size="small" round type="info">
              {{ store.statusCounts.todo }} todo
            </NTag>
            <NTag v-if="store.project" size="small" round type="warning">
              {{ store.statusCounts.in_progress }} active
            </NTag>
            <NTag v-if="store.project" size="small" round type="success">
              {{ store.statusCounts.done }} done
            </NTag>
          </NSpace>
        </NLayoutHeader>

        <NLayout class="app-main" content-class="main-content">
          <div v-if="store.loading && !store.project" class="loading">Loading project...</div>
          <RouterView v-else />
        </NLayout>
      </NLayout>
    </NMessageProvider>
  </NConfigProvider>
</template>

<style>
/* Minimal global overrides — Naive UI handles most styling */
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
  min-height: 100vh;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  height: 52px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.app-title {
  font-size: 17px;
  font-weight: 700;
}

.header-menu {
  flex: 0 0 auto;
}

.header-right {
  flex-shrink: 0;
}

.main-content {
  height: calc(100vh - 52px);
  overflow: hidden;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  opacity: 0.5;
  font-size: 14px;
}
</style>
