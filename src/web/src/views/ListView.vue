<script setup lang="ts">
import { ref, computed, h, type Component } from 'vue'
import {
  NButton,
  NDataTable,
  NIcon,
  NInput,
  NSelect,
  NSpace,
  NTooltip,
  type DataTableColumns,
} from 'naive-ui'
import {
  AntennaBars1,
  AntennaBars2,
  AntennaBars3,
  AntennaBars4,
  AntennaBars5,
} from '@vicons/tabler'
import { useProjectStore } from '@/stores/project'
import type { Todo, Status, Priority, Label } from '@/types'
import {
  STATUSES,
  PRIORITIES,
  STATUS_DISPLAY,
  PRIORITY_DISPLAY,
  STATUS_COLORS,
  PRIORITY_COLORS,
  LABEL_DISPLAY,
  LABEL_COLORS,
} from '@/types'

import CreateTodoModal from '@/components/CreateTodoModal.vue'

const PRIORITY_ICON: Record<Priority, Component> = {
  none: AntennaBars1,
  low: AntennaBars2,
  medium: AntennaBars3,
  high: AntennaBars4,
  urgent: AntennaBars5,
}

function renderPriority(icon: Component, color: string, label: string) {
  return h(
    NTooltip,
    { trigger: 'hover' },
    {
      trigger: () => h(NIcon, { size: 20, color }, { default: () => h(icon) }),
      default: () => label,
    },
  )
}

const store = useProjectStore()

const showCreate = ref(false)
const filterStatus = ref<Status | null>(null)
const filterPriority = ref<Priority | null>(null)
const searchQuery = ref('')

const statusFilterOptions = STATUSES.map((s) => ({ label: STATUS_DISPLAY[s], value: s }))
const priorityFilterOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }))

function renderPriorityLabel(option: { label: string; value: string }) {
  const p = option.value as Priority
  return h('span', { style: 'display: flex; align-items: center; gap: 8px' }, [
    h(NIcon, { size: 16, color: PRIORITY_COLORS[p] }, { default: () => h(PRIORITY_ICON[p]) }),
    option.label,
  ])
}

const filteredTodos = computed(() => {
  let list = store.activeTodos
  if (filterStatus.value) {
    list = list.filter((t) => t.status === filterStatus.value)
  }
  if (filterPriority.value) {
    list = list.filter((t) => t.priority === filterPriority.value)
  }
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    list = list.filter(
      (t) => t.title.toLowerCase().includes(q) || t.description.toLowerCase().includes(q),
    )
  }
  return list
})

const columns: DataTableColumns<Todo> = [
  {
    title: 'Ref',
    key: 'ref',
    width: 90,
    render(row) {
      return h('span', { style: 'font-family: monospace; font-size: 12px; opacity: 0.5' }, row.ref)
    },
  },
  {
    title: 'Title',
    key: 'title',
    ellipsis: { tooltip: true },
    render(row) {
      const style = row.status === 'completed' ? 'text-decoration: line-through; opacity: 0.6' : ''
      return h('span', { style }, row.title)
    },
  },
  {
    title: 'Status',
    key: 'status',
    width: 120,
    render(row) {
      const status = row.status
      if (!status) return h('span', { style: 'opacity: 0.3' }, '\u2014')
      return h('span', { style: 'display: flex; align-items: center; gap: 6px; font-size: 12px' }, [
        h('span', {
          style: `width: 8px; height: 8px; border-radius: 50%; background: ${STATUS_COLORS[status]}; flex-shrink: 0`,
        }),
        STATUS_DISPLAY[status],
      ])
    },
  },
  {
    title: 'Priority',
    key: 'priority',
    width: 70,
    align: 'center',
    render(row) {
      return renderPriority(
        PRIORITY_ICON[row.priority],
        PRIORITY_COLORS[row.priority],
        PRIORITY_DISPLAY[row.priority],
      )
    },
  },
  {
    title: 'Labels',
    key: 'labels',
    width: 160,
    render(row) {
      if (!row.labels?.length) return null
      return h(
        'span',
        { style: 'display: flex; flex-wrap: wrap; gap: 4px' },
        row.labels.map((l: Label) =>
          h(
            'span',
            {
              style: `font-size: 10px; font-weight: 600; padding: 1px 6px; border-radius: 8px; white-space: nowrap; background: ${LABEL_COLORS[l]}22; color: ${LABEL_COLORS[l]}`,
            },
            LABEL_DISPLAY[l],
          ),
        ),
      )
    },
  },
  {
    title: 'Assignee',
    key: 'assigneeName',
    width: 120,
    render(row) {
      return row.assigneeName ? h('span', {}, row.assigneeName) : null
    },
  },
]

function handleRowClick(row: Todo) {
  store.openTodo(row.number)
}

const rowProps = (row: Todo) => ({
  style: 'cursor: pointer',
  onClick: () => handleRowClick(row),
})
</script>

<template>
  <div class="list-view">
    <div class="list-toolbar">
      <h2>Table</h2>
      <NSpace :size="8" align="center" class="list-filters">
        <NInput
          v-model:value="searchQuery"
          placeholder="Search..."
          clearable
          size="small"
          style="width: 200px"
        />
        <NSelect
          v-model:value="filterStatus"
          :options="statusFilterOptions"
          placeholder="Status"
          clearable
          size="small"
          style="width: 140px"
        />
        <NSelect
          v-model:value="filterPriority"
          :options="priorityFilterOptions"
          :render-label="renderPriorityLabel"
          placeholder="Priority"
          clearable
          size="small"
          style="width: 160px"
        />
      </NSpace>
      <NButton type="primary" size="small" @click="showCreate = true">+ New Todo</NButton>
    </div>

    <div class="list-table-container">
      <NDataTable
        :columns="columns"
        :data="filteredTodos"
        :row-props="rowProps"
        :bordered="false"
        size="small"
      />
    </div>

    <CreateTodoModal :open="showCreate" @close="showCreate = false" />
  </div>
</template>

<style scoped>
.list-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.list-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  gap: 16px;
  flex-shrink: 0;
}

.list-toolbar h2 {
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
}

.list-filters {
  flex: 1;
  justify-content: center;
}

.list-table-container {
  flex: 1;
  padding: 0 24px 24px;
  overflow: auto;
}
</style>
