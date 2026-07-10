<template>
  <div class="home-page">
    <!-- Hero Section -->
    <section
      class="relative overflow-hidden px-4 transition-all duration-1000 ease-in-out"
      :style="{
        background: isDark ? 'linear-gradient(135deg, #1e1b4b 0%, #0f172a 50%, #1e293b 100%)' : 'linear-gradient(135deg, #eef2ff 0%, #f8fafc 50%, #ede9fe 100%)',
        maxHeight: heroVisible ? '800px' : '0px',
        paddingTop: heroVisible ? '64px' : '0px',
        paddingBottom: heroVisible ? '96px' : '0px',
        opacity: heroVisible ? '1' : '0',
      }"
    >
      <!-- Decorative blobs -->
      <div class="absolute top-0 right-0 w-96 h-96 rounded-full opacity-20 blur-3xl" :style="{ background: isDark ? '#4f46e5' : '#818cf8' }"></div>
      <div class="absolute bottom-0 left-0 w-72 h-72 rounded-full opacity-10 blur-3xl" :style="{ background: isDark ? '#6366f1' : '#a5b4fc' }"></div>

      <div class="max-w-4xl mx-auto text-center relative z-10">
        <h1 class="text-5xl md:text-6xl lg:text-7xl font-extrabold mb-6 tracking-tight leading-tight">
          <span v-for="(part, i) in titleParts" :key="i" :style="{ color: part.color }">{{ part.text }}</span>
        </h1>
        <p class="text-xl md:text-2xl mb-4 leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }">
          {{ settingsStore.settings.site_subtitle || '' }}
        </p>
        <p class="text-base md:text-lg mb-10 leading-relaxed whitespace-pre-line" :style="{ color: 'var(--color-text-muted)' }">
          {{ settingsStore.settings.site_description || '' }}
        </p>

        <!-- Search -->
        <div ref="heroSearchRef" class="flex justify-center mb-8">
          <div class="flex w-full max-w-lg">
            <input
              v-model="heroSearch"
              type="text"
              placeholder="搜索文章、标签或作者..."
              class="flex-1 px-4 py-3 text-base rounded-l-xl border outline-none transition-colors"
              :style="{
                backgroundColor: 'var(--color-bg-card)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)',
              }"
              @keyup.enter="doHeroSearch"
              @focus="$event.target.style.borderColor = 'var(--color-primary)'"
              @blur="$event.target.style.borderColor = 'var(--color-border)'"
            />
            <button
              @click="doHeroSearch"
              class="px-6 py-3 text-white font-medium rounded-r-xl transition-colors"
              :style="{ backgroundColor: 'var(--color-primary)' }"
            >
              搜索
            </button>
          </div>
        </div>

        <!-- CTA Buttons -->
        <div class="flex justify-center gap-4">
          <button
            @click="goWrite"
            class="px-6 py-3 rounded-xl text-white font-medium transition-all hover:shadow-lg no-underline border-0 cursor-pointer"
            :style="{ backgroundColor: 'var(--color-primary)' }"
          >
            开始分享
          </button>
          <router-link
            to="/knowledge-base"
            class="px-6 py-3 rounded-xl font-medium transition-all no-underline border"
            :style="{
              color: 'var(--color-text)',
              borderColor: 'var(--color-border)',
              backgroundColor: 'var(--color-bg-card)',
            }"
          >
            探索知识
          </router-link>
          <button
            @click="showGuestbookForm = true"
            class="px-6 py-3 rounded-xl font-medium transition-all no-underline border cursor-pointer"
            :style="{
              color: 'var(--color-text)',
              borderColor: 'var(--color-border)',
              backgroundColor: 'var(--color-bg-card)',
            }"
          >
            💬 留言反馈
          </button>
        </div>

        <!-- Stats -->
        <div class="flex justify-center gap-8 md:gap-16 mt-12">
          <router-link to="/knowledge-base" class="text-center no-underline">
            <div class="text-2xl md:text-3xl font-bold" :style="{ color: 'var(--color-primary)' }">{{ totalPosts }}</div>
            <div class="text-xs md:text-sm mt-1" :style="{ color: 'var(--color-text-muted)' }">技术文章</div>
          </router-link>
          <router-link to="/categories" class="text-center no-underline">
            <div class="text-2xl md:text-3xl font-bold" :style="{ color: 'var(--color-primary)' }">{{ totalCategories }}</div>
            <div class="text-xs md:text-sm mt-1" :style="{ color: 'var(--color-text-muted)' }">专栏分类</div>
          </router-link>
          <router-link to="/tags" class="text-center no-underline">
            <div class="text-2xl md:text-3xl font-bold" :style="{ color: 'var(--color-primary)' }">{{ totalTags }}</div>
            <div class="text-xs md:text-sm mt-1" :style="{ color: 'var(--color-text-muted)' }">内容标签</div>
          </router-link>
        </div>
      </div>
    </section>

    <!-- News Section -->
    <section v-if="newsItems.length > 0" class="max-w-4xl mx-auto px-4 pt-12 md:pt-16 pb-0">
      <!-- Header row: title + search + date -->
      <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-0 mb-4">
        <h2 class="text-lg md:text-xl font-bold" :style="{ color: 'var(--color-text)' }">📢 每日简讯</h2>
        <div class="hidden sm:block flex-1 min-w-2"></div>
        <div class="flex items-center gap-2">
          <n-date-picker v-model:value="newsDateRange" type="daterange" clearable size="small" style="width:200px">
            <template #footer>
              <div style="display:flex;gap:4px;flex-wrap:wrap;padding:8px 12px;border-top:1px solid var(--color-border)">
                <n-button size="tiny" quaternary @click="setNewsDateRange('today')">今天</n-button>
                <n-button size="tiny" quaternary @click="setNewsDateRange('yesterday')">昨天</n-button>
                <n-button size="tiny" quaternary @click="setNewsDateRange('week')">本周</n-button>
                <n-button size="tiny" quaternary @click="setNewsDateRange('lastWeek')">上周</n-button>
                <n-button size="tiny" quaternary @click="setNewsDateRange('month')">本月</n-button>
                <n-button size="tiny" quaternary @click="setNewsDateRange('lastMonth')">上月</n-button>
              </div>
            </template>
          </n-date-picker>
          <input
            v-model="newsSearch"
            type="text"
            placeholder="搜索资讯..."
            class="w-full sm:w-48 md:w-64 px-3 py-1.5 text-sm rounded-lg border outline-none transition-colors"
            :style="{
              backgroundColor: 'var(--color-bg-card)',
              borderColor: 'var(--color-border)',
              color: 'var(--color-text)',
            }"
            @focus="($event.target as HTMLInputElement).style.borderColor = 'var(--color-primary)'"
            @blur="($event.target as HTMLInputElement).style.borderColor = 'var(--color-border)'"
          />
        </div>
      </div>
      <!-- Topic type filter pills -->
      <div class="flex flex-wrap gap-1.5 mb-6">
        <button
          v-for="t in topicTypes"
          :key="t.value"
          @click="toggleNewsTopic(t.value)"
          class="px-2.5 py-1 text-xs rounded-full border transition-colors cursor-pointer select-none"
          :class="t.value === '' ? (newsTopicFilters.size === 0 ? 'border-transparent text-white' : 'hover:border-gray-400') : (newsTopicFilters.has(t.value) ? 'border-transparent text-white' : 'hover:border-gray-400')"
          :style="t.value === ''
            ? (newsTopicFilters.size === 0 ? { backgroundColor: 'var(--color-primary)', borderColor: 'var(--color-primary)', color: '#fff' } : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)' })
            : (newsTopicFilters.has(t.value) ? { backgroundColor: topicTypeColor(t.value, 1), borderColor: topicTypeColor(t.value, 1), color: '#fff' } : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)' })"
        >{{ t.label }}</button>
      </div>
      <div v-if="filteredNews.length === 0" class="text-center py-8" :style="{ color: 'var(--color-text-muted)' }">
        没有找到相关资讯
      </div>
      <div class="space-y-4">
        <article
          v-for="item in filteredNews"
          :key="item.id"
          :ref="el => { if (el) newsRefs.set(item.id, el as HTMLElement) }"
          class="p-5 rounded-xl border cursor-pointer transition-all duration-200"
          :style="{
            borderColor: 'var(--color-border)',
            backgroundColor: clickedNewsIds.has(item.id) ? (isDark ? '#1e293b' : '#eef2ff') : 'var(--color-bg-card)',
            scrollMarginTop: '80px',
          }"
          :class="{
            'hover:shadow-lg hover:-translate-y-0.5': true,
          }"
          @click="toggleNews(item)"
        >
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold flex items-start gap-2" :style="{ color: 'var(--color-text)' }">
                <!-- Title icon: news/bulletin -->
                <svg class="flex-shrink-0 mt-0.5" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" :style="{ color: 'var(--color-primary)' }">
                  <path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"/>
                  <path d="M18 14h-8M16 18H8"/>
                </svg>
                <span class="line-clamp-3 md:line-clamp-2">{{ item.title }}</span>
              </h3>
            </div>
            <div class="flex items-center gap-2">
              <span v-if="item.topic_type" class="text-xs px-1.5 py-0.5 rounded" :style="{ backgroundColor: topicTypeColor(item.topic_type, 0.15), color: topicTypeColor(item.topic_type, 1) }">{{ topicTypeLabel(item.topic_type) }}</span>
              <span class="text-xs whitespace-nowrap pt-0.5" :style="{ color: 'var(--color-text-muted)' }">{{ formatDate(item.published_at || item.created_at) }}</span>
            </div>
          </div>
          <p class="text-sm leading-relaxed line-clamp-2 mt-2 flex items-start gap-1.5 min-h-[2.5rem]" :style="{ color: item.summary ? 'var(--color-text-secondary)' : 'transparent' }">
            <!-- Summary icon: indent/quote -->
            <svg class="flex-shrink-0 mt-0.5" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" :style="{ color: 'var(--color-text-muted)', opacity: 0.5 }">
              <path d="M3 21V9l9-9 2 2-7 7h5v12H3zm11 0V9l9-9 2 2-7 7h5v12h-9z"/>
            </svg>
            <span>{{ item.summary }}</span>
          </p>
          <!-- Expanded content -->
          <div v-if="expandedNewsId === item.id" class="mt-4 pt-4 border-t" :style="{ borderColor: 'var(--color-border)' }">
            <div v-if="newsLoadingId === item.id" class="text-center py-4 text-sm" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
            <div v-else class="text-sm leading-relaxed markdown-body p-4 rounded-lg border-l-2" :style="{ color: 'var(--color-text)', backgroundColor: isDark ? '#1e293b' : '#f1f5f9', borderLeftColor: isDark ? '#6366f1' : '#818cf8', maxWidth: 'none' }" v-html="item.content_html || item.content" @click="onNewsContentClick"></div>
          </div>
        </article>
      </div>
      <div v-if="hasMore" ref="loadMoreRef" class="text-center py-6">
        <span v-if="loadingMore" :style="{ color: 'var(--color-text-muted)' }">加载中...</span>
      </div>
      <div v-else class="text-center pt-0 pb-1">
        <span :style="{ color: 'var(--color-text-muted)', fontSize: '13px' }">— 我是有底线的 —</span>
      </div>
    </section>

    <GuestbookFormModal :visible="showGuestbookForm" @close="showGuestbookForm = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import { useDarkMode } from '@/composables/useDarkMode'
