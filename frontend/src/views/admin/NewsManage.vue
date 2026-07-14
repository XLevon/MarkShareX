<template>
  <div class="news-manage">
    <div class="page-header">
      <h2 class="font-bold" style="color: var(--input-color); font-size: 28px">📢 资讯管理</h2>
      <n-button type="primary" @click="openCreate">+ 新建资讯</n-button>
    </div>

    <!-- Filter Bar -->
    <n-card size="small" style="margin-bottom:16px">
      <n-space align="center" :wrap="true">
        <span style="font-size:13px;color:var(--color-text-muted)">日期</span>
        <n-date-picker v-model:value="filterDateRange" type="daterange" clearable size="small" style="width:220px"
          @update:value="applyFilter">
          <template #footer>
            <div style="display:flex;gap:4px;flex-wrap:wrap;padding:8px 12px;border-top:1px solid var(--color-border)">
              <n-button size="tiny" quaternary @click="setDateRange('today')">今天</n-button>
              <n-button size="tiny" quaternary @click="setDateRange('yesterday')">昨天</n-button>
              <n-button size="tiny" quaternary @click="setDateRange('week')">本周</n-button>
              <n-button size="tiny" quaternary @click="setDateRange('lastWeek')">上周</n-button>
              <n-button size="tiny" quaternary @click="setDateRange('month')">本月</n-button>
              <n-button size="tiny" quaternary @click="setDateRange('lastMonth')">上月</n-button>
            </div>
          </template>
        </n-date-picker>
        <span style="font-size:13px;color:var(--color-text-muted)">状态</span>
        <n-select v-model:value="filterStatus" :options="filterStatusOptions" clearable size="small" style="width:100px" placeholder="全部"
          @update:value="applyFilter" />
        <span style="font-size:13px;color:var(--color-text-muted)">题材</span>
        <n-select v-model:value="filterTopicTypes" :options="filterTopicOptions" multiple clearable size="small" style="width:180px" placeholder="全部"
          @update:value="applyFilter" />
        <n-input v-model:value="filterSearch" placeholder="搜索资讯..." clearable size="small" style="width:160px"
          @keydown.enter="applyFilter" @clear="applyFilter" />
        <n-button v-if="checkedIds.length" size="small" type="success" @click="batchPublish">发布 {{ checkedIds.length }}</n-button>
        <n-button v-if="checkedIds.length" size="small" type="warning" @click="batchUnpublish">撤回 {{ checkedIds.length }}</n-button>
        <n-button v-if="isAdmin && checkedIds.length" size="small" type="error" @click="batchDelete">删除 {{ checkedIds.length }}</n-button>
      </n-space>
    </n-card>

    <n-card>
      <n-data-table
        :columns="columns"
        :data="items"
        :loading="loading"
        :row-key="rowKey"
        :checked-row-keys="checkedIds"
        @update:checked-row-keys="onCheckedChange"
        :pagination="false"
      />
      <div style="display:flex;justify-content:flex-end;margin-top:12px">
        <n-pagination
          v-model:page="pagination.page"
          :page-size="pagination.pageSize"
          :item-count="pagination.itemCount"
          :page-sizes="[10,20,50]"
          show-size-picker
          @update:page="loadData"
          @update:page-size="onPageSizeChange"
        />
      </div>
    </n-card>

    <!-- Create/Edit Modal -->
    <n-modal v-model:show="showModal" :mask-closable="false" title="资讯管理">
      <n-card style="width: 720px; max-width: 90vw" :title="editingId ? '编辑资讯' : '新建资讯'">
        <template #header-extra>
          <n-space :size="2">
            <n-button size="tiny" quaternary @click="navPrevEdit" :disabled="editingIndex <= 0">&lt;</n-button>
            <n-button size="tiny" quaternary @click="navNextEdit" :disabled="editingIndex < 0 || editingIndex >= items.length - 1">&gt;</n-button>
            <n-button size="tiny" quaternary @click="showModal = false">✕</n-button>
          </n-space>
        </template>
        <n-form label-placement="left" label-width="80">
          <n-form-item label="标题" required>
            <n-input v-model:value="form.title" placeholder="输入资讯标题" />
          </n-form-item>
          <n-form-item label="摘要">
            <n-input v-model:value="form.summary" type="textarea" :rows="2" placeholder="简短摘要，列表展示用" />
          </n-form-item>
          <n-form-item label="内容">
            <n-input v-model:value="form.content" type="textarea" :rows="8" placeholder="Markdown 格式正文" />
          </n-form-item>
          <n-form-item label="状态">
            <n-select v-model:value="form.status" :options="statusOptions" />
          </n-form-item>
          <n-form-item label="题材">
            <n-select v-model:value="form.topic_type" :options="topicTypeOptions" />
          </n-form-item>
          <n-form-item label="排序">
            <n-input-number v-model:value="form.sort_order" :min="0" />
          </n-form-item>
        </n-form>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showModal = false">取消</n-button>
            <n-button type="primary" :loading="saving" @click="handleSave">
              {{ editingId ? '保存' : '创建' }}
            </n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>

    <!-- Preview Modal -->
    <n-modal v-model:show="showPreview" :mask-closable="false">
      <n-card style="width:800px;max-width:95vw" :title="previewTitle">
        <template #header-extra>
          <n-space :size="2">
            <n-button size="tiny" quaternary @click="navPrevPreview" :disabled="previewIndex <= 0">&lt;</n-button>
            <n-button size="tiny" quaternary @click="navNextPreview" :disabled="previewIndex < 0 || previewIndex >= items.length - 1">&gt;</n-button>
            <n-button size="tiny" quaternary @click="showPreview = false">✕</n-button>
          </n-space>
        </template>
        <div v-if="previewHtml" class="markdown-body" v-html="previewHtml"></div>
        <div v-else style="color:var(--color-text-muted);text-align:center;padding:40px">暂无内容</div>
        <template #footer>
          <n-space justify="end">
            <n-button @click="showPreview = false">关闭</n-button>
          </n-space>
        </template>
      </n-card>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue'
