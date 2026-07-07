<template>
  <div class="dashboard">
    <!-- 页面标题 -->
    <div class="page-header">
      <h1 class="page-title">📊 控制台</h1>
    </div>

    <!-- 统计卡片 5 列 -->
    <div class="stats-row">
      <router-link to="/admin/posts?status=published&mtab=posts" class="stat-card stat-link">
        <div class="stat-icon published">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
          <span v-if="stats.dailyPublished > 0" class="stat-icon-badge orange">{{ stats.dailyPublished > 99 ? '99+' : stats.dailyPublished }}</span>
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ stats.published }}</span>
          <span class="stat-label">已发布数</span>
        </div>
      </router-link>
      <router-link to="/admin/posts?status=draft&mtab=posts" class="stat-card stat-link">
        <div class="stat-icon drafts">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
          <span v-if="stats.dailyDrafts > 0" class="stat-icon-badge orange">{{ stats.dailyDrafts > 99 ? '99+' : stats.dailyDrafts }}</span>
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ stats.drafts }}</span>
          <span class="stat-label">草稿数</span>
        </div>
      </router-link>
      <router-link to="/admin/analytics/views" class="stat-card stat-link">
        <div class="stat-icon views">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
          <span v-if="stats.dailyViews > 0" class="stat-icon-badge orange">{{ stats.dailyViews > 99 ? '99+' : stats.dailyViews }}</span>
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ stats.views }}</span>
          <span class="stat-label">阅读量</span>
        </div>
      </router-link>
      <router-link to="/admin/likes" class="stat-card stat-link">
        <div class="stat-icon likes">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
          <span v-if="stats.dailyLikes > 0" class="stat-icon-badge orange">{{ stats.dailyLikes > 99 ? '99+' : stats.dailyLikes }}</span>
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ stats.likes }}</span>
          <span class="stat-label">点赞量</span>
        </div>
      </router-link>
      <router-link to="/admin/analytics/comments" class="stat-card stat-link">
        <div class="stat-icon comments">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
          <span v-if="stats.pendingComments > 0" class="stat-icon-badge">{{ stats.pendingComments > 99 ? '99+' : stats.pendingComments }}</span>
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ stats.comments || 0 }}</span>
          <span class="stat-label">评论数</span>
        </div>
      </router-link>
      <router-link to="/admin/guestbook" class="stat-card stat-link">
        <div class="stat-icon guestbook">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/></svg>
        </div>
        <div class="stat-body">
          <span class="stat-value">{{ stats.guestbookEntries }}</span>
          <span class="stat-label">留言量</span>
        </div>
      </router-link>
    </div>

    <!-- 快捷入口 5 列 -->
    <div class="section-title">快捷入口</div>
    <div class="quick-actions">
      <router-link to="/admin/posts/new" class="quick-card">
        <div class="quick-icon write">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>
        </div>
        <span class="quick-label">写文章</span>
        <span class="quick-desc">新建一篇 Markdown 文章</span>
      </router-link>
      <router-link to="/admin/posts" class="quick-card">
        <div class="quick-icon manage">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>
        </div>
        <span class="quick-label">管理文章</span>
        <span class="quick-desc">查看、编辑、删除文章</span>
      </router-link>
      <router-link to="/admin/posts?mtab=categories" class="quick-card">
        <div class="quick-icon category">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>
        </div>
        <span class="quick-label">分类管理</span>
        <span class="quick-desc">管理文章分类结构</span>
      </router-link>
      <router-link to="/admin/posts?mtab=tags" class="quick-card">
        <div class="quick-icon tags-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
        </div>
        <span class="quick-label">标签管理</span>
        <span class="quick-desc">管理文章标签系统</span>
      </router-link>
      <router-link v-if="isAdmin" to="/admin/settings" class="quick-card">
        <div class="quick-icon settings">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
        </div>
        <span class="quick-label">基础设置</span>
        <span class="quick-desc">站点标题、描述等配置</span>
      </router-link>
      <router-link to="/admin/news" class="quick-card">
        <div class="quick-icon news-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16v16H4z"/><path d="M4 4l8 8 8-8"/></svg>
        </div>
        <span class="quick-label">资讯管理</span>
        <span class="quick-desc">管理资讯文章采集发布</span>
      </router-link>
    </div>

    <!-- 存储使用量 -->
    <div class="section-title">存储使用量</div>
    <div class="storage-card">
      <div class="storage-header">
        <span class="storage-label">已用 {{ formatSize(storageUsed) }} / 总计 {{ formatSize(storageTotal) }}</span>
        <span class="storage-pct">{{ storagePct }}%</span>
      </div>
      <div class="storage-bar">
        <div class="storage-fill" :style="{ width: storagePct + '%' }"></div>
      </div>
      <div class="storage-hint">拖拽或点击上传文件，支持图片、文档、压缩包等格式</div>
    </div>

    <!-- 两栏布局 -->
    <div class="bottom-grid">
      <!-- 最近文章 -->
      <div class="recent-posts">
        <div class="section-title">最近文章</div>
        <div class="post-list">
          <div v-for="post in recentPosts" :key="post.id" class="post-item" @click="router.push(`/admin/posts/${post.id}`)">
            <img
              v-if="post.cover_image || post.category_cover_image"
              :src="post.cover_image || post.category_cover_image"
              class="post-thumb"
              referrerpolicy="no-referrer"
              @error="$event.target.style.display='none'"
            />
            <div class="post-main">
              <span class="post-title">{{ post.title || '无标题' }}</span>
              <div class="post-meta">
                <span v-if="post.author_name" class="post-author">{{ post.author_name }}</span>
                <span class="post-date">{{ formatDate(post.created_at) }}</span>
                <span class="post-status" :class="post.status">{{ post.status === 'published' ? '已发布' : '草稿' }}</span>
              </div>
            </div>
            <svg class="post-arrow" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="9 18 15 12 9 6"/></svg>
          </div>
          <div v-if="recentPosts.length === 0 && !loading" class="empty-hint">暂无文章</div>
        </div>
        <router-link to="/admin/posts" class="view-all">查看全部 →</router-link>
      </div>

      <!-- 阅读趋势图 -->
      <div class="trend-chart">
        <div class="section-title">
          阅读趋势
          <span class="chart-range">
            <button :class="{ active: chartRange === 7 }" @click="chartRange = 7">7天</button>
            <button :class="{ active: chartRange === 30 }" @click="chartRange = 30">30天</button>
          </span>
        </div>
        <div class="chart-summary">
          <span>近{{ chartRange }}天阅读量 {{ totalViews }}</span>
          <span class="chart-avg">日均 {{ avgViews }}</span>
        </div>
        <div class="chart-container">
          <svg viewBox="0 0 400 200" class="chart-svg" preserveAspectRatio="none">
            <defs>
              <linearGradient id="trendGrad" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#4f46e5" stop-opacity="0.3"/>
                <stop offset="100%" stop-color="#4f46e5" stop-opacity="0.02"/>
              </linearGradient>
            </defs>
            <line v-for="i in 4" :key="'grid-'+i" :x1="0" :y1="i*50" :x2="400" :y2="i*50" stroke="rgba(255,255,255,0.06)" stroke-width="1"/>
            <!-- X-axis at bottom -->
            <line x1="20" :y1="xAxisY" :x2="380" :y2="xAxisY" stroke="rgba(255,255,255,0.15)" stroke-width="1"/>
            <path :d="areaPath" fill="url(#trendGrad)"/>
            <path :d="linePath" fill="none" stroke="#4f46e5" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
            <circle v-for="(pt, i) in points" :key="'pt-'+i" :cx="pt.x" :cy="pt.y" r="3" fill="var(--chart-bg)" stroke="#4f46e5" stroke-width="2"/>
            <!-- X-axis labels -->
            <text
              v-for="(l, i) in labelPoints"
              :key="'lbl-'+i"
              :x="l.x"
              :y="l.y"
              text-anchor="middle"
              font-size="9"
              fill="#6b7280"
            >{{ l.label }}</text>
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { fetchAdminPosts } from '@/api/posts'
import { fetchFiles } from '@/api/files'
import { getCommentPendingCount } from '@/api/admin'
import { fetchGuestbook } from '@/api/guestbook'
import api from '@/api/index'
import dayjs from 'dayjs'

