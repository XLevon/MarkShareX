<template>
  <div>
    <n-card>
      <n-data-table
        :columns="columns"
        :data="items"
        :bordered="false"
        size="small"
        :row-props="(getRowProps as any)"
        :row-class-name="getRowClass"
      />
    </n-card>

    <!-- 编辑/新增弹窗 -->
    <n-modal v-model:show="showModal" preset="card" :title="editItem ? '编辑' : '新增'" style="max-width:480px" :mask-closable="false">
      <n-form v-if="editForm" label-placement="left" label-width="80">
        <n-form-item label="编码">
          <n-input
            v-if="!editItem"
            v-model:value="editForm.code"
            placeholder="英文编码，创建后不可修改，如 tutorial"
          />
          <n-input v-else :value="editItem.code" disabled placeholder="编码创建后不可修改" />
        </n-form-item>
        <n-form-item label="显示名称">
          <n-input v-model:value="editForm.display_name" placeholder="如 🤖 AI 整理" />
        </n-form-item>
        <n-form-item label="颜色">
          <n-color-picker v-model:value="editForm.color" :modes="['hex']" />
        </n-form-item>
        <n-form-item label="排序">
          <n-input-number v-model:value="editForm.sort_order" :min="0" :max="99" />
        </n-form-item>
        <n-form-item label="启用">
          <n-switch v-model:value="editForm.is_active" />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showModal = false">取消</n-button>
          <n-button type="primary" @click="handleSave">保存</n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'KnowledgeBaseTab' })
import { ref, computed, onMounted, h } from 'vue'
import { useMessage, useDialog, NButton, NSpace } from 'naive-ui'
import {
  fetchAdminArticleTypes, createArticleType, updateArticleType, deleteArticleType, reorderArticleTypes,
  fetchAdminArticleStatuses, createArticleStatus, updateArticleStatus, deleteArticleStatus, reorderArticleStatuses,
} from '@/api/admin'

const props = defineProps<{ mode: 'types' | 'statuses' }>()
const message = useMessage()
const dialog = useDialog()

// ── Data ──
interface ItemWithCount { id: number; code: string; display_name: string; color: string; sort_order: number; is_active: boolean; post_count: number }
const items = ref<ItemWithCount[]>([])
const showModal = ref(false)
const editItem = ref<any>(null)
const editForm = ref<{ code: string; display_name: string; color: string; sort_order: number; is_active: boolean } | null>(null)
const saving = ref(false)

async function load() {
  try {
    if (props.mode === 'types') {
      const res = await fetchAdminArticleTypes()
      items.value = (res.data?.data ?? []) as ItemWithCount[]
    } else {
      const res = await fetchAdminArticleStatuses()
      items.value = (res.data?.data ?? []) as ItemWithCount[]
    }
  } catch { message.error('加载失败') }
}

// ── Drag & Drop ──
const dragIndex = ref<number | null>(null)
const overIndex = ref<number | null>(null)
const dropHalf = ref<'top' | 'bottom'>('bottom')
const reordering = ref(false)

function getRowProps(_row: any, index: number) {
  return {
    draggable: 'true',
    ondragstart: (e: DragEvent) => onDragStart(e, index),
    ondragover: (e: DragEvent) => onDragOver(e, index),
    ondragenter: (e: DragEvent) => onDragEnter(e, index),
    ondragleave: () => onDragLeave(),
    ondragend: () => onDragEnd(),
    ondrop: (e: DragEvent) => onDrop(e, index),
  }
}

function getRowClass(_row: any, index: number) {
  if (dragIndex.value === index) return 'drag-row'
  if (overIndex.value === index) {
    return dropHalf.value === 'top' ? 'drag-over-top' : 'drag-over-bottom'
  }
  return ''
}

function onDragStart(e: DragEvent, index: number) {
  dragIndex.value = index
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', String(index))
  }
}

function onDragOver(e: DragEvent, index: number) {
  e.preventDefault()
  if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  const target = e.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  const y = e.clientY - rect.top
  dropHalf.value = y < rect.height / 2 ? 'top' : 'bottom'
  overIndex.value = index
}

function onDragEnter(_e: DragEvent, index: number) {
  if (dragIndex.value !== index) overIndex.value = index
}

function onDragLeave() { overIndex.value = null }