import { NButton, NSwitch, NSpace, NCheckbox, useMessage } from 'naive-ui'
import { fetchAdminNews, fetchAdminNewsItem, createNews, updateNews, deleteNews, type NewsItem } from '@/api/news'
import api from '@/api'

import { useAuthStore } from '@/stores/auth'

const message = useMessage()
const authStore = useAuthStore()
const isAdmin = computed(() => authStore.user?.role === 'admin')

const loading = ref(false)
const saving = ref(false)
const items = ref<NewsItem[]>([])
const pagination = ref({
  page: 1,
  pageSize: 10,
  itemCount: 0,
})

const checkedIds = ref<number[]>([])

// Filters — persisted in localStorage
const FILTER_KEY = 'marksharex_news_filters'

function loadFilters() {
  try {
    const saved = localStorage.getItem(FILTER_KEY)
    if (saved) {
      const f = JSON.parse(saved)
      if (f.search) filterSearch.value = f.search
      if (f.dateRange) filterDateRange.value = f.dateRange
      if (f.status) filterStatus.value = f.status
      if (f.topicTypes) filterTopicTypes.value = f.topicTypes
    }
  } catch {}
}

function saveFilters() {
  localStorage.setItem(FILTER_KEY, JSON.stringify({
    search: filterSearch.value,
    dateRange: filterDateRange.value,
    status: filterStatus.value,
    topicTypes: filterTopicTypes.value,
  }))
}

function setDateRange(preset: 'today' | 'yesterday' | 'week' | 'lastWeek' | 'month' | 'lastMonth') {
  const now = new Date()
  let start: Date, end: Date
  switch (preset) {
    case 'today':
      start = new Date(now.getFullYear(), now.getMonth(), now.getDate())
      end = new Date(now.getFullYear(), now.getMonth(), now.getDate(), 23, 59, 59, 999)
      break
    case 'yesterday': {
      const d = new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1)
      start = d
      end = new Date(d.getFullYear(), d.getMonth(), d.getDate(), 23, 59, 59, 999)
      break
    }
    case 'week': {
      const day = now.getDay() || 7 // Sunday=0→7, Monday=1
      start = new Date(now.getFullYear(), now.getMonth(), now.getDate() - day + 1)
      end = new Date(now.getFullYear(), now.getMonth(), now.getDate() - day + 7, 23, 59, 59, 999)
      break
    }
    case 'lastWeek': {
      const day = now.getDay() || 7
      end = new Date(now.getFullYear(), now.getMonth(), now.getDate() - day, 23, 59, 59, 999)
      start = new Date(end.getFullYear(), end.getMonth(), end.getDate() - 6)
      break
    }
    case 'month':
      start = new Date(now.getFullYear(), now.getMonth(), 1)
      end = new Date(now.getFullYear(), now.getMonth() + 1, 0, 23, 59, 59, 999)
      break
    case 'lastMonth':
      start = new Date(now.getFullYear(), now.getMonth() - 1, 1)
      end = new Date(now.getFullYear(), now.getMonth(), 0, 23, 59, 59, 999)
      break
  }
  filterDateRange.value = [start.getTime(), end.getTime()]
}

const filterDateRange = ref<[number, number] | null>(null)
const filterStatus = ref<string | null>(null)
const filterTopicTypes = ref<string[]>([])
const filterSearch = ref('')

const filterStatusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已发布', value: 'published' },
]