import { fetchPosts } from '@/api/posts'
import { fetchCategories } from '@/api/categories'
import { fetchTags } from '@/api/tags'
import { fetchNews, fetchNewsItem, fetchTopicTypes, type NewsItem } from '@/api/news'
import GuestbookFormModal from '@/components/shared/GuestbookFormModal.vue'
import { navSearchVisible } from '@/composables/useSearchVisibility'
import { useHeroVisibility } from '@/composables/useHeroVisibility'
import { useTitleParts } from '@/composables/useTitleParts'

const route = useRoute()
const router = useRouter()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()
const { isDark } = useDarkMode()
const heroVisible = useHeroVisibility()

const titleParts = useTitleParts(
  () => settingsStore.settings.site_title || 'MarkShareX',
  () => isDark.value
)

const heroSearch = ref('')
const heroSearchRef = ref<HTMLElement | null>(null)
const showGuestbookForm = ref(false)

// News section
const newsItems = ref<NewsItem[]>([])
const newsPage = ref(1)
const hasMore = ref(false)
const loadingMore = ref(false)
const loadMoreRef = ref<HTMLElement | null>(null)
let newsObserver: IntersectionObserver | null = null

const newsSearch = ref('')
const newsTopicFilters = ref(new Set<string>())
const newsDateRange = ref<[number, number] | null>(null)
const topicTypeValues = ref<string[]>([])

