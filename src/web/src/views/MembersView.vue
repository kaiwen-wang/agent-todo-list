<script setup lang="ts">
import { ref, computed, h } from "vue";
import {
  NButton,
  NCard,
  NDataTable,
  NEmpty,
  NForm,
  NFormItem,
  NInput,
  NModal,
  NPopconfirm,
  NSpace,
  useMessage,
  type DataTableColumns,
} from "naive-ui";
import { useProjectStore } from "@/stores/project";
import { fetchGitIdentity } from "@/api";
import type { Member, Todo } from "@/types";
import { STATUS_DISPLAY, STATUS_COLORS } from "@/types";

const store = useProjectStore();
const message = useMessage();

const showAddModal = ref(false);
const showEditModal = ref(false);
const selectedMember = ref<Member | null>(null);
const editingMember = ref<Member | null>(null);

// Add form state (human members only)
const addName = ref("");
const addEmail = ref("");
const addSubmitting = ref(false);
const addSelfLoading = ref(false);

// Edit form state (human members only)
const editName = ref("");
const editEmail = ref("");
const editSubmitting = ref(false);

// ── Agents ──

const humanMembers = computed(() => store.members.filter((m) => m.role !== "agent"));

// ── Table ──

const columns: DataTableColumns<Member> = [
  {
    title: "Name",
    key: "name",
    render(row) {
      return row.name;
    },
  },
  {
    title: "Email",
    key: "email",
    render(row) {
      return row.email ? h("span", { style: "font-size: 12px; opacity: 0.6" }, row.email) : "";
    },
  },
  {
    title: "Tasks",
    key: "tasks",
    width: 80,
    render(row) {
      const count = store.todos.filter(
        (t) => t.assignee === row.id && t.status !== "archived" && t.status !== "wont_do",
      ).length;
      return count > 0 ? String(count) : "";
    },
  },
];

/** Todos assigned to the selected member */
const memberTodos = computed(() => {
  if (!selectedMember.value) return [];
  return store.todos.filter(
    (t) =>
      t.assignee === selectedMember.value!.id && t.status !== "archived" && t.status !== "wont_do",
  );
});

const memberTodoColumns: DataTableColumns<Todo> = [
  {
    title: "Ref",
    key: "ref",
    width: 70,
    render(row) {
      return h("span", { style: "font-family: monospace; font-size: 11px; opacity: 0.5" }, row.ref);
    },
  },
  {
    title: "Title",
    key: "title",
    ellipsis: { tooltip: true },
  },
  {
    title: "Status",
    key: "status",
    width: 110,
    render(row) {
      return h("span", { style: "display: flex; align-items: center; gap: 6px; font-size: 12px" }, [
        h("span", {
          style: `width: 8px; height: 8px; border-radius: 50%; background: ${STATUS_COLORS[row.status ?? "none"]}; flex-shrink: 0`,
        }),
        STATUS_DISPLAY[row.status ?? "none"],
      ]);
    },
  },
];

function selectMember(member: Member) {
  selectedMember.value = member;
}

const rowProps = (row: Member) => ({
  style: "cursor: pointer",
  onClick: () => selectMember(row),
});

function openAdd() {
  addName.value = "";
  addEmail.value = "";
  showAddModal.value = true;
}

function openEdit(member: Member) {
  editingMember.value = member;
  editName.value = member.name;
  editEmail.value = member.email ?? "";
  showEditModal.value = true;
}

function openTodo(row: Todo) {
  selectedMember.value = null;
  store.openTodo(row.number);
}

const todoRowProps = (row: Todo) => ({
  style: "cursor: pointer",
  onClick: () => openTodo(row),
});

async function handleAdd() {
  if (!addName.value.trim()) return;
  addSubmitting.value = true;
  try {
    await store.addMember({
      name: addName.value.trim(),
      email: addEmail.value.trim() || undefined,
    });
    message.success("Member added");
    showAddModal.value = false;
  } catch {
    message.error("Failed to add member");
  } finally {
    addSubmitting.value = false;
  }
}

async function handleEdit() {
  if (!editingMember.value || !editName.value.trim()) return;
  editSubmitting.value = true;
  try {
    const updates: Record<string, string | null> = {};
    if (editName.value.trim() !== editingMember.value.name) {
      updates.name = editName.value.trim();
    }
    const newEmail = editEmail.value.trim() || null;
    if (newEmail !== editingMember.value.email) {
      updates.email = newEmail;
    }
    if (Object.keys(updates).length > 0) {
      await store.updateMember(editingMember.value.id, updates);
      message.success("Member updated");
    }
    showEditModal.value = false;
  } catch {
    message.error("Failed to update member");
  } finally {
    editSubmitting.value = false;
  }
}

async function handleRemove(member: Member) {
  try {
    await store.removeMember(member.id);
    message.success(`Removed ${member.name}`);
    if (selectedMember.value?.id === member.id) {
      selectedMember.value = null;
    }
  } catch {
    message.error("Failed to remove member");
  }
}