const filterTopicOptions = [
  { label: '时政', value: 'politics' },
  { label: '财经', value: 'finance' },
  { label: '科技', value: 'technology' },
  { label: '社会', value: 'society' },
  { label: '文娱', value: 'entertainment' },
  { label: '体育', value: 'sports' },
  { label: '国际', value: 'international' },
  { label: '法治', value: 'law' },
  { label: '教育', value: 'education' },
]

const showModal = ref(false)
const editingId = ref<number | null>(null)
const editingIndex = ref(-1)
const showPreview = ref(false)
const previewIndex = ref(-1)
const previewTitle = ref('')
const previewHtml = ref('')
const form = ref({
  title: '',
  summary: '',
  content: '',
  status: 'draft',
  topic_type: '',
  sort_order: 0,
})

const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已发布', value: 'published' },
]

const topicTypeOptions = [
  { label: '不限', value: '' },
  { label: '时政新闻', value: 'politics' },
  { label: '财经新闻', value: 'finance' },
  { label: '科技新闻', value: 'technology' },
  { label: '社会新闻', value: 'society' },
  { label: '文娱新闻', value: 'entertainment' },
  { label: '体育新闻', value: 'sports' },
  { label: '国际新闻', value: 'international' },
  { label: '法治新闻', value: 'law' },
  { label: '教育新闻', value: 'education' },
]

function rowKey(row: NewsItem) { return row.id }

function onCheckedChange(keys: number[]) { checkedIds.value = keys }

const columns = computed(() => [
  { type: 'selection' as const, width: 40 },
  { title: 'ID', key: 'id', width: 60 },
  { title: '标题', key: 'title', ellipsis: { tooltip: true } },
  { title: '发布', key: 'status', width: 70, render(row: NewsItem) {
    const published = row.status === 'published'
    return h(NSwitch, { size: 'small', value: published, onUpdateValue: (v: boolean) => toggleNewsStatus(row, v) })
  }},
  { title: '题材', key: 'topic_type', width: 90, render(row: NewsItem) {
    const map: Record<string, string> = { politics: '时政', finance: '财经', technology: '科技', society: '社会', entertainment: '文娱', sports: '体育', international: '国际', law: '法治', education: '教育' }
    return row.topic_type ? map[row.topic_type] || row.topic_type : '-'
  }},
  { title: '排序', key: 'sort_order', width: 80 },
  { title: '创建时间', key: 'created_at', width: 180, render(row: NewsItem) {
    return row.created_at ? new Date(row.created_at).toLocaleString('zh-CN') : '-'
  }},
  { title: '操作', key: 'actions', width: 200, render(row: NewsItem) {
    return h(NSpace, { size: 'small' }, {
      default: () => [
        h(NButton, { size: 'small', onClick: () => openPreview(row) }, { default: () => '查看' }),
        h(NButton, { size: 'small', onClick: () => openEdit(row) }, { default: () => '编辑' }),
        h(NButton, { size: 'small', type: 'error', onClick: () => handleDelete(row) }, { default: () => '删除' }),
      ]
    })
  }},
])

function buildFilterParams() {
  const params: Record<string, any> = {
    page: pagination.value.page,
    page_size: pagination.value.pageSize,
    status: 'all',
  }
  if (filterSearch.value.trim()) params.search = filterSearch.value.trim()
  if (filterStatus.value) params.status = filterStatus.value
  if (filterTopicTypes.value.length) params.topic_type = filterTopicTypes.value.join(',')
  if (filterDateRange.value) {
    const [start, end] = filterDateRange.value
    // 使用本地时间格式化，避免 UTC 时区偏移（如 7月10日 CST → 7月9日 UTC）
    const sd = new Date(start)
    const ed = new Date(end)
    const pad = (n: number) => n.toString().padStart(2, '0')
    params.date_from = `${sd.getFullYear()}-${pad(sd.getMonth()+1)}-${pad(sd.getDate())}`
    params.date_to = `${ed.getFullYear()}-${pad(ed.getMonth()+1)}-${pad(ed.getDate())}`
  }
  return params
}

async function loadData() {
  loading.value = true
  try {
    const params = buildFilterParams()
    const resp = await fetchAdminNews(params)
    items.value = resp.data.data || []
    pagination.value.itemCount = resp.data.pagination?.total || 0
    checkedIds.value = []
  } catch {
    message.error('加载失败')
  } finally {
    loading.value = false
  }
}

function onPageSizeChange(s: number) { pagination.value.pageSize = s; pagination.value.page = 1; loadData() }

function applyFilter() { saveFilters(); pagination.value.page = 1; loadData() }

function openCreate() {
  editingId.value = null
  editingIndex.value = -1
  form.value = { title: '', summary: '', content: '', status: 'draft', topic_type: '', sort_order: 0 }
  showModal.value = true
}