const topicTypeLabelMap: Record<string, string> = {
  politics: '时政', finance: '财经', technology: '科技', society: '社会',
  entertainment: '文娱', sports: '体育', international: '国际', law: '法治', education: '教育',
}

const topicTypes = computed(() => {
  const types: { label: string; value: string }[] = [{ label: '全部', value: '' }]
  for (const v of topicTypeValues.value) {
    types.push({ label: topicTypeLabelMap[v] || v, value: v })
  }
  return types
})

const topicTypeColorMap: Record<string, string> = {
  politics: '#e74c3c',
  finance: '#e67e22',
  technology: '#3498db',
  society: '#1abc9c',
  entertainment: '#9b59b6',
  sports: '#27ae60',
  international: '#f39c12',
  law: '#34495e',
  education: '#e91e63',
}

function topicTypeLabel(v: string): string {
  return topicTypeLabelMap[v] || v
}

function topicTypeColor(v: string, alpha: number): string {
  const hex = topicTypeColorMap[v] || '#6b7280'
  if (alpha === 1) return hex
  const r = parseInt(hex.slice(1, 3), 16)
  const g = parseInt(hex.slice(3, 5), 16)
  const b = parseInt(hex.slice(5, 7), 16)
  return `rgba(${r},${g},${b},${alpha})`
}