const router = useRouter()
const loading = ref(false)

const isAdmin = computed(() => {
  try {
    const stored = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
    if (stored) return JSON.parse(stored).role === 'admin'
  } catch {}
  return false
})

const stats = reactive({ posts: 0, published: 0, drafts: 0, views: 0, likes: 0, comments: 0, pendingComments: 0, guestbookEntries: 0, dailyViews: 0, dailyLikes: 0, dailyPublished: 0, dailyDrafts: 0 })
const recentPosts = ref<any[]>([])
const storageUsed = ref(0)
const storageTotal = ref(1024 * 1024 * 1024)
const chartRange = ref(7)

const trendData = ref<number[]>([])

async function loadTrendData(days: number) {
  try {
    const { data: resp } = await api.get<{ data: { date: string, views: number }[] }>(`/analytics/trend?days=${days}`)
    trendData.value = resp.data.map(p => p.views)
  } catch {
    trendData.value = new Array(days).fill(0)
  }
}

const chartLabels = computed(() => {
  const labels: string[] = []
  const now = dayjs()
  for (let i = chartRange.value - 1; i >= 0; i--) {
    labels.push(now.subtract(i, 'day').format('MM/DD'))
  }
  return labels
})

const xAxisY = computed(() => 185)  // bottom of 200 SVG, labels at 199

