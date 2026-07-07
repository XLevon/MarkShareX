<template>
  <div class="news-manage">
    <div class="page-header">
      <h2 class="font-bold mb-6" style="color: var(--input-color); font-size: 28px">📢 资讯管理</h2>
      <n-button type="primary" @click="openCreate">+ 新建资讯</n-button>
    </div>

    <n-card>
      <n-data-table
        :columns="columns"
        :data="items"
        :loading="loading"
        :pagination="{ page: page, pageSize: pageSize, itemCount: total, onChange: onPageChange, onUpdatePageSize: onPageSizeChange }"
      />
    </n-card>

    <!-- Create/Edit Modal -->
    <n-modal v-model:show="showModal" :mask-closable="false" title="资讯管理">
      <n-card style="width: 720px; max-width: 90vw" :title="editingId ? '编辑资讯' : '新建资讯'">
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import { NButton, NSwitch, NSpace, useMessage } from 'naive-ui'
import { fetchAdminNews, createNews, updateNews, deleteNews, type NewsItem } from '@/api/news'

const message = useMessage()

const loading = ref(false)
const saving = ref(false)
const items = ref<NewsItem[]>([])
const page = ref(1)
const pageSize = ref(10)
const total = ref(0)

const showModal = ref(false)
const editingId = ref<number | null>(null)
const form = ref({
  title: '',
  summary: '',
  content: '',
  status: 'draft',
  sort_order: 0,
})

const statusOptions = [
  { label: '草稿', value: 'draft' },
  { label: '已发布', value: 'published' },
]

const columns = [
  { title: 'ID', key: 'id', width: 60 },
  { title: '标题', key: 'title', ellipsis: { tooltip: true } },
  { title: '发布', key: 'status', width: 70, render(row: NewsItem) {
    const published = row.status === 'published'
    return h(NSwitch, { size: 'small', value: published, onUpdateValue: (v: boolean) => toggleNewsStatus(row, v) })
  }},
  { title: '排序', key: 'sort_order', width: 80 },
  { title: '创建时间', key: 'created_at', width: 180, render(row: NewsItem) {
    return row.created_at ? new Date(row.created_at).toLocaleString('zh-CN') : '-'
  }},
  { title: '操作', key: 'actions', width: 160, render(row: NewsItem) {
    return h(NSpace, { size: 'small' }, {
      default: () => [
        h(NButton, { size: 'small', onClick: () => openEdit(row) }, { default: () => '编辑' }),
        h(NButton, { size: 'small', type: 'error', onClick: () => handleDelete(row) }, { default: () => '删除' }),
      ]
    })
  }},
]

async function loadData() {
  loading.value = true
  try {
    const resp = await fetchAdminNews({ page: page.value, page_size: pageSize.value })
    items.value = resp.data.data || []
    total.value = resp.data.pagination?.total || 0
  } catch {
    message.error('加载失败')
  } finally {
    loading.value = false
  }
}

function onPageChange(p: number) { page.value = p; loadData() }
function onPageSizeChange(s: number) { pageSize.value = s; page.value = 1; loadData() }

function openCreate() {
  editingId.value = null
  form.value = { title: '', summary: '', content: '', status: 'draft', sort_order: 0 }
  showModal.value = true
}

function openEdit(row: NewsItem) {
  editingId.value = row.id
  form.value = {
    title: row.title,
    summary: row.summary,
    content: row.content,
    status: row.status,
    sort_order: row.sort_order,
  }
  showModal.value = true
}

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

onMounted(() => loadData())
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
.page-header h2 {
  margin: 0;
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
}
</style>
