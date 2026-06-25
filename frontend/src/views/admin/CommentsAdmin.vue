<template>
  <div>
    <div class="flex items-center justify-between mb-6">
      <h2 class="font-bold" style="color: var(--input-color); font-size: 28px">评论管理</h2>
    </div>

    <!-- Status filter tabs -->
    <div class="flex gap-2 mb-5">
      <button
        v-for="tab in visibleTabs"
        :key="tab.key"
        @click="filterStatus = tab.key; loadComments()"
        class="px-4 py-2 rounded-lg text-sm font-medium border-0 cursor-pointer transition-all"
        :style="filterStatus === tab.key ? { background: '#4f46e5', color: '#fff' } : { background: 'var(--modal-bg)', color: '#9ca3af' }"
      >
        {{ tab.label }}
        <span v-if="tab.key === 'pending' && pendingCount > 0" class="tab-badge">{{ pendingCount > 99 ? '99+' : pendingCount }}</span>
      </button>
    </div>

    <!-- Comment table -->
    <n-card>
      <n-data-table :columns="columns" :data="items" :loading="loading" :bordered="false" />
      <div v-if="totalPages > 1" class="flex justify-center mt-5 gap-2">
        <n-button
          v-for="p in totalPages"
          :key="p"
          size="small"
          :type="p === currentPage ? 'primary' : 'default'"
          @click="currentPage = p; loadComments()"
        >{{ p }}</n-button>
      </div>
    </n-card>

    <!-- Content popup -->
    <Teleport to="body">
      <div v-if="popupComment" class="fixed inset-0 z-50 flex items-center justify-center" style="background: rgba(0,0,0,0.6)" @click.self="popupComment = null">
        <div class="rounded-2xl p-6 w-full max-h-[80vh] overflow-y-auto" style="max-width: 640px; background: var(--modal-bg); border: 1px solid rgba(255,255,255,0.08)">
          <div class="flex items-center justify-between mb-4">
            <h3 class="m-0 text-base font-semibold" style="color: var(--input-color)">评论详情</h3>
            <button class="w-8 h-8 flex items-center justify-center rounded-lg border-0 cursor-pointer" style="background: transparent; color: var(--text-secondary); font-size: 20px" @click="popupComment = null">✕</button>
          </div>
          <div class="mb-3 flex items-center gap-3 text-xs" style="color: var(--text-secondary)">
            <span>#{{ popupComment.id }}</span>
            <span>{{ popupComment.author_name }}</span>
            <span>{{ popupComment.created_at }}</span>
            <span :style="{ color: statusColor(popupComment.status) }">[{{ statusText(popupComment.status) }}]</span>
          </div>
          <div class="markdown-body text-sm leading-relaxed" style="color: var(--text-primary); white-space: pre-wrap; word-break: break-word" v-text="popupComment.content"></div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue'
import { NButton, NSpace, NTag } from 'naive-ui'
import { useMessage } from 'naive-ui'
import { useRouter } from 'vue-router'
import { fetchAdminComments, updateCommentStatus } from '@/api/comments'
import { getCommentPendingCount } from '@/api/admin'
import type { Comment } from '@/api/index'

const message = useMessage()
const router = useRouter()
const loading = ref(false)
const items = ref<any[]>([])
const filterStatus = ref('')
const currentPage = ref(1)
const totalPages = ref(1)
const popupComment = ref<any>(null)
const pendingCount = ref(0)

const userRole = computed(() => {
  try {
    const raw = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
    if (raw) return JSON.parse(raw).role || 'visitor'
  } catch {}
  return 'visitor'
})
const isAdmin = computed(() => userRole.value === 'admin' || userRole.value === 'sub_admin')

const allTabs = [
  { key: '', label: '全部', roles: ['admin', 'sub_admin', 'author', 'visitor'] },
  { key: 'approved', label: '已通过', roles: ['admin', 'sub_admin', 'author', 'visitor'] },
  { key: 'pending', label: '待审核', roles: ['admin', 'sub_admin', 'author'] },
]