async function openEdit(row: NewsItem) {
  editingId.value = row.id
  editingIndex.value = items.value.findIndex(item => item.id === row.id)
  form.value = {
    title: row.title,
    summary: row.summary,
    content: row.content || '',
    status: row.status,
    topic_type: row.topic_type,
    sort_order: row.sort_order,
  }
  showModal.value = true
  // 列表 API 不含正文，需要单独加载
  try {
    const { data } = await fetchAdminNewsItem(row.id)
    form.value.content = data.data.content || ''
  } catch {}
}
function navPrevEdit() { const idx = editingIndex.value; if (idx > 0) openEdit(items.value[idx - 1]) }
function navNextEdit() { const idx = editingIndex.value; if (idx >= 0 && idx < items.value.length - 1) openEdit(items.value[idx + 1]) }

async function openPreview(row: NewsItem) {
  previewTitle.value = row.title
  previewIndex.value = items.value.findIndex(item => item.id === row.id)
  previewHtml.value = ''
  showPreview.value = true
  try {
    const { data } = await fetchAdminNewsItem(row.id)
    previewHtml.value = data.data.content_html || ''
  } catch {
    previewHtml.value = '<p style="color:var(--color-text-muted);text-align:center;padding:40px">加载失败</p>'
  }
}
function navPrevPreview() { const idx = previewIndex.value; if (idx > 0) openPreview(items.value[idx - 1]) }
function navNextPreview() { const idx = previewIndex.value; if (idx >= 0 && idx < items.value.length - 1) openPreview(items.value[idx + 1]) }

async function handleSave() {
  if (!form.value.title.trim()) { message.warning('请输入标题'); return }
  saving.value = true
  try {
    if (editingId.value) {
      await updateNews(editingId.value, form.value)
      message.success('更新成功')
    } else {
      await createNews(form.value)
      message.success('创建成功')
    }
    showModal.value = false
    loadData()
  } catch {
    message.error('保存失败')
  } finally {
    saving.value = false
  }
}

async function handleDelete(row: NewsItem) {
  if (!confirm(`确定删除「${row.title}」？`)) return
  try {
    await deleteNews(row.id)
    message.success('删除成功')
    loadData()
  } catch {
    message.error('删除失败')
  }
}

async function toggleNewsStatus(row: NewsItem, v: boolean) {
  const newStatus = v ? 'published' : 'draft'
  try {
    await updateNews(row.id, { status: newStatus })
    row.status = newStatus
    message.success(v ? '已发布' : '已切换为草稿')
  } catch (e: any) {
    message.error(e?.response?.data?.error || '切换失败')
  }
}

async function batchPublish() {
  if (!checkedIds.value.length) return
  saving.value = true
  let ok = 0, fail = 0
  for (const id of checkedIds.value) {
    try { await updateNews(id, { status: 'published' }); ok++ } catch { fail++ }
  }
  if (fail) message.warning(`已发布 ${ok} 条，${fail} 条失败`)
  else message.success(`已发布 ${ok}/${checkedIds.value.length} 条`)
  saving.value = false
  loadData()
}

async function batchUnpublish() {
  if (!checkedIds.value.length) return
  saving.value = true
  let ok = 0, fail = 0
  for (const id of checkedIds.value) {
    try { await updateNews(id, { status: 'draft' }); ok++ } catch { fail++ }
  }
  if (fail) message.warning(`已取消发布 ${ok} 条，${fail} 条失败`)
  else message.success(`已取消发布 ${ok}/${checkedIds.value.length} 条`)
  saving.value = false
  loadData()
}

async function batchDelete() {
  if (!checkedIds.value.length) return
  if (!confirm(`确定删除选中的 ${checkedIds.value.length} 条资讯吗？此操作不可恢复。`)) return
  saving.value = true
  try {
    await api.post('/admin/news/batch-delete', { ids: checkedIds.value })
    message.success(`已删除 ${checkedIds.value.length} 条`)
    loadData()
  } catch (e: any) {
    message.error(e?.response?.data?.error || '批量删除失败')
  } finally {
    saving.value = false
  }
}

onMounted(() => { loadFilters(); loadData() })
</script>

<style scoped>
.news-manage {
  padding: 0 0 24px;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}
:page-header h2 {
  margin: 0;
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
}

@media (max-width: 640px) {
  .page-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }
  .page-header h2 {
    font-size: 22px;
  }
  /* 筛选栏控件纵向堆叠 */
  .news-manage :deep(.n-card .n-space) {
    flex-wrap: wrap;
  }
  .news-manage :deep(.n-card .n-space > *) {
    min-width: 100%;
  }
  /* 表格横向滚动 */
  .news-manage :deep(.n-data-table) {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }
}
</style>
