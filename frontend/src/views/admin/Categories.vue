<template>
  <div>
    <n-card>
      <n-data-table :columns="columns" :data="flatCategories" :loading="loading" :bordered="false" size="large"
        :row-props="getRowProps"
        :row-class-name="getRowClass"
      />
    </n-card>

    <!-- 新建/编辑分类弹窗 -->
    <n-modal v-model:show="showModal" preset="card" :title="editingId ? '编辑分类' : '新建分类'" style="max-width: 520px" :mask-closable="false" :trap-focus="!showImageSelector">
      <n-form :model="form">
        <n-form-item label="分类名称">
          <n-input v-model:value="form.name" placeholder="请输入分类名称" />
        </n-form-item>
        <n-form-item label="父分类">
          <n-select
            v-model:value="form.parent_id"
            :options="parentCategoryOptions"
            placeholder="无（一级分类）"
            clearable
          />
        </n-form-item>
        <n-form-item label="分类描述">
          <n-input v-model:value="form.description" type="textarea" placeholder="分类描述（可选）" :rows="3" />
        </n-form-item>
        <n-form-item label="封面图片">
          <div class="flex flex-col gap-3">
            <!-- 已选图片预览 -->
            <div v-if="categoryImageUrl" class="relative inline-flex">
              <img :src="categoryImageUrl" class="w-32 h-20 object-cover rounded-lg border border-white/10" />
              <button
                @click="form.image_url = ''"
                class="absolute -top-2 -right-2 w-6 h-6 rounded-full flex items-center justify-center text-white text-xs border-0 cursor-pointer"
                style="background: #ef4444"
                title="移除封面"
              >✕</button>
            </div>
            <!-- 设置图片按钮 -->
            <div>
              <n-button size="small" @click="showImageSelector = true">设置图片</n-button>
            </div>
          </div>
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="showModal = false">取消</n-button>
          <n-button type="primary" :loading="saving" @click="handleSave">保存</n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 删除确认 -->
    <n-modal v-model:show="showDeleteModal" preset="dialog" title="确认删除" positive-text="删除" negative-text="取消"
      @positive-click="confirmDelete">
      <p>确定要删除分类「{{ deletingItem?.name }}」吗？</p>
    </n-modal>

    <!-- 统一图片选择器 -->
    <ImageSelector
      :visible="showImageSelector"
      title="设置分类封面"
      :no-teleport="true"
      @close="showImageSelector = false"
      @select="onCategoryImageSelected"
    />

  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, h, computed } from 'vue'
import { useRouter } from 'vue-router'
import { NButton, NSpace, NTag, NSpin, NTooltip, NSwitch } from 'naive-ui'
import { useMessage } from 'naive-ui'
import { fetchAdminCategories, createCategory, updateCategory, deleteCategory, reorderCategories } from '@/api/categories'
import ImageSelector from '@/components/shared/ImageSelector.vue'
import type { Category } from '@/api/index'
import { fetchNetworkResources } from '@/api/admin'
import type { NetworkResource } from '@/api/admin'

const message = useMessage()
const router = useRouter()
const loading = ref(false)
const saving = ref(false)
const categories = ref<Category[]>([])

// 折叠状态：从 localStorage 恢复，首次访问默认折叠所有有子分类的
const COLLAPSED_KEY = 'marksharex_categories_collapsed'
const collapsedParents = ref<Set<number>>(new Set(
  JSON.parse(localStorage.getItem(COLLAPSED_KEY) || '[]')
))

function toggleCollapse(parentId: number) {
  const next = new Set(collapsedParents.value)
  if (next.has(parentId)) {
    next.delete(parentId)
  } else {
    next.add(parentId)
  }
  collapsedParents.value = next
  localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...next]))
}

async function toggleVisibility(row: Category, visible: boolean) {
  try {
    await updateCategory(row.id, { is_visible: visible })
    // flatCategories 是 computed（展开副本），必须改 categories 里的原始对象才能触发重算
    const original = categories.value.find(c => c.id === row.id)
    if (original) original.is_visible = visible
    message.success(visible ? '分类已显示' : '分类已隐藏')
  } catch (e: any) {
    message.error(e?.response?.data?.error || '操作失败')
  }
}

// ── 拖拽排序 ──
const dragIndex = ref<number | null>(null)
const overIndex = ref<number | null>(null)
const dropHalf = ref<'top' | 'bottom'>('bottom') // 上半/下半
const reordering = ref(false)

