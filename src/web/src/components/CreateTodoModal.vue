<script setup lang="ts">
import { ref, watch, computed, h, type Component } from 'vue'
import {
  NModal,
  NCard,
  NForm,
  NFormItem,
  NIcon,
  NInput,
  NSelect,
  NButton,
  NSpace,
  useMessage,
} from 'naive-ui'
import {
  AntennaBars1,
  AntennaBars2,
  AntennaBars3,
  AntennaBars4,
  AntennaBars5,
} from '@vicons/tabler'
import type { Status, Priority, Label } from '@/types'
import {
  STATUSES,
  PRIORITIES,
  LABELS,
  STATUS_DISPLAY,
  PRIORITY_DISPLAY,
  PRIORITY_COLORS,
  STATUS_COLORS,
  LABEL_DISPLAY,
  LABEL_COLORS,
} from '@/types'
import { useProjectStore } from '@/stores/project'

const PRIORITY_ICON: Record<Priority, Component> = {
  none: AntennaBars1,
  low: AntennaBars2,
  medium: AntennaBars3,
  high: AntennaBars4,
  urgent: AntennaBars5,
}

const props = defineProps<{
  open: boolean
  defaultStatus?: Status
}>()

const emit = defineEmits<{
  close: []
}>()

const store = useProjectStore()
const message = useMessage()

const title = ref('')
const description = ref('')
const status = ref<Status>(props.defaultStatus ?? 'todo')
const priority = ref<Priority>('none')
const labels = ref<Label[]>([])
const assignee = ref<string | null>(null)
const submitting = ref(false)

const memberOptions = computed(() =>
  store.members.map((m) => ({ label: m.name, value: m.id }))
)

const statusOptions = STATUSES.map((s) => ({ label: STATUS_DISPLAY[s], value: s }))
const priorityOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }))
const labelOptions = LABELS.map((l) => ({ label: LABEL_DISPLAY[l], value: l }))

function renderStatusLabel(option: { label: string; value: string }) {
  const s = option.value as Status
  return h('span', { style: 'display: flex; align-items: center; gap: 8px' }, [
    h('span', { style: `width: 8px; height: 8px; border-radius: 50%; background: ${STATUS_COLORS[s]}; flex-shrink: 0` }),
    option.label,
  ])
}

function renderPriorityLabel(option: { label: string; value: string }) {
  const p = option.value as Priority
  return h('span', { style: 'display: flex; align-items: center; gap: 8px' }, [
    h(NIcon, { size: 16, color: PRIORITY_COLORS[p] }, { default: () => h(PRIORITY_ICON[p]) }),
    option.label,
  ])
}

function renderLabelTag(option: { label: string; value: string }) {
  const l = option.value as Label
  return h('span', { style: 'display: flex; align-items: center; gap: 6px' }, [
    h('span', { style: `width: 8px; height: 8px; border-radius: 50%; background: ${LABEL_COLORS[l]}; flex-shrink: 0` }),
    option.label,
  ])
}

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      title.value = ''
      description.value = ''
      status.value = props.defaultStatus ?? 'todo'
      priority.value = 'none'
      labels.value = []
      assignee.value = null
    }
  },
)

async function submit() {
  if (!title.value.trim()) return
  submitting.value = true
  try {
    await store.addTodo({
      title: title.value.trim(),
      description: description.value.trim() || undefined,
      status: status.value,
      priority: priority.value,
      labels: labels.value.length ? labels.value : undefined,
      assignee: assignee.value,
    })
    message.success('Todo created')
    emit('close')
  } catch {
    message.error('Failed to create todo')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <NModal :show="open" @update:show="(v: boolean) => !v && emit('close')">
    <NCard
      title="New Todo"
      :bordered="true"
      closable
      @close="emit('close')"
      style="width: 520px; max-width: 95vw"
      role="dialog"
    >
      <NForm @submit.prevent="submit" label-placement="top">
        <NFormItem label="Title">
          <NInput
            v-model:value="title"
            placeholder="What needs to be done?"
            autofocus
            @keydown.enter.prevent="submit"
          />
        </NFormItem>

        <NFormItem label="Description">
          <NInput
            v-model:value="description"
            type="textarea"
            placeholder="Add details... (optional)"
            :rows="3"
          />
        </NFormItem>

        <NSpace :size="12">
          <NFormItem label="Status" style="flex: 1">
            <NSelect v-model:value="status" :options="statusOptions" :render-label="renderStatusLabel" />
          </NFormItem>
          <NFormItem label="Priority" style="flex: 1">
            <NSelect
              v-model:value="priority"
              :options="priorityOptions"
              :render-label="renderPriorityLabel"
            />
          </NFormItem>
        </NSpace>

        <NFormItem label="Labels">
          <NSelect
            v-model:value="labels"
            :options="labelOptions"
            :render-label="renderLabelTag"
            multiple
            clearable
            placeholder="Add labels..."
          />
        </NFormItem>

        <NFormItem label="Assignee">
          <NSelect
            v-model:value="assignee"
            :options="memberOptions"
            placeholder="Assign to a member..."
            clearable
          />
        </NFormItem>

        <NSpace justify="end" :size="8">
          <NButton @click="emit('close')">Cancel</NButton>
          <NButton type="primary" @click="submit" :loading="submitting" :disabled="!title.trim()">
            Create Todo
          </NButton>
        </NSpace>
      </NForm>
    </NCard>
  </NModal>
</template>