function toggleNewsTopic(value: string) {
  if (value === '') {
    newsTopicFilters.value = new Set()
  } else {
    const next = new Set(newsTopicFilters.value)
    if (next.has(value)) next.delete(value)
    else next.add(value)
    newsTopicFilters.value = next
  }
}

function setNewsDateRange(preset: 'today' | 'yesterday' | 'week' | 'lastWeek' | 'month' | 'lastMonth') {
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
      const day = now.getDay() || 7
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
  newsDateRange.value = [start.getTime(), end.getTime()]
}
const newsRefs = new Map<number, HTMLElement>()
const clickedNewsIds = reactive(new Set<number>())
const expandedNewsId = ref<number | null>(null)
const newsLoadingId = ref<number | null>(null)

const batchSize = computed(() => parseInt(settingsStore.settings.batch_load_size || '5') || 5)
const scrollSize = computed(() => parseInt(settingsStore.settings.scroll_load_size || '3') || 3)

const filteredNews = computed(() => {
  let items = newsItems.value

  // Topic type filter (multi-select toggle)
  if (newsTopicFilters.value.size > 0) {
    items = items.filter(item => newsTopicFilters.value.has(item.topic_type))
  }

  // Date range filter
  if (newsDateRange.value) {
    const [rangeStart, rangeEnd] = newsDateRange.value
    items = items.filter(item => {
      const d = item.published_at || item.created_at
      if (!d) return false
      const ts = new Date(d).getTime()
      return ts >= rangeStart && ts <= rangeEnd
    })
  }

  // Text search
  const q = newsSearch.value.trim().toLowerCase()
  if (!q) return items
  return items.filter(item =>
    item.title.toLowerCase().includes(q) ||
    (item.summary && item.summary.toLowerCase().includes(q))
  )
})