function getRowProps(row: any, index: number) {
  const editable = canEdit(row)
  return {
    style: { height: rowHeight + 'px', cursor: 'pointer' },
    draggable: editable ? 'true' : undefined,
    ondragstart: (e: DragEvent) => onDragStart(e, index),
    ondragover: (e: DragEvent) => onDragOver(e, index),
    ondragenter: (e: DragEvent) => onDragEnter(e, index),
    ondragleave: () => onDragLeave(),
    ondragend: () => onDragEnd(),
    ondrop: (e: DragEvent) => onDrop(e, index),
    onClick: () => viewCategoryArticles(row),
  }
}

function getRowClass(row: any, index: number) {
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
  // 判断鼠标在目标行的上半还是下半
  const target = e.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  const y = e.clientY - rect.top
  dropHalf.value = y < rect.height / 2 ? 'top' : 'bottom'
  overIndex.value = index
}

function onDragEnter(_e: DragEvent, index: number) {
  if (dragIndex.value !== index) {
    overIndex.value = index
  }
}

function onDragLeave() {
  overIndex.value = null
}

async function onDrop(e: DragEvent, targetIndex: number) {
  e.preventDefault()
  if (dragIndex.value === null || dragIndex.value === targetIndex) {
    dragIndex.value = null
    overIndex.value = null
    return
  }

  // 根据上半/下半决定插入位置
  let insertAt = dropHalf.value === 'top' ? targetIndex : targetIndex + 1
  // 如果拖拽源在插入位置之前，splice 移除后目标位置左移了
  if (dragIndex.value < insertAt) insertAt--

  const list = [...flatCategories.value]
  const [moved] = list.splice(dragIndex.value, 1)
  list.splice(insertAt, 0, moved)

  // 从 flatCategories 重建所有分类的 sort_order
  // flatCategories 包含所有一级 + 展开的子分类
  const sorted = [...categories.value]
  list.forEach((item, i) => {
    const cat = sorted.find(c => c.id === item.id)
    if (cat) {
      ;(cat as any)._newOrder = i
    }
  })
  sorted.sort((a, b) => ((a as any)._newOrder ?? a.sort_order) - ((b as any)._newOrder ?? b.sort_order))

  dragIndex.value = null
  overIndex.value = null
  reordering.value = true

  try {
    await reorderCategories(sorted.map(c => c.id))
    message.success('排序已更新')
    await loadCategories()
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

// 层级化排序的扁平列表（折叠的子分类不显示）
const flatCategories = computed(() => {
  const cats = categories.value || []
  const topLevel = cats.filter(c => !c.parent_id)
  const children = cats.filter(c => c.parent_id)
  const result: (Category & { _indent: number; _parentName: string; _hasChildren: boolean })[] = []

  topLevel.sort((a, b) => a.sort_order - b.sort_order).forEach(c => {
    const hasKids = children.some(ch => ch.parent_id === c.id)
    result.push({ ...c, _indent: 0, _parentName: '', _hasChildren: hasKids })
    // 折叠的不显示
    if (!collapsedParents.value.has(c.id)) {
      children.filter(ch => ch.parent_id === c.id)
        .sort((a, b) => a.sort_order - b.sort_order)
        .forEach(ch => {
          result.push({ ...ch, _indent: 1, _parentName: c.name, _hasChildren: false })
        })
    }
  })
  // 预防孤儿子分类（parent 在 DB 中不存在）— 不补充被折叠隐藏的子分类
  const topIds = new Set(topLevel.map(c => c.id))
  children.filter(ch => !topIds.has(ch.parent_id!)).forEach(ch => {
    result.push({ ...ch, _indent: 1, _parentName: ch.parent_id ? '?' : '', _hasChildren: false })
  })
  return result
})
const showModal = ref(false)
const showDeleteModal = ref(false)
const showImageSelector = ref(false)
const editingId = ref<number | null>(null)
const deletingItem = ref<Category | null>(null)

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

function canEdit(cat: Category) {
  return isPrivileged.value || cat.user_id === currentUserId.value
}

const form = ref({ name: '', description: '', image_url: '', parent_id: null as number | null })

// 网络资源缓存（用于 nr:{id} 预览）
const networkUrlCache = reactive(new Map<number, string>())

// 分类封面图片预览URL
const categoryImageUrl = computed(() => {
  const img = form.value.image_url
  if (!img) return ''
  if (img.startsWith('nr:')) {
    return networkUrlCache.get(Number(img.slice(3))) || ''
  }
  if (img.startsWith('http://') || img.startsWith('https://')) {
    return img
  }
  // 后端已返回相对路径 /uploads/xxx，直接使用
  if (img.startsWith('/uploads/') || img.startsWith('./uploads/')) {
    return img.replace('./uploads/', '/uploads/')
  }
  return `/uploads/${img}`
})

// 图片选择回调
function onCategoryImageSelected(value: string) {
  // 归一化：自托管绝对 URL → 相对路径 /uploads/xxx
  if (value.startsWith('http') && value.includes('/uploads/')) {
    form.value.image_url = value.replace(/https?:\/\/[^/]+\/uploads\//, '/uploads/')
  } else {
    form.value.image_url = value
  }
  // 如果是 nr:{id}，预热缓存
  if (value.startsWith('nr:')) {
    const id = Number(value.slice(3))
    if (!networkUrlCache.has(id)) {
      // 加载网络资源缓存
      fetchNetworkResources({ page_size: 200, source_type: 'image' }).then(({ data: resp }) => {
        (resp.data || []).forEach((nr: NetworkResource) => {
          networkUrlCache.set(nr.id, nr.url)
        })
      }).catch(() => {})
    }
  }
  showImageSelector.value = false
}

// 父分类可选项：仅一级分类 + 排除自身（编辑时）
const parentCategoryOptions = computed(() => {
  let items = categories.value.filter(c => !c.parent_id)
  if (editingId.value) {
    // 排除自身和已经是其子分类的
    const childIds = new Set(categories.value.filter(c => c.parent_id === editingId.value).map(c => c.id))
    items = items.filter(c => c.id !== editingId.value && !childIds.has(c.id))
  }
  return items.map(c => ({ label: c.name, value: c.id }))
})

const rowHeight = 96

const columns = [
  {
    title: '',
    key: 'drag',
    width: 36,
    render: (row: any) => {
      return h('div', {
        class: 'drag-handle',
        style: { cursor: 'grab', color: 'var(--color-text-muted)', fontSize: '18px', userSelect: 'none', textAlign: 'center' },
      }, '⠿')
    },
  },
  {
    title: '封面',
    key: 'image_url',
    width: 120,
    render: (row: any) => {
      const style: any = {
        width: '100px',
        height: (rowHeight - 16) + 'px',
        objectFit: 'cover',
        borderRadius: '6px',
        marginLeft: row._indent > 0 ? '20px' : '0',
      }
      return row.image_url
        ? h('img', { src: row.image_url, style })
        : h('div', { style: { ...style, background: 'var(--modal-bg)', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#374151', fontSize: '13px' } }, '—')
    },
  },
  { title: '名称', key: 'name', width: 160, render: (row: any) => {
    if (row._indent > 0) {
      return h('div', { style: { paddingLeft: '28px', fontSize: '13px' } }, [
        h('span', { style: { color: 'var(--color-text-muted)', fontSize: '11px' } }, '⤷ '),
        h('span', {}, row.name)
      ])
    }
    // 一级分类：有子项时显示折叠箭头
    const toggleIcon = row._hasChildren
      ? h('span', {
          style: { cursor: 'pointer', marginRight: '6px', fontSize: '15px', color: 'var(--color-text-muted)', userSelect: 'none' },
          ref: (el: any) => { if (el) el.onclick = (e: Event) => { e.stopPropagation(); toggleCollapse(row.id) } }
        }, collapsedParents.value.has(row.id) ? '▶' : '▼')
      : h('span', { style: { display: 'inline-block', width: '18px' } })
    return h('div', { style: { fontWeight: 500 } }, [toggleIcon, h('span', {}, row.name)])
  }},
  { title: '描述', key: 'description', ellipsis: undefined, render: (row: Category) => row.description
    ? h('div', { style: { whiteSpace: 'normal', wordBreak: 'break-word', lineHeight: '1.5', maxHeight: (rowHeight - 16) + 'px', overflow: 'hidden' } }, row.description)
    : h('span', { style: { color: '#4b5563' } }, '—')
  },
  {
    title: '文章数',
    key: 'post_count',
    width: 70,
    render: (row: Category) => h('span', { style: { color: (row.post_count ?? 0) > 0 ? '#818cf8' : '#9ca3af' } }, String(row.post_count ?? 0)),
  },
  {
    title: '可见',
    key: 'is_visible',
    width: 60,
    render: (row: Category) => {
      const editable = canEdit(row)
      return h(NSwitch, {
        size: 'small',
        value: row.is_visible,
        disabled: !editable,
        onUpdateValue: (val: boolean) => toggleVisibility(row, val),
      })
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 160,
    render: (row: any) => {
      const editable = canEdit(row)
      const hasPosts = (row.post_count || 0) > 0
      const hasChildren = categories.value.some(c => c.parent_id === row.id)
      const cannotDelete = hasPosts || hasChildren
      const deleteTip = hasPosts ? '该分类下有文章，无法删除' : hasChildren ? '该分类下有子分类，无法删除' : ''
      return h(NSpace, { size: 4 }, () => [
        editable
          ? h(NButton, { size: 'tiny', onClick: (e: MouseEvent) => { e.stopPropagation(); openEditModal(row) } }, () => '编辑')
          : h('span', { style: { color: '#4b5563', fontSize: '12px' } }, '只读'),
        editable && row.parent_id === null
          ? h(NButton, { size: 'tiny', onClick: (e: MouseEvent) => { e.stopPropagation(); openCreateChildModal(row) } }, () => '建子类')
          : null,
        editable
          ? h(
              NTooltip,
              cannotDelete ? { trigger: 'hover' } : { trigger: 'manual', show: false },
              {
                trigger: () =>
                  h(NButton, {
                    size: 'tiny',
                    type: 'error',
                    disabled: cannotDelete,
                    onClick: (e: MouseEvent) => { e.stopPropagation(); deletingItem.value = row; showDeleteModal.value = true },
                  }, () => '删除'),
                default: () => deleteTip,
              },
            )
          : null,
      ])
    },
  },
]

async function loadCategories() {
  loading.value = true
  try {
    const { data: resp } = await fetchAdminCategories()
    const wasEmpty = categories.value.length === 0
    categories.value = resp?.data ?? []
    // 首次访问且无缓存时：默认折叠所有有子分类的父级
    if (wasEmpty && !localStorage.getItem(COLLAPSED_KEY)) {
      const kids = (categories.value || []).filter(c => c.parent_id).map(c => c.parent_id!)
      collapsedParents.value = new Set([...new Set(kids)])
      localStorage.setItem(COLLAPSED_KEY, JSON.stringify([...collapsedParents.value]))
    }
  } finally {
    loading.value = false
  }
}

async function viewCategoryArticles(row: Category) {
  router.push({ path: '/admin/posts', query: { category_id: row.id } })
}

function openCreateModal() {
  editingId.value = null
  form.value = { name: '', description: '', image_url: '', parent_id: null }
  showModal.value = true
}

function openCreateChildModal(parent: Category) {
  editingId.value = null
  form.value = { name: '', description: '', image_url: '', parent_id: parent.id }
  showModal.value = true
}

function openEditModal(item: Category) {
  editingId.value = item.id
  // 统一用 image_url：API 返回的可能含完整 URL，需转换为相对路径或 nr: 前缀
  let img = item.image_url || ''
  form.value = {
    name: item.name,
    description: item.description || '',
    image_url: img,
    parent_id: (item as any).parent_id ?? null
  }
  showModal.value = true
}

async function handleSave() {
  if (!form.value.name) {
    message.warning('请输入分类名称')
    return
  }
  saving.value = true
  try {
    const categoryData = {
      name: form.value.name,
      description: form.value.description,
      parent_id: form.value.parent_id ?? null,
      image_url: form.value.image_url,
    }
    
    if (editingId.value) {
      await updateCategory(editingId.value, categoryData)
      message.success('分类已更新')
    } else {
      await createCategory(categoryData)
      message.success('分类已创建')
    }
    showModal.value = false
    loadCategories()
  } catch (e: any) {
    message.error(e.response?.data?.error || '操作失败')
  } finally {
    saving.value = false
  }
}

async function confirmDelete() {
  if (!deletingItem.value) return
  try {
    await deleteCategory(deletingItem.value.id)
    showDeleteModal.value = false
    deletingItem.value = null
    loadCategories()
  } catch { /* ignore */ }
}

onMounted(loadCategories)

defineExpose({ openCreateModal })
</script>

<style scoped>
/* 拖拽排序样式 */
:deep(.drag-row) {
  opacity: 0.4;
}
:deep(.drag-row td) {
  border-bottom: 2px dashed var(--color-primary) !important;
}
:deep(.drag-over-top td) {
  border-top: 2px solid var(--color-primary) !important;
}
:deep(.drag-over-bottom td) {
  border-bottom: 2px solid var(--color-primary) !important;
}
:deep(.drag-handle) {
  cursor: grab;
}

@media (max-width: 640px) {
  :deep(.n-data-table) {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }
}

</style>