async function onDrop(e: DragEvent, targetIndex: number) {
  e.preventDefault()
  if (dragIndex.value === null || dragIndex.value === targetIndex) {
    dragIndex.value = null; overIndex.value = null; return
  }

  let insertAt = dropHalf.value === 'top' ? targetIndex : targetIndex + 1
  if (dragIndex.value < insertAt) insertAt--

  const list = [...items.value]
  const [moved] = list.splice(dragIndex.value, 1)
  list.splice(insertAt, 0, moved)

  dragIndex.value = null
  overIndex.value = null
  reordering.value = true

  try {
    const ids = list.map(i => i.id)
    if (props.mode === 'types') await reorderArticleTypes(ids)
    else await reorderArticleStatuses(ids)
    message.success('排序已更新')
    await load()
  } catch {
    message.error('排序失败')
  } finally {
    reordering.value = false
  }
}

function onDragEnd() {
  dragIndex.value = null
  overIndex.value = null
}

// ── Create / Edit ──
function openCreate() {
  editItem.value = null
  editForm.value = { code: '', display_name: '', color: '#6b7280', sort_order: 99, is_active: true }
  showModal.value = true
}

function openEdit(item: ItemWithCount) {
  editItem.value = item
  editForm.value = { code: item.code, display_name: item.display_name, color: item.color, sort_order: item.sort_order, is_active: item.is_active }
  showModal.value = true
}

async function handleSave() {
  if (!editForm.value) return
  saving.value = true
  try {
    if (editItem.value) {
      if (props.mode === 'types') await updateArticleType(editItem.value.id, editForm.value)
      else await updateArticleStatus(editItem.value.id, editForm.value)
    } else {
      if (props.mode === 'types') await createArticleType(editForm.value)
      else await createArticleStatus(editForm.value)
    }
    showModal.value = false
    message.success('保存成功')
    load()
  } catch (e: any) {
    message.error(e?.response?.data?.error || '保存失败')
  } finally {
    saving.value = false
  }
}

// ── Delete ──
function handleDelete(item: ItemWithCount) {
  if (item.post_count > 0) {
    message.warning(`「${item.display_name}」下有 ${item.post_count} 篇文章，无法删除`)
    return
  }
  dialog.warning({
    title: '确认删除',
    content: `确定要删除「${item.display_name}」吗？`,
    positiveText: '删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      try {
        if (props.mode === 'types') await deleteArticleType(item.id)
        else await deleteArticleStatus(item.id)
        message.success('删除成功')
        load()
      } catch (e: any) {
        message.error(e?.response?.data?.error || '删除失败')
      }
    }
  })
}

// ── Columns ──
const columns = computed(() => [
  {
    title: '',
    key: 'drag',
    width: 36,
    render: () => h('div', {
      class: 'drag-handle',
      style: { cursor: 'grab', color: 'var(--color-text-muted)', fontSize: '18px', userSelect: 'none', textAlign: 'center' },
    }, '⠿'),
  },
  { title: '编码', key: 'code', width: 160 },
  { title: '显示名称', key: 'display_name', width: 240 },
  { title: '文章数', key: 'post_count', width: 70, render(row: ItemWithCount) { return h('span', { style: { color: row.post_count > 0 ? '#818cf8' : '#9ca3af' } }, String(row.post_count)) } },
  { title: '颜色', key: 'color', width: 60, render(row: ItemWithCount) { return h('span', { style: { display:'inline-block', width:'20px', height:'20px', borderRadius:'4px', backgroundColor: row.color, border:'1px solid rgba(0,0,0,.15)' } }) } },
  { title: '排序', key: 'sort_order', width: 55 },
  { title: '状态', key: 'is_active', width: 55, render(row: ItemWithCount) { return h('span', { style: { color: row.is_active ? '#22c55e' : '#9ca3af' } }, row.is_active ? '启用' : '禁用') } },
  {
    title: '操作', key: 'action', width: 140, render(row: ItemWithCount) {
      return h(NSpace, { size: 4 }, () => [
        h(NButton, { size: 'tiny', onClick: () => openEdit(row) }, () => '编辑'),
        h(NButton, {
          size: 'tiny',
          type: 'error',
          disabled: row.post_count > 0,
          onClick: () => handleDelete(row),
        }, () => '删除'),
      ])
    }
  },
])

onMounted(load)

defineExpose({ openCreate })
</script>

<style scoped>
:deep(.drag-row) { opacity: 0.4; }
:deep(.drag-over-top td) { border-top: 2px solid var(--color-primary) !important; }
:deep(.drag-over-bottom td) { border-bottom: 2px solid var(--color-primary) !important; }
</style>