const labelPoints = computed(() => {
  const labels = chartLabels.value
  const w = 400
  const pad = 20
  return labels.map((l, i) => ({
    x: pad + (i / Math.max(labels.length - 1, 1)) * (w - pad * 2),
    y: xAxisY.value + 14,
    label: l,
  }))
})

const points = computed(() => {
  const data = trendData.value
  const maxVal = Math.max(...data, 1)
  const w = 400
  const pad = 20
  const axisY = xAxisY.value
  const topPad = 15
  return data.map((v, i) => ({
    x: pad + (i / Math.max(data.length - 1, 1)) * (w - pad * 2),
    y: axisY - topPad - (v / maxVal) * (axisY - topPad * 2),
  }))
})

const linePath = computed(() => {
  if (points.value.length === 0) return ''
  let d = `M ${points.value[0].x} ${points.value[0].y}`
  for (let i = 1; i < points.value.length; i++) {
    d += ` L ${points.value[i].x} ${points.value[i].y}`
  }
  return d
})

const areaPath = computed(() => {
  if (points.value.length === 0) return ''
  const last = points.value[points.value.length - 1]
  const first = points.value[0]
  const bottom = xAxisY.value
  let d = linePath.value
  d += ` L ${last.x} ${bottom} L ${first.x} ${bottom} Z`
  return d
})

const totalViews = computed(() => trendData.value.reduce((a, b) => a + b, 0))
const avgViews = computed(() => Math.round(totalViews.value / Math.max(trendData.value.length, 1)))

const storagePct = computed(() => {
  if (storageTotal.value === 0) return 0
  return Math.round((storageUsed.value / storageTotal.value) * 100)
})

function formatSize(bytes: number) {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB'
}

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD HH:mm')
}

