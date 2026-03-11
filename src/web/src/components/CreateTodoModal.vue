<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import {
  NModal,
  NCard,
  NForm,
  NFormItem,
  NInput,
  NSelect,
  NButton,
  NSpace,
  useMessage,
} from 'naive-ui'
import type { Status, Priority } from '@/types'
import { STATUSES, PRIORITIES, STATUS_DISPLAY, PRIORITY_DISPLAY } from '@/types'
import { useProjectStore } from '@/stores/project'

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
const priority = ref<Priority>('medium')
const assignee = ref<string | null>(null)
const submitting = ref(false)

const memberOptions = computed(() =>
  store.members.map((m) => ({ label: m.name, value: m.id }))
)

const statusOptions = STATUSES.map((s) => ({ label: STATUS_DISPLAY[s], value: s }))
const priorityOptions = PRIORITIES.map((p) => ({ label: PRIORITY_DISPLAY[p], value: p }))

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      title.value = ''
      description.value = ''
      status.value = props.defaultStatus ?? 'todo'
      priority.value = 'medium'
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
            <NSelect v-model:value="status" :options="statusOptions" />
          </NFormItem>
          <NFormItem label="Priority" style="flex: 1">
            <NSelect v-model:value="priority" :options="priorityOptions" />
          </NFormItem>
        </NSpace>

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