async function toggleNews(item: NewsItem) {
  if (expandedNewsId.value === item.id) {
    expandedNewsId.value = null
    // 收起后滚动回标题行位置
    await nextTick()
    const el = newsRefs.get(item.id)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
    return
  }
  expandedNewsId.value = item.id
  clickedNewsIds.add(item.id)
  // Lazy load content if not already loaded
  if (!item.content && !item.content_html) {
    newsLoadingId.value = item.id
    try {
      const resp = await fetchNewsItem(item.id)
      const detail = resp.data.data
      item.content = detail.content
      item.content_html = detail.content_html
    } catch { /* ignore */ }
    finally { newsLoadingId.value = null }
  }
  // Scroll title row to just below navbar
  await nextTick()
  const el2 = newsRefs.get(item.id)
  if (el2) {
    el2.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

function onNewsContentClick(e: MouseEvent) {
  // 点击链接时不收起资讯（让链接正常打开）
  if ((e.target as HTMLElement).closest('a')) {
    e.stopPropagation()
  }
}

async function loadNewsInitial() {
  try {
    const resp = await fetchNews({ page: 1, page_size: batchSize.value })
    const data = resp.data.data || []
    newsItems.value = data
    newsPage.value = 1
    hasMore.value = data.length >= batchSize.value
  } catch { /* ignore */ }
}

async function loadTopicTypes() {
  try {
    const params: Record<string, string> = {}
    // Pass date range if set
    if (newsDateRange.value) {
      const [start, end] = newsDateRange.value
      const sd = new Date(start), ed = new Date(end)
      const pad = (n: number) => n.toString().padStart(2, '0')
      params.date_from = `${sd.getFullYear()}-${pad(sd.getMonth()+1)}-${pad(sd.getDate())}`
      params.date_to = `${ed.getFullYear()}-${pad(ed.getMonth()+1)}-${pad(ed.getDate())}`
    }
    // Pass search keyword
    const q = newsSearch.value.trim()
    if (q) params.search = q
    const resp = await fetchTopicTypes(params)
    topicTypeValues.value = resp.data.data || []
  } catch { /* ignore */ }
}

async function loadMoreNews() {
  if (loadingMore.value || !hasMore.value) return
  loadingMore.value = true
  try {
    const nextPage = newsPage.value + 1
    const resp = await fetchNews({ page: nextPage, page_size: batchSize.value })
    const data = resp.data.data || []
    newsItems.value.push(...data)
    newsPage.value = nextPage
    hasMore.value = data.length >= batchSize.value
  } catch { /* ignore */ }
  finally { loadingMore.value = false }
}

function formatDate(dateStr: string) {
  if (!dateStr) return ''
  const d = new Date(dateStr)
  return d.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}

const totalPosts = ref(0)
const totalCategories = ref(0)
const totalTags = ref(0)

async function loadStats() {
  try {
    const [catResp, tagResp, postResp] = await Promise.all([
      fetchCategories(),
      fetchTags(),
      fetchPosts({ page_size: 1, status: 'published' }),
    ])
    totalCategories.value = (catResp.data.data || []).length
    totalTags.value = (tagResp.data.data || []).length
    totalPosts.value = postResp.data.pagination?.total || 0
  } catch { /* ignore */ }
}

function doHeroSearch() {
  const q = heroSearch.value.trim()
  if (q) {
    router.push({ path: '/search', query: { q } })
  } else {
    router.push({ path: '/search', query: {} })
  }
}

function goWrite() {
  if (authStore.isAuthenticated) {
    const role = authStore.user?.role
    if (role === 'visitor') {
      router.push('/apply')
    } else {
      router.push('/admin/posts/new')
    }
  } else {
    router.replace('/login?redirect=/admin/posts/new')
  }
}

onMounted(() => {
  loadStats()
  loadNewsInitial()
  loadTopicTypes()
  setTimeout(() => { heroVisible.value = false }, 3000)
  if (heroSearchRef.value) {
    const observer = new IntersectionObserver(
      ([entry]) => { navSearchVisible.value = entry.isIntersecting ? false : true },
      { threshold: 0 }
    )
    observer.observe(heroSearchRef.value)
    navSearchVisible.value = false
    onUnmounted(() => { observer.disconnect(); navSearchVisible.value = true })
  }
})

// Setup infinite scroll observer after news items render
watch(hasMore, async (val) => {
  if (newsObserver) { newsObserver.disconnect(); newsObserver = null }
  if (val) {
    await nextTick()
    if (loadMoreRef.value) {
      newsObserver = new IntersectionObserver(
        ([entry]) => { if (entry.isIntersecting) loadMoreNews() },
        { threshold: 0 }
      )
      newsObserver.observe(loadMoreRef.value)
    }
  }
})

// Re-fetch topic types when date or search changes
watch([newsDateRange, newsSearch], () => { loadTopicTypes() })

watch(() => route.query.search, () => loadStats())
</script>