onMounted(async () => {
  loading.value = true
  try {
    const [allResp, draftResp] = await Promise.all([
      fetchAdminPosts({ page: 1, page_size: 1 }),
      fetchAdminPosts({ page: 1, page_size: 1, status: 'draft' }),
    ])
    stats.posts = allResp.data.pagination.total
    stats.drafts = draftResp.data.pagination.total
    stats.published = stats.posts - stats.drafts

    const recentResp = await fetchAdminPosts({ page: 1, page_size: 5 })
    recentPosts.value = recentResp.data.data

    try {
      const { data: viewsResp } = await api.get<{ data: number }>('/analytics/total-views')
      stats.views = viewsResp.data || 0
    } catch { /* keep 0 */ }

    try {
      const { data: likesResp } = await api.get<{ data: number }>('/analytics/total-likes')
      stats.likes = likesResp.data || 0
    } catch { /* keep 0 */ }

    try {
      const { data: commentsResp } = await api.get<{ data: number }>('/analytics/total-comments')
      stats.comments = commentsResp.data || 0
    } catch { /* keep 0 */ }

    try {
      const { data: guestbookResp } = await fetchGuestbook({ page: 1, page_size: 1 })
      stats.guestbookEntries = guestbookResp.pagination.total
    } catch { /* keep 0 */ }

    try {
      // authors only see pending comments on their own posts (matches CommentsAdmin behavior)
      const role = (() => {
        try {
          const stored = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
          return stored ? JSON.parse(stored).role : ''
        } catch { return '' }
      })()
      const params = (role !== 'admin' && role !== 'sub_admin') ? { scope: 'mine' } as any : undefined
      const { data: pendingResp } = await getCommentPendingCount(params)
      stats.pendingComments = pendingResp.data || 0
    } catch { /* keep 0 */ }

    try {
      const filesResp = await fetchFiles({ page: 1, page_size: 1 })
      const total = filesResp.data.pagination.total
      if (total > 0) {
        const allResp = await fetchFiles({ page: 1, page_size: Math.min(total, 100) })
        storageUsed.value = allResp.data.data.reduce((s: number, f: any) => s + (f.size || 0), 0)
      }
    } catch { /* ignore */ }

    await loadTrendData(chartRange.value)

    // Today's daily stats from trend data
    stats.dailyViews = trendData.value[trendData.value.length - 1] || 0

    try {
      const { data: todayLikesResp } = await api.get<{ data: number }>('/analytics/today-likes')
      stats.dailyLikes = todayLikesResp.data || 0
    } catch { /* keep 0 */ }

    try {
      const { data: todayPostsResp } = await api.get<{ data: { published: number, drafts: number } }>('/analytics/today-posts')
      stats.dailyPublished = todayPostsResp.data?.published || 0
      stats.dailyDrafts = todayPostsResp.data?.drafts || 0
    } catch { /* keep 0 */ }
  } finally {
    loading.value = false
  }
})

watch(chartRange, (days) => loadTrendData(days))
</script>

