<script setup lang="ts">
import { ref } from 'vue'
import {
  NButton,
  NCard,
  NDataTable,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NPopconfirm,
  NSelect,
  NSpace,
  NTag,
  useMessage,
  type DataTableColumns,
} from 'naive-ui'
import { useProjectStore } from '@/stores/project'
import type { Member, MemberRole } from '@/types'

const store = useProjectStore()
const message = useMessage()

const showAddModal = ref(false)
const showEditModal = ref(false)
const editingMember = ref<Member | null>(null)

// Add form state
const addName = ref('')
const addEmail = ref('')
const addRole = ref<MemberRole>('member')
const addSubmitting = ref(false)

// Edit form state
const editName = ref('')
const editEmail = ref('')
const editRole = ref<MemberRole>('member')
const editSubmitting = ref(false)

const roleOptions = [
  { label: 'Owner', value: 'owner' },
  { label: 'Member', value: 'member' },
  { label: 'Agent', value: 'agent' },
]

const ROLE_COLORS: Record<MemberRole, string> = {
  owner: '#f59e0b',
  member: '#3b82f6',
  agent: '#8b5cf6',
}

const columns: DataTableColumns<Member> = [
  {
    title: 'Name',
    key: 'name',
    render(row) {
      return row.name
    },
  },
  {
    title: 'Role',
    key: 'role',
    width: 120,
    render(row) {
      const color = ROLE_COLORS[row.role]
      return h(
        NTag,
        {
          size: 'small',
          round: true,
          bordered: false,
          color: { color: color + '22', textColor: color },
        },
        () => row.role,
      )
    },
  },
  {
    title: 'Email',
    key: 'email',
    render(row) {
      return row.email || '\u2014'
    },
  },
  {
    title: '',
    key: 'actions',
    width: 140,
    render(row) {
      return h(NSpace, { size: 4 }, () => [
        h(
          NButton,
          { size: 'tiny', quaternary: true, onClick: () => openEdit(row) },
          () => 'Edit',
        ),
        h(
          NPopconfirm,
          { onPositiveClick: () => handleRemove(row) },
          {
            trigger: () =>
              h(NButton, { size: 'tiny', quaternary: true, type: 'error' }, () => 'Remove'),
            default: () => `Remove ${row.name}?`,
          },
        ),
      ])
    },
  },
]

import { h } from 'vue'

function openAdd() {
  addName.value = ''
  addEmail.value = ''
  addRole.value = 'member'
  showAddModal.value = true
}

function openEdit(member: Member) {
  editingMember.value = member
  editName.value = member.name
  editEmail.value = member.email ?? ''
  editRole.value = member.role
  showEditModal.value = true
}

async function handleAdd() {
  if (!addName.value.trim()) return
  addSubmitting.value = true
  try {
    await store.addMember({
      name: addName.value.trim(),
      role: addRole.value,
      email: addEmail.value.trim() || undefined,
    })
    message.success('Member added')
    showAddModal.value = false
  } catch {
    message.error('Failed to add member')
  } finally {
    addSubmitting.value = false
  }
}

async function handleEdit() {
  if (!editingMember.value || !editName.value.trim()) return
  editSubmitting.value = true
  try {
    const updates: Record<string, string | null> = {}
    if (editName.value.trim() !== editingMember.value.name) {
      updates.name = editName.value.trim()
    }
    if (editRole.value !== editingMember.value.role) {
      updates.role = editRole.value
    }
    const newEmail = editEmail.value.trim() || null
    if (newEmail !== editingMember.value.email) {
      updates.email = newEmail
    }
    if (Object.keys(updates).length > 0) {
      await store.updateMember(editingMember.value.id, updates as any)
      message.success('Member updated')
    }
    showEditModal.value = false
  } catch {
    message.error('Failed to update member')
  } finally {
    editSubmitting.value = false
  }
}

async function handleRemove(member: Member) {
  try {
    await store.removeMember(member.id)
    message.success(`Removed ${member.name}`)
  } catch {
    message.error('Failed to remove member')
  }
}
</script>

<template>
  <div class="members-view">
    <div class="members-toolbar">
      <h2>Members</h2>
      <NButton type="primary" size="small" @click="openAdd">+ Add Member</NButton>
    </div>

    <div class="members-table-container">
      <NDataTable :columns="columns" :data="store.members" :bordered="false" size="small" />
    </div>

    <!-- Add Member Modal -->
    <NModal :show="showAddModal" @update:show="(v: boolean) => !v && (showAddModal = false)">
      <NCard
        title="Add Member"
        :bordered="true"
        closable
        @close="showAddModal = false"
        style="width: 440px; max-width: 95vw"
        role="dialog"
      >
        <NForm @submit.prevent="handleAdd" label-placement="top">
          <NFormItem label="Name">
            <NInput
              v-model:value="addName"
              placeholder="Member name"
              autofocus
              @keydown.enter.prevent="handleAdd"
            />
          </NFormItem>
          <NFormItem label="Email">
            <NInput v-model:value="addEmail" placeholder="email@example.com (optional)" />
          </NFormItem>
          <NFormItem label="Role">
            <NSelect v-model:value="addRole" :options="roleOptions" />
          </NFormItem>
          <NSpace justify="end" :size="8">
            <NButton @click="showAddModal = false">Cancel</NButton>
            <NButton
              type="primary"
              @click="handleAdd"
              :loading="addSubmitting"
              :disabled="!addName.trim()"
            >
              Add Member
            </NButton>
          </NSpace>
        </NForm>
      </NCard>
    </NModal>

    <!-- Edit Member Modal -->
    <NModal :show="showEditModal" @update:show="(v: boolean) => !v && (showEditModal = false)">
      <NCard
        title="Edit Member"
        :bordered="true"
        closable
        @close="showEditModal = false"
        style="width: 440px; max-width: 95vw"
        role="dialog"
      >
        <NForm @submit.prevent="handleEdit" label-placement="top">
          <NFormItem label="Name">
            <NInput
              v-model:value="editName"
              placeholder="Member name"
              autofocus
              @keydown.enter.prevent="handleEdit"
            />
          </NFormItem>
          <NFormItem label="Email">
            <NInput v-model:value="editEmail" placeholder="email@example.com (optional)" />
          </NFormItem>
          <NFormItem label="Role">
            <NSelect v-model:value="editRole" :options="roleOptions" />
          </NFormItem>
          <NSpace justify="end" :size="8">
            <NButton @click="showEditModal = false">Cancel</NButton>
            <NButton
              type="primary"
              @click="handleEdit"
              :loading="editSubmitting"
              :disabled="!editName.trim()"
            >
              Save Changes
            </NButton>
          </NSpace>
        </NForm>
      </NCard>
    </NModal>
  </div>
</template>

<style scoped>
.members-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.members-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 24px;
  flex-shrink: 0;
}

.members-toolbar h2 {
  font-size: 16px;
  font-weight: 600;
}

.members-table-container {
  flex: 1;
  padding: 0 24px 24px;
  overflow: auto;
}
</style>
