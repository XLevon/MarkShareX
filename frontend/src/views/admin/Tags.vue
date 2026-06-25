<template>
  <div>
    <!-- 新建标签弹窗 -->
    <n-modal v-model:show="showModal" preset="card" title="新建标签" style="max-width: 420px" :mask-closable="false">
      <n-form :model="form">
        <n-form-item label="标签名称">
          <n-input ref="tagInputRef" v-model:value="form.name" placeholder="请输入标签名称" @keyup.enter="handleCreate" />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showModal = false">取消</n-button>
          <n-button type="primary" :loading="creating" @click="handleCreate">创建</n-button>
        </n-space>
      </template>
    </n-modal>

    <n-card>
      <n-data-table :columns="columns" :data="tags" :loading="loading" :bordered="false" />
    </n-card>

    <n-modal v-model:show="showDeleteModal" preset="dialog" title="确认删除" positive-text="删除" negative-text="取消"
      @positive-click="confirmDelete">
      <p>确定要删除标签「{{ deletingItem?.name }}」吗？</p>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import { NButton, NTag, NSpace } from 'naive-ui'
import { useMessage } from 'naive-ui'
import { fetchTags, createTag, deleteTag } from '@/api/tags'
import type { Tag } from '@/api/index'

const message = useMessage()
const loading = ref(false)
const creating = ref(false)
const tags = ref<Tag[]>([])
const showModal = ref(false)
const form = ref({ name: '' })
const tagInputRef = ref<any>(null)
function openCreateModal() { form.value.name = ''; showModal.value = true; setTimeout(() => tagInputRef.value?.focus(), 100) }
const showDeleteModal = ref(false)
const deletingItem = ref<Tag | null>(null)

// Ownership helpers
const currentUserId = ref<number | null>(null)
const isPrivileged = ref(false)
function loadCurrentUser() {
  try {
    const stored = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
    if (stored) {
      const u = JSON.parse(stored)
      currentUserId.value = u.id
      isPrivileged.value = ['admin', 'sub_admin'].includes(u.role)
    }
  } catch {}
}
loadCurrentUser()

function canEdit(tag: Tag) {
  return isPrivileged.value || tag.user_id === currentUserId.value
}

const columns = [
  { title: '名称', key: 'name', width: 200 },
  {
    title: '文章数',
    key: 'post_count',
    width: 70,
    render: (row: Tag) => h('span', { style: { color: (row.post_count ?? 0) > 0 ? '#818cf8' : '#9ca3af' } }, String(row.post_count ?? 0)),
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    render: (row: Tag) => {
      const editable = canEdit(row)
      return h(NSpace, { size: 2 }, () => [
        editable
          ? h(NButton, { type: 'error', size: 'tiny', onClick: () => { deletingItem.value = row; showDeleteModal.value = true } }, () => '删除')
          : h('span', { style: { color: '#4b5563', fontSize: '12px' } }, '只读'),
      ])
    },
  },
]

async function loadTags() {
  loading.value = true
  try {
    const { data: resp } = await fetchTags()
    tags.value = resp.data
  } finally {
    loading.value = false
  }
}

async function handleCreate() {
  if (!form.value.name.trim()) {
    message.warning('请输入标签名称')
    return
  }
  creating.value = true
  try {
    await createTag(form.value.name.trim())
    form.value.name = ''
    showModal.value = false
    message.success('标签已创建')
    loadTags()
  } catch (e: any) {
    message.error(e.response?.data?.error || '创建失败')
  } finally {
    creating.value = false
  }
}

async function confirmDelete() {
  if (!deletingItem.value) return
  try {
    await deleteTag(deletingItem.value.id)
    message.success('标签已删除')
    loadTags()
  } catch (e: any) {
    message.error(e.response?.data?.error || '删除失败')
  }
}

onMounted(loadTags)

defineExpose({ openCreateModal })
</script>