async function handleAddSelf() {
  addSelfLoading.value = true;
  try {
    const identity = await fetchGitIdentity();
    if (!identity.name) {
      message.warning("git user.name is not configured");
      return;
    }
    const existing = store.members.find((m) => m.name === identity.name && m.role !== "agent");
    if (existing) {
      message.info(`"${identity.name}" is already a member`);
      return;
    }
    await store.addMember({
      name: identity.name,
      email: identity.email ?? undefined,
    });
    message.success(`Added ${identity.name}`);
  } catch (e) {
    console.error("Add self failed:", e);
    message.error(`Failed to add self: ${e instanceof Error ? e.message : e}`);
  } finally {
    addSelfLoading.value = false;
  }
}
</script>

<template>
  <div class="members-view">
    <!-- Members section -->
    <div class="members-toolbar">
      <h2>Members</h2>
      <div style="display: flex; gap: 8px">
        <NButton size="small" :loading="addSelfLoading" @click="handleAddSelf">Add Self</NButton>
        <NButton type="primary" size="small" @click="openAdd">+ Add Member</NButton>
      </div>
    </div>

    <div class="members-table-container">
      <NDataTable
        :columns="columns"
        :data="humanMembers"
        :row-props="rowProps"
        :bordered="false"
        size="small"
      />
    </div>

    <!-- Member Detail Modal -->
    <NModal :show="!!selectedMember" @update:show="(v: boolean) => !v && (selectedMember = null)">
      <NCard
        v-if="selectedMember"
        :bordered="true"
        closable
        @close="selectedMember = null"
        style="width: 640px; max-width: 95vw"
        role="dialog"
      >
        <template #header>
          <div class="member-detail-header">
            <span class="member-detail-avatar">{{
              selectedMember.name.charAt(0).toUpperCase()
            }}</span>
            <div>
              <div class="member-detail-name">{{ selectedMember.name }}</div>
              <div v-if="selectedMember.email" class="member-detail-meta">
                <span class="member-detail-email">{{ selectedMember.email }}</span>
              </div>
            </div>
          </div>
        </template>

        <div class="member-detail-actions">
          <NButton size="small" quaternary @click="openEdit(selectedMember)">Edit</NButton>
          <NPopconfirm @positive-click="handleRemove(selectedMember)">
            <template #trigger>
              <NButton size="small" quaternary type="error">Remove</NButton>
            </template>
            Remove {{ selectedMember.name }}?
          </NPopconfirm>
        </div>

        <div class="member-detail-section">
          <div class="section-title">Assigned Tasks ({{ memberTodos.length }})</div>
          <NDataTable
            v-if="memberTodos.length"
            :columns="memberTodoColumns"
            :data="memberTodos"
            :row-props="todoRowProps"
            :bordered="false"
            size="small"
          />
          <NEmpty v-else description="No tasks assigned" size="small" style="padding: 24px 0" />
        </div>
      </NCard>
    </NModal>

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

/* ── Agent section ── */

.agent-section {
  padding: 16px 24px 0;
  flex-shrink: 0;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}

.section-header h3 {
  font-size: 14px;
  font-weight: 600;
  opacity: 0.7;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.agent-picker {
  display: flex;
  gap: 10px;
}

.agent-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 8px;
  border: 1px solid #e8e8e8;
  background: #fafafa;
  cursor: pointer;
  font-family: inherit;
  font-size: inherit;
  color: inherit;
  text-align: left;
  flex: 1;
}

.agent-card:hover:not(:disabled) {
  border-color: #8b5cf6;
  background: #8b5cf608;
}

.agent-card:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.agent-card-enabled {
  border-color: #8b5cf6;
  background: #8b5cf608;
}

.agent-toggle {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 10px;
  background: #e8e8e8;
  color: #999;
  flex-shrink: 0;
}

.agent-toggle.on {
  background: #8b5cf622;
  color: #8b5cf6;
}

.agent-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: #8b5cf622;
  color: #8b5cf6;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.3px;
  flex-shrink: 0;
}

.agent-info {
  flex: 1;
  min-width: 0;
}

.agent-name {
  font-size: 14px;
  font-weight: 600;
}

.agent-desc {
  font-size: 12px;
  opacity: 0.5;
  margin-top: 1px;
}

/* ── Members section ── */

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

/* ── Member detail modal ── */

.member-detail-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.member-detail-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: #e8e8e8;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  font-weight: 700;
  color: #666;
  flex-shrink: 0;
}

.member-detail-name {
  font-size: 16px;
  font-weight: 700;
}

.member-detail-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 2px;
}

.member-detail-email {
  font-size: 12px;
  opacity: 0.5;
}

.member-detail-actions {
  display: flex;
  gap: 4px;
  margin-bottom: 16px;
}

.member-detail-section {
  margin-top: 4px;
}

.section-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  opacity: 0.45;
  margin-bottom: 8px;
}
</style>
