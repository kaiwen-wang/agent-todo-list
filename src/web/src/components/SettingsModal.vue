<script setup lang="ts">
import { ref, watch } from 'vue'
import { NModal, NCard, NForm, NFormItem, NInput, NButton, NSpace, useMessage } from 'naive-ui'
import { useProjectStore } from '@/stores/project'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const store = useProjectStore()
const message = useMessage()

const name = ref('')
const prefix = ref('')
const description = ref('')
const submitting = ref(false)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen && store.project) {
      name.value = store.project.name
      prefix.value = store.project.prefix
      description.value = store.project.description ?? ''
    }
  },
)

async function submit() {
  if (!name.value.trim() || !prefix.value.trim()) return
  submitting.value = true
  try {
    const updates: Record<string, string> = {}
    if (name.value.trim() !== store.project?.name) {
      updates.name = name.value.trim()
    }
    if (prefix.value.trim().toUpperCase() !== store.project?.prefix) {
      updates.prefix = prefix.value.trim()
    }
    if (description.value.trim() !== (store.project?.description ?? '')) {
      updates.description = description.value.trim()
    }
    if (Object.keys(updates).length > 0) {
      await store.updateProjectSettings(updates)
      message.success('Settings saved')
    }
    emit('close')
  } catch {
    message.error('Failed to save settings')
  } finally {
    submitting.value = false
  }
}
</script>

<template>
  <NModal :show="open" @update:show="(v: boolean) => !v && emit('close')">
    <NCard
      title="Project Settings"
      :bordered="true"
      closable
      @close="emit('close')"
      style="width: 440px; max-width: 95vw"
      role="dialog"
    >
      <NForm @submit.prevent="submit" label-placement="top">
        <NFormItem label="Project Name">
          <NInput
            v-model:value="name"
            placeholder="My Project"
            autofocus
            @keydown.enter.prevent="submit"
          />
        </NFormItem>

        <NFormItem label="Prefix">
          <NInput
            v-model:value="prefix"
            placeholder="TODO"
            style="text-transform: uppercase"
            @keydown.enter.prevent="submit"
          />
          <template #feedback>
            Used for todo references like {{ (prefix || 'TODO').toUpperCase() }}-1,
            {{ (prefix || 'TODO').toUpperCase() }}-2, etc.
          </template>
        </NFormItem>

        <NFormItem label="Description">
          <NInput
            v-model:value="description"
            type="textarea"
            placeholder="Brief project description"
            :rows="3"
          />
        </NFormItem>

        <NSpace justify="end" :size="8" style="margin-top: 8px">
          <NButton @click="emit('close')">Cancel</NButton>
          <NButton
            type="primary"
            @click="submit"
            :loading="submitting"
            :disabled="!name.trim() || !prefix.trim()"
          >
            Save
          </NButton>
        </NSpace>
      </NForm>
    </NCard>
  </NModal>
</template>
