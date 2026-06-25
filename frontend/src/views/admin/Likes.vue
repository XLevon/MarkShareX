<template>
  <div>
    <div class="mb-6">
      <h1 class="text-2xl font-bold" :style="{ color: 'var(--color-text)' }">点赞记录</h1>
      <p class="text-sm mt-1" :style="{ color: 'var(--color-text-muted)' }">
        总计 <strong :style="{ color: 'var(--color-primary)' }">{{ totalLikes }}</strong> 次点赞
      </p>
    </div>

    <!-- Table -->
    <n-data-table
      :columns="columns"
      :data="records"
      :loading="loading"
      :pagination="pagination"
      :row-key="(r: LikeRecord) => r.id"
      size="small"
      :bordered="false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, h, onMounted } from 'vue'
import { NDataTable } from 'naive-ui'
import { useRouter } from 'vue-router'
import api from '@/api/index'
import type { DataTableColumn } from 'naive-ui'
import dayjs from 'dayjs'

interface LikeRecord {
  id: number
  post_id: number
  post_title: string
  post_slug: string
  author_name: string
  published_at: string | null
  user_name: string
  created_at: string
}

function fmtDate(d: string | null) {
  if (!d) return '-'
  return dayjs(d).format('YYYY-MM-DD HH:mm:ss')
}

const router = useRouter()
const loading = ref(false)
const records = ref<LikeRecord[]>([])
const totalLikes = ref(0)
const pagination = ref({
  page: 1,
  pageSize: 20,
  itemCount: 0,
  showSizePicker: true,
  pageSizes: [10, 20, 50],
  onChange: (page: number) => { pagination.value.page = page; loadData() },
  onUpdatePageSize: (size: number) => { pagination.value.pageSize = size; pagination.value.page = 1; loadData() },
})

const columns: DataTableColumn<LikeRecord>[] = [
  {
    title: '文章',
    key: 'post_title',
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
        row.post_title,
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
    title: '点赞人',
    key: 'user_name',
    width: 140,
    ellipsis: { tooltip: true },
  },
  {
    title: '点赞时间',
    key: 'created_at',
    width: 170,
    render(row) {
      return fmtDate(row.created_at)
    },
  },
]

async function loadData() {
  loading.value = true
  try {
    const { data: resp } = await api.get<{ data: LikeRecord[]; pagination: any }>(
      `/admin/likes?page=${pagination.value.page}&page_size=${pagination.value.pageSize}`,
    )
    records.value = resp.data || []
    if (resp.pagination) {
      pagination.value.itemCount = resp.pagination.total
      totalLikes.value = resp.pagination.total
    }
  } catch {
    records.value = []
  }
  loading.value = false
}

onMounted(loadData)
</script>
