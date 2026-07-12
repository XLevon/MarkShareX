<template>
  <div class="analytics-page">
    <h1 class="page-title">阅读统计</h1>

    <!-- Summary -->
    <div class="summary-row">
      <div class="summary-card clickable" @click="openReadLogs()">
        <span class="summary-value">{{ totalViews }}</span>
        <span class="summary-label">总阅读量</span>
      </div>
      <div class="summary-card">
        <span class="summary-value">{{ totalItems }}</span>
        <span class="summary-label">文章数</span>
      </div>
      <div class="summary-card">
        <span class="summary-value">{{ avgViews }}</span>
        <span class="summary-label">平均阅读</span>
      </div>
    </div>

    <!-- Posts table -->
    <n-data-table
      :columns="columns"
      :data="posts"
      :loading="loading"
      :pagination="pagination"
      :row-key="(p: PostView) => p.post_id"
      :bordered="false"
      size="small"
    />

    <!-- Read Logs Modal -->
    <n-modal v-model:show="showLogsModal" preset="card" :title="logsTitle" style="max-width: 800px">
      <n-data-table
        :columns="logsColumns"
        :data="readLogs"
        :loading="logsLoading"
        :bordered="false"
        size="small"
      />
      <template #footer>
        <n-space justify="space-between" align="center">
          <span class="text-dim text-sm">共 {{ logsTotal }} 条</span>
          <n-space>
            <n-button size="small" :disabled="logsPage <= 1" @click="loadLogsPage(logsPage - 1)">上一页</n-button>
            <span class="text-sm" style="color: var(--text-secondary)">{{ logsPage }} / {{ logsTotalPages }}</span>
            <n-button size="small" :disabled="logsPage >= logsTotalPages" @click="loadLogsPage(logsPage + 1)">下一页</n-button>
            <n-button @click="showLogsModal = false">关闭</n-button>
          </n-space>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, h } from 'vue'
import { useRouter } from 'vue-router'
import api from '@/api/index'
import { fetchReadLogs, type ReadLog } from '@/api/admin'
import type { DataTableColumn } from 'naive-ui'
import dayjs from 'dayjs'

interface PostView {
  post_id: number
  title: string
  slug: string
  author_name: string
  view_count: number
  like_count: number
  comment_count: number
  published_at: string | null
}

function fmtDate(d: string | null) {
  if (!d) return '-'
  return dayjs(d).format('YYYY-MM-DD HH:mm:ss')
}

const router = useRouter()
const loading = ref(false)
const posts = ref<PostView[]>([])
const totalItems = ref(0)

const pagination = ref({
  page: 1,
  pageSize: 20,
  itemCount: 0,
  showSizePicker: true,
  pageSizes: [10, 20, 50],
  onChange: (page: number) => { pagination.value.page = page; loadData() },
  onUpdatePageSize: (size: number) => { pagination.value.pageSize = size; pagination.value.page = 1; loadData() },
})

const totalViews = computed(() => posts.value.reduce((s, p) => s + (p.view_count || 0), 0))
const avgViews = computed(() => posts.value.length ? Math.round(totalViews.value / posts.value.length) : 0)

function goPost(id: number) {
  router.push(`/post/${id}`)
}

const columns: DataTableColumn<PostView>[] = [
  {
    title: '文章',
    key: 'title',
    ellipsis: { tooltip: true },
    render(row) {
      return h(
        'a',
        {
          class: 'no-underline font-medium',
          style: { color: 'var(--color-primary)', cursor: 'pointer' },
          onClick: (e: Event) => {
            e.preventDefault()
            router.push(`/post/${row.post_id}`)
          },
        },
        row.title,
      )
    },
  },
  {
    title: '作者',
    key: 'author_name',
    width: 140,
    ellipsis: { tooltip: true },
  },
  {
    title: '发布时间',
    key: 'published_at',
    width: 170,
    render(row) {
      return fmtDate(row.published_at)
    },
  },
  {
    title: '阅读量',
    key: 'view_count',
    width: 90,
    render(row) {
      return h(
        'a',
        {
          class: 'no-underline font-medium',
          style: { color: '#f59e0b', cursor: 'pointer' },
          onClick: (e: Event) => {
            e.preventDefault()
            openReadLogs(row.post_id)
          },
        },
        String(row.view_count || 0),
      )
    },
  },
  {
    title: '点赞量',
    key: 'like_count',
    width: 90,
  },
  {
    title: '评论量',
    key: 'comment_count',
    width: 90,
  },
]

async function loadData() {
  loading.value = true
  try {
    const { data: resp } = await api.get<{ data: PostView[]; pagination: any }>(
      `/analytics/post-views?page=${pagination.value.page}&page_size=${pagination.value.pageSize}`,
    )
    posts.value = resp.data || []
    if (resp.pagination) {
      pagination.value.itemCount = resp.pagination.total
      totalItems.value = resp.pagination.total
    }
  } catch {
    posts.value = []
  }
  loading.value = false
}

onMounted(loadData)

// ── Read Logs Modal ──
const showLogsModal = ref(false)
const logsLoading = ref(false)
const readLogs = ref<ReadLog[]>([])
const logsTitle = ref('阅读日志')
const logsPage = ref(1)
const logsTotal = ref(0)
const logsTotalPages = ref(0)
const logsFilterPostId = ref<number | undefined>(undefined)

const logsColumns = [
  { title: '文章', key: 'post_title', width: 200, ellipsis: true },
  { title: '读者', key: 'username', width: 100,
    render: (row: ReadLog) => row.username || h('span', { style: { color: '#6b7280' } }, '访客')
  },
  { title: 'IP', key: 'ip_address', width: 130 },
  { title: '设备', key: 'device_type', width: 80 },
  { title: '时长', key: 'duration_seconds', width: 70,
    render: (row: ReadLog) => {
      const s = row.duration_seconds || 0
      if (s >= 60) return `${Math.floor(s / 60)}分${s % 60}秒`
      return `${s}秒`
    }
  },
  { title: '时间', key: 'created_at', width: 150,
    render: (row: ReadLog) => dayjs(row.created_at).format('MM-DD HH:mm:ss')
  },
]

async function loadLogsPage(page: number) {
  logsLoading.value = true
  try {
    const params: any = { page, page_size: 20 }
    if (logsFilterPostId.value) params.post_id = logsFilterPostId.value
    const { data: resp } = await fetchReadLogs(params)
    readLogs.value = resp.data.data
    logsTotal.value = resp.data.pagination.total
    logsTotalPages.value = resp.data.pagination.pages
    logsPage.value = page
  } catch { /* ignore */ }
  logsLoading.value = false
}

async function openReadLogs(postId?: number) {
  logsTitle.value = postId
    ? `「${posts.value.find(p => p.post_id === postId)?.title || '文章'}」阅读日志`
    : '全部阅读日志'
  logsFilterPostId.value = postId
  showLogsModal.value = true
  await loadLogsPage(1)
}
</script>

<style scoped>
.analytics-page {
  animation: fadeIn 0.3s ease;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}
.page-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
  margin-bottom: 24px;
}

/* Summary */
.summary-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
  margin-bottom: 28px;
}
.summary-card {
  background: var(--card-bg);
  border: 1px solid rgba(255,255,255,0.05);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.summary-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
}
.summary-card.clickable {
  cursor: pointer;
  transition: border-color 0.15s;
}
.summary-card.clickable:hover {
  border-color: var(--color-primary-light);
}
.summary-label {
  font-size: 13px;
  color: var(--text-dim);
}

@media (max-width: 640px) {
  :deep(.n-data-table) {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }
}

</style>