const visibleTabs = computed(() => allTabs.filter(t => t.roles.includes(userRole.value)))

function statusText(s: string) {
  const map: Record<string, string> = { pending: '待审', approved: '通过', deleted: '已删' }
  return map[s] || s
}
function statusColor(s: string) {
  const map: Record<string, string> = { pending: '#f59e0b', approved: '#34d399', deleted: '#ef4444' }
  return map[s] || '#9ca3af'
}

const columns = computed(() => [
  {
    title: '文章',
    key: 'post_title',
    width: 180,
    ellipsis: { tooltip: true },
    render: (row: any) =>
      h('a', {
        href: `/post/${row.post_id}`,
        style: { color: '#818cf8', cursor: 'pointer', textDecoration: 'none' },
        onClick: (e: Event) => { e.preventDefault(); router.push(`/post/${row.post_id}`) },
      }, row.post_title || `#${row.post_id}`),
  },
  {
    title: '内容',
    key: 'content',
    ellipsis: { tooltip: true },
    render: (row: any) =>
      h('div', {
        style: { maxWidth: '280px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', cursor: 'pointer', color: '#818cf8' },
        onClick: () => { popupComment.value = row },
      }, row.content.replace(/[#*`>_\[\]]/g, '').slice(0, 60)),
  },
  { title: '作者', key: 'author_name', width: 100 },
  {
    title: '状态',
    key: 'status',
    width: 80,
    render: (row: any) => {
      const map: Record<string, { text: string; color: string }> = {
        pending: { text: '待审', color: '#f59e0b' },
        approved: { text: '通过', color: '#34d399' },
        deleted: { text: '已删', color: '#ef4444' },
      }
      const s = map[row.status] || { text: row.status, color: '#9ca3af' }
      return h(NTag, { size: 'small', style: { borderColor: s.color, color: s.color } }, () => s.text)
    },
  },
  {
    title: '时间',
    key: 'created_at',
    width: 160,
    render: (row: any) => row.created_at || '',
  },
  {
    title: '操作',
    key: 'actions',
    width: 160,
    render: (row: any) =>
      h(NSpace, { size: 4 }, () => [
        h(NButton, { size: 'tiny', onClick: () => router.push(`/post/${row.post_id}#comment-${row.id}`) }, () => '查看'),
        !['visitor'].includes(userRole.value) && row.status !== 'approved' && h(NButton, { size: 'tiny', type: 'success', onClick: () => changeStatus(row, 'approved') }, () => '通过'),
        h(NButton, { size: 'tiny', type: 'error', onClick: () => changeStatus(row, 'deleted') }, () => '删除'),
      ].filter(Boolean)),
  },
])

async function changeStatus(row: any, status: string) {
  try {
    await updateCommentStatus(row.id, status)
    message.success(status === 'deleted' ? '已删除' : '状态已更新')
    loadComments()
    loadPendingCount()
  } catch {
    message.error('操作失败')
  }
}

async function loadComments() {
  loading.value = true
  try {
    const params: any = { page: currentPage.value, page_size: 20 }
    if (filterStatus.value) params.status = filterStatus.value
    const { data: resp } = await fetchAdminComments(params)
    items.value = resp.data || []
    totalPages.value = resp.pagination?.pages || 1
  } finally {
    loading.value = false
  }
}

async function loadPendingCount() {
  try {
    const role = userRole.value
    const params = (role !== 'admin' && role !== 'sub_admin') ? { scope: 'mine' } : undefined
    const { data: resp } = await getCommentPendingCount(params)
    pendingCount.value = resp.data || 0
  } catch { /* ignore */ }
}

onMounted(() => {
  loadComments()
  loadPendingCount()
})
</script>

<style scoped>
.tab-badge {
  font-size: 11px;
  padding: 1px 6px;
  border-radius: 10px;
  background: #ef4444;
  color: #fff;
  margin-left: 4px;
  font-weight: 600;
}
</style>