<style scoped>
.dashboard {
  animation: fadeIn 0.3s ease;
}
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--card-border-color);
}
.page-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ===== 统计卡片 5 列 ===== */
.stats-row {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
  margin-bottom: 28px;
}
.stat-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 20px 12px;
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  transition: border-color 0.2s;
  text-align: center;
}
.stat-card:hover {
  border-color: rgba(79, 70, 229, 0.3);
}
.stat-link {
  text-decoration: none;
  cursor: pointer;
}
.stat-icon {
  position: relative;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
.stat-icon.published { background: rgba(16, 185, 129, 0.12); color: #10b981; }
.stat-icon.drafts   { background: rgba(139, 92, 246, 0.12); color: #8b5cf6; }
.stat-icon.views    { background: rgba(245, 158, 11, 0.12); color: #f59e0b; }
.stat-icon.likes    { background: rgba(239, 68, 68, 0.12);  color: #f87171; }
.stat-icon.comments { background: rgba(107, 114, 128, 0.12); color: var(--text-secondary); }
.stat-icon.guestbook { background: rgba(16, 185, 129, 0.12); color: #34d399; }
.stat-body { display: flex; flex-direction: column; gap: 2px; }
.stat-value { font-size: 22px; font-weight: 700; color: var(--input-color); letter-spacing: -0.5px; }
.stat-label { font-size: 12px; color: var(--text-dim); }

/* ===== 快捷入口 5 列 ===== */
.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.quick-actions {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
  margin-bottom: 28px;
}
.quick-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 22px 12px;
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  text-decoration: none;
  transition: all 0.2s;
  cursor: pointer;
}
.quick-card:hover {
  border-color: rgba(79, 70, 229, 0.3);
  transform: translateY(-2px);
}
.quick-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.quick-icon.write   { background: rgba(79, 70, 229, 0.1);  color: #818cf8; }
.quick-icon.manage  { background: rgba(16, 185, 129, 0.1);  color: #34d399; }
.quick-icon.category { background: rgba(245, 158, 11, 0.1); color: #fbbf24; }
.quick-icon.tags-icon { background: rgba(239, 68, 68, 0.1);  color: #f87171; }
.quick-icon.settings { background: rgba(139, 92, 246, 0.1); color: #a78bfa; }
.quick-label { font-size: 14px; font-weight: 600; color: var(--input-color); }
.quick-desc { font-size: 12px; color: var(--text-dim); text-align: center; }

/* ===== 存储 ===== */
.storage-card {
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 28px;
}
.storage-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
}
.storage-label { font-size: 13px; color: var(--text-secondary); }
.storage-pct { font-size: 13px; color: #4f46e5; font-weight: 600; }
.storage-bar {
  height: 6px;
  background: var(--storage-bar-bg);
  border-radius: 3px;
  overflow: hidden;
  margin-bottom: 8px;
}
.storage-fill {
  height: 100%;
  background: linear-gradient(90deg, #4f46e5, #818cf8);
  border-radius: 3px;
  transition: width 0.5s ease;
}
.storage-hint {
  font-size: 12px;
  color: #4b5563;
}

/* ===== 底部两栏 ===== */
.bottom-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

/* ===== 最近文章 ===== */
.recent-posts {
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 20px;
}
.recent-posts .section-title {
  margin-bottom: 16px;
}
.post-list {
  display: flex;
  flex-direction: column;
}
.post-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 14px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  cursor: pointer;
  transition: padding-left 0.15s;
}
.post-item:last-child { border-bottom: none; }
.post-item:hover { padding-left: 6px; }
.post-thumb {
  width: 60px;
  height: 46px;
  object-fit: cover;
  border-radius: 6px;
  flex-shrink: 0;
  margin-right: 12px;
  margin-top: 2px;
}
.post-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
  flex: 1;
  text-align: left;
}
.post-title {
  font-size: 14px;
  color: var(--input-color);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.post-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
}
.post-date { color: var(--text-dim); }
.post-author {
  color: var(--text-secondary);
  max-width: 100px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.post-status {
  padding: 1px 8px;
  border-radius: 10px;
  font-size: 11px;
  font-weight: 500;
}
.post-status.published { background: rgba(16, 185, 129, 0.12); color: #34d399; }
.post-status.draft { background: rgba(107, 114, 128, 0.12); color: var(--text-secondary); }
.post-arrow { color: #4b5563; flex-shrink: 0; }
.empty-hint { padding: 20px 0; text-align: center; color: #4b5563; font-size: 13px; }
.view-all {
  display: block;
  text-align: center;
  margin-top: 16px;
  font-size: 13px;
  color: #4f46e5;
  text-decoration: none;
}
.view-all:hover { text-decoration: underline; }

/* ===== 阅读趋势 ===== */
.trend-chart {
  background: var(--card-bg);
  border: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
}
.chart-range {
  display: flex;
  gap: 2px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 8px;
  padding: 2px;
}
.chart-range button {
  border: none;
  background: transparent;
  color: var(--text-dim);
  padding: 4px 12px;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}
.chart-range button.active {
  background: rgba(79, 70, 229, 0.15);
  color: #818cf8;
}
.chart-container {
  flex: 1;
  display: flex;
  align-items: flex-end;
  margin-top: auto;
}
.chart-svg {
  width: 100%;
  height: 100%;
  min-height: 150px;
}
.chart-summary {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 13px;
  color: var(--text-secondary);
}
.chart-avg { color: #4f46e5; font-weight: 600; }

.stat-icon-badge {
  position: absolute;
  top: -6px;
  right: -6px;
  min-width: 18px;
  height: 18px;
  padding: 0 5px;
  border-radius: 9px;
  background: #ef4444;
  color: #fff;
  font-size: 11px;
  font-weight: 700;
  line-height: 18px;
  text-align: center;
}
.stat-icon-badge.orange {
  background: #f59e0b;
}

/* ===== Responsive ===== */
@media (max-width: 1024px) {
  .stats-row { grid-template-columns: repeat(3, 1fr); }
  .quick-actions { grid-template-columns: repeat(3, 1fr); }
  .bottom-grid { grid-template-columns: 1fr; }
}
@media (max-width: 640px) {
  .stats-row { grid-template-columns: repeat(2, 1fr); }
  .quick-actions { grid-template-columns: repeat(2, 1fr); }
}
</style>
