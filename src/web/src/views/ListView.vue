<script setup lang="ts">
import { ref, computed, h } from 'vue'
import {
  NButton,
  NDataTable,
  NInput,
  NSelect,
  NSpace,
  NTag,
  type DataTableColumns,
} from 'naive-ui'
import { useProjectStore } from '@/stores/project'
import type { Todo, Status, Priority } from '@/types'
import {
  STATUSES,
  PRIORITIES,
  STATUS_DISPLAY,
  PRIORITY_DISPLAY,
  STATUS_COLORS,
  PRIORITY_COLORS,
} from '@/types'
import CreateTodoModal from '@/components/CreateTodoModal.vue'

const store = useProjectStore()

const showCreate = ref(false)
const filterStatus = ref<Status | null>(null)
const filterPriority = ref<Priority | null>(null)
const searchQuery = ref('')

const statusFilterOptions = STATUSES.map((s) => ({ label: STATUS_DISPLAY[s], value: s }))
const priorityFilterOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }))

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
      const style = row.status === 'done' ? 'text-decoration: line-through; opacity: 0.6' : ''
      return h('span', { style }, row.title)
    },
  },
  {
    title: 'Status',
    key: 'status',
    width: 120,
    render(row) {
      return h(
        NTag,
        {
          size: 'small',
          round: true,
          bordered: false,
          color: { color: STATUS_COLORS[row.status] + '22', textColor: STATUS_COLORS[row.status] },
        },
        () => STATUS_DISPLAY[row.status],
      )
    },
  },
  {
    title: 'Priority',
    key: 'priority',
    width: 100,
    render(row) {
      return h('span', { style: 'display: flex; align-items: center; gap: 6px; font-size: 12px' }, [
        h('span', {
          style: `width: 8px; height: 8px; border-radius: 50%; background: ${PRIORITY_COLORS[row.priority]}; flex-shrink: 0`,
        }),
        PRIORITY_DISPLAY[row.priority],
      ])
    },
  },
  {
    title: 'Assignee',
    key: 'assigneeName',
    width: 120,
    render(row) {
      return row.assigneeName
        ? h('span', {}, row.assigneeName)
        : h('span', { style: 'opacity: 0.3' }, '\u2014')
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
      <h2>List</h2>
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
          placeholder="Priority"
          clearable
          size="small"
          style="width: 140px"
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
