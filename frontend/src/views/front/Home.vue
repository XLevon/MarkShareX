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
              @focus="($event.target as HTMLElement).style.borderColor = 'var(--color-primary)'"
              @blur="($event.target as HTMLElement).style.borderColor = 'var(--color-border)'"
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
    <section class="max-w-4xl mx-auto px-4 py-12 md:pt-16">
      <!-- Title -->
      <div class="mb-4">
        <h1 class="text-3xl font-bold mb-2 flex items-center gap-2" :style="{ color: 'var(--color-text)' }">
          <svg width="28" height="28" viewBox="0 0 1024 1024" xmlns="http://www.w3.org/2000/svg">
            <defs>
              <linearGradient id="news-grad" x1="0" y1="0" x2="1" y2="1">
                <stop offset="0%" stop-color="#6366f1"></stop>
                <stop offset="100%" stop-color="#a855f7"></stop>
              </linearGradient>
            </defs>
            <rect width="880" height="880" x="72" y="72" rx="120" fill="none" stroke="url(#news-grad)" stroke-width="50"></rect>
            <line x1="72" y1="320" x2="952" y2="320" stroke="url(#news-grad)" stroke-width="50"></line>
            <line x1="72" y1="540" x2="600" y2="540" stroke="url(#news-grad)" stroke-width="50" stroke-linecap="round"></line>
            <line x1="72" y1="700" x2="400" y2="700" stroke="url(#news-grad)" stroke-width="50" stroke-linecap="round"></line>
            <circle cx="780" cy="610" r="100" fill="#f59e0b"></circle>
            <text x="780" y="645" text-anchor="middle" fill="#fff" font-size="110" font-weight="bold">N</text>
          </svg>
          每日简讯
        </h1>
        <p class="text-sm" :style="{ color: 'var(--color-text-muted)' }">浏览最新资讯，掌握行业动态</p>
      </div>
      <!-- Controls row -->
      <div class="flex flex-wrap items-center gap-2 mb-3">
        <!-- PC: NaiveUI 日期范围选择器 -->
          <n-date-picker v-if="!isMobile" v-model:value="newsDateRange" type="daterange" clearable size="small" class="w-[170px] shrink-0">
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
          <!-- 移动端：两个独立日期选择器 -->
          <template v-else>
            <n-date-picker v-model:value="newsRangeStart" type="date" clearable size="small" class="flex-1 min-w-0" placeholder="开始" @update:value="onMobileDateChange" />
            <n-date-picker v-model:value="newsRangeEnd" type="date" clearable size="small" class="flex-1 min-w-0" placeholder="结束" @update:value="onMobileDateChange" />
          </template>
          <input
            v-model="newsSearch"
            type="text"
            placeholder="搜索资讯..."
            class="flex-1 min-w-0 px-3 py-1.5 text-sm rounded-lg border outline-none transition-colors"
            :style="{
              backgroundColor: 'var(--color-bg-card)',
              borderColor: 'var(--color-border)',
              color: 'var(--color-text)',
            }"
            @focus="($event.target as HTMLInputElement).style.borderColor = 'var(--color-primary)'"
            @blur="($event.target as HTMLInputElement).style.borderColor = 'var(--color-border)'"
          />
      </div>
      <!-- Topic type filter pills -->
      <div class="flex flex-wrap gap-1.5 mb-4">
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

      <!-- Loading state -->
      <div v-if="newsLoading" class="text-center py-8" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>

      <!-- Empty state -->
      <div v-else-if="newsItems.length === 0" class="text-center py-8" :style="{ color: 'var(--color-text-muted)' }">
        没有找到相关资讯
      </div>

      <!-- News list -->
      <template v-else>
        <!-- Top pagination bar -->
        <div class="flex items-center justify-end mb-4 gap-2">
          <div class="flex items-center gap-1">
            <button
              :disabled="currentPage === 0"
              @click="goToPage(currentPage - 1)"
              class="w-8 h-8 text-base rounded border transition-colors select-none flex items-center justify-center leading-none font-bold"
              :style="currentPage === 0 ? { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-muted)', opacity: '0.3', cursor: 'not-allowed' } : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)', cursor: 'pointer' }"
            >‹</button>
            <template v-for="p in visiblePages" :key="p">
              <span v-if="p === -1" class="px-0.5 text-xs" :style="{ color: 'var(--color-text-muted)' }">…</span>
              <button v-else @click="goToPage(p - 1)" class="w-7 h-7 text-xs rounded border transition-colors select-none flex items-center justify-center" :style="(p - 1) === currentPage ? { backgroundColor: 'var(--color-primary)', borderColor: 'var(--color-primary)', color: '#fff', cursor: 'default' } : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)', cursor: 'pointer' }">{{ p }}</button>
            </template>
            <button
              :disabled="currentPage >= totalNewsPages - 1"
              @click="goToPage(currentPage + 1)"
              class="w-8 h-8 text-base rounded border transition-colors select-none flex items-center justify-center leading-none font-bold"
              :style="currentPage >= totalNewsPages - 1 ? { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-muted)', opacity: '0.3', cursor: 'not-allowed' } : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)', cursor: 'pointer' }"
            >›</button>
          </div>
          <select
            :value="newsPageSize"
            @change="changePageSize(Number(($event.target as HTMLSelectElement).value))"
            class="px-2 py-1 text-xs rounded border outline-none cursor-pointer"
            :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)' }"
          >
            <option v-for="size in pageSizeOptions" :key="size" :value="size">{{ size }} 条/页</option>
          </select>
        </div>

        <div class="space-y-4">
          <article
            v-for="item in newsItems"
            :key="item.id"
            :ref="el => { if (el) newsRefs.set(item.id, el as HTMLElement) }"
            class="p-5 rounded-xl border cursor-pointer transition-all duration-200 hover:shadow-lg hover:-translate-y-0.5"
            :style="{
              borderColor: 'var(--color-border)',
              backgroundColor: clickedNewsIds.has(item.id) ? (isDark ? '#1e293b' : '#eef2ff') : 'var(--color-bg-card)',
              scrollMarginTop: '80px',
            }"
            @click="toggleNews(item)"
          >
            <div class="flex items-start justify-between gap-4">
              <div class="flex-1 min-w-0">
                <h3 class="text-lg font-semibold flex items-start gap-2" :style="{ color: 'var(--color-text)' }">
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
              <svg class="flex-shrink-0 mt-0.5" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" :style="{ color: 'var(--color-text-muted)', opacity: 0.5 }">
                <path d="M3 21V9l9-9 2 2-7 7h5v12H3zm11 0V9l9-9 2 2-7 7h5v12h-9z"/>
              </svg>
              <span>{{ item.summary }}</span>
            </p>
            <!-- Expanded content -->
            <div v-if="expandedNewsId === item.id" class="mt-4 pt-4 border-t" :style="{ borderColor: 'var(--color-border)' }">
              <div v-if="newsLoadingId === item.id" class="text-center py-4 text-sm" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
              <div v-else class="text-sm leading-relaxed markdown-body p-4 rounded-lg border-l-2" :style="{ color: 'var(--color-text)', backgroundColor: isDark ? '#1e293b' : '#f1f5f9', borderLeftColor: isDark ? '#6366f1' : '#818cf8', maxWidth: 'none' }" v-html="item.content_html || renderMarkdown(item.content || '')" @click="onNewsContentClick"></div>
            </div>
          </article>
        </div>

        <!-- Bottom pagination -->
        <div class="flex items-center justify-end pt-6 pb-4 gap-2">
          <!-- Prev -->
          <button
            :disabled="currentPage === 0"
            @click="goToPage(currentPage - 1)"
            class="w-8 h-8 text-base rounded border transition-colors select-none flex items-center justify-center leading-none font-bold"
            :style="currentPage === 0
              ? { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-muted)', opacity: '0.3', cursor: 'not-allowed' }
              : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)', cursor: 'pointer' }"
          >‹</button>

          <!-- Page numbers -->
          <template v-for="p in visiblePages" :key="p">
            <span v-if="p === -1" class="px-0.5 text-xs" :style="{ color: 'var(--color-text-muted)' }">…</span>
            <button
              v-else
              @click="goToPage(p - 1)"
              class="w-7 h-7 text-xs rounded border transition-colors select-none flex items-center justify-center"
              :style="(p - 1) === currentPage
                ? { backgroundColor: 'var(--color-primary)', borderColor: 'var(--color-primary)', color: '#fff', cursor: 'default' }
                : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)', cursor: 'pointer' }"
            >{{ p }}</button>
          </template>

          <!-- Next -->
          <button
            :disabled="currentPage >= totalNewsPages - 1"
            @click="goToPage(currentPage + 1)"
            class="w-8 h-8 text-base rounded border transition-colors select-none flex items-center justify-center leading-none font-bold"
            :style="currentPage >= totalNewsPages - 1
              ? { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-muted)', opacity: '0.3', cursor: 'not-allowed' }
              : { backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)', cursor: 'pointer' }"
          >›</button>

          <select
            :value="newsPageSize"
            @change="changePageSize(Number(($event.target as HTMLSelectElement).value))"
            class="px-2 py-1 text-xs rounded border outline-none cursor-pointer"
            :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)' }"
          >
            <option v-for="size in pageSizeOptions" :key="size" :value="size">{{ size }} 条/页</option>
          </select>
        </div>
      </template>
    </section>

    <GuestbookFormModal :visible="showGuestbookForm" @close="showGuestbookForm = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import { useDarkMode } from '@/composables/useDarkMode'
import { fetchPosts } from '@/api/posts'
import { fetchCategories } from '@/api/categories'
import { fetchTags } from '@/api/tags'
import { fetchNews, fetchNewsItem, fetchTopicTypes, type NewsItem } from '@/api/news'
import { renderMarkdown } from '@/utils/renderMarkdown'
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

// ── News section: paginated ──
const pageSizeOptions = [10, 20, 30, 50]
const newsPageSize = ref(10)
const totalNewsCount = ref(0)
const newsItems = ref<NewsItem[]>([])
const newsLoading = ref(false)
const currentPage = ref(0)
const newsPageData = ref<Map<number, NewsItem[]>>(new Map())  // page cache

const newsSearch = ref('')
const newsTopicFilters = ref(new Set<string>())
const newsDateRange = ref<[number, number] | null>(null)
const isMobile = ref(false)
const newsRangeStart = ref<number | null>(null)
const newsRangeEnd = ref<number | null>(null)
const calendarStartTime = computed(() => {
  const base = newsDateRange.value
    ? new Date(newsDateRange.value[0])
    : new Date()
  return new Date(base.getFullYear(), base.getMonth() - 1, 1).getTime()
})
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

function onMobileDateChange() {
  if (newsRangeStart.value && newsRangeEnd.value) {
    newsDateRange.value = [newsRangeStart.value, newsRangeEnd.value]
  } else {
    newsDateRange.value = null
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

async function toggleNews(item: NewsItem) {
  if (expandedNewsId.value === item.id) {
    expandedNewsId.value = null
    await nextTick()
    const el = newsRefs.get(item.id)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
    return
  }
  expandedNewsId.value = item.id
  clickedNewsIds.add(item.id)
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
  await nextTick()
  const el2 = newsRefs.get(item.id)
  if (el2) {
    el2.scrollIntoView({ behavior: 'smooth', block: 'start' })
  }
}

function onNewsContentClick(e: MouseEvent) {
  if ((e.target as HTMLElement).closest('a')) {
    e.stopPropagation()
  }
}

function buildNewsParams(params: Record<string, any>) {
  const q = newsSearch.value.trim()
  if (q) params.search = q
  if (newsTopicFilters.value.size > 0) {
    params.topic_type = Array.from(newsTopicFilters.value).join(',')
  }
  if (newsDateRange.value) {
    const [start, end] = newsDateRange.value
    const sd = new Date(start), ed = new Date(end)
    const pad = (n: number) => n.toString().padStart(2, '0')
    params.date_from = `${sd.getFullYear()}-${pad(sd.getMonth()+1)}-${pad(sd.getDate())}`
    params.date_to = `${ed.getFullYear()}-${pad(ed.getMonth()+1)}-${pad(ed.getDate())}`
  }
}

async function loadPage(pageNum: number) {
  // Check cache first
  if (newsPageData.value.has(pageNum)) {
    newsItems.value = newsPageData.value.get(pageNum)!
    currentPage.value = pageNum
    window.scrollTo({ top: 0, behavior: 'smooth' })
    return
  }

  newsLoading.value = true
  try {
    const params: Record<string, any> = { page: pageNum + 1, page_size: newsPageSize.value }
    buildNewsParams(params)
    const resp = await fetchNews(params)
    const data = resp.data.data || []
    totalNewsCount.value = resp.data.pagination?.total || data.length
    newsItems.value = data
    currentPage.value = pageNum
    newsPageData.value.set(pageNum, data)
    window.scrollTo({ top: 0, behavior: 'smooth' })
  } catch { /* ignore */ }
  finally { newsLoading.value = false }
}

function goToPage(pageNum: number) {
  if (pageNum < 0 || pageNum >= totalNewsPages.value) return
  loadPage(pageNum)
}

function changePageSize(size: number) {
  if (newsPageSize.value === size) return
  newsPageSize.value = size
  newsPageData.value.clear()
  currentPage.value = 0
  loadPage(0)
}

// Computed: total pages
const totalNewsPages = computed(() => Math.max(1, Math.ceil(totalNewsCount.value / newsPageSize.value)))

// Computed: visible page numbers for pagination
const visiblePages = computed(() => {
  const total = totalNewsPages.value
  const cur = currentPage.value
  const pages: number[] = []

  if (total <= 5) {
    for (let i = 1; i <= total; i++) pages.push(i)
    return pages
  }

  // Always show first page
  pages.push(1)
  if (cur > 3) pages.push(-1)  // ellipsis

  // Show current ± 1
  for (let i = Math.max(2, cur); i <= Math.min(total - 1, cur + 2); i++) {
    pages.push(i)
  }

  if (cur < total - 3) pages.push(-1)  // ellipsis
  // Always show last page
  pages.push(total)

  return pages
})

async function loadTopicTypes() {
  try {
    const params: Record<string, string> = {}
    if (newsDateRange.value) {
      const [start, end] = newsDateRange.value
      const sd = new Date(start), ed = new Date(end)
      const pad = (n: number) => n.toString().padStart(2, '0')
      params.date_from = `${sd.getFullYear()}-${pad(sd.getMonth()+1)}-${pad(sd.getDate())}`
      params.date_to = `${ed.getFullYear()}-${pad(ed.getMonth()+1)}-${pad(ed.getDate())}`
    }
    const q = newsSearch.value.trim()
    if (q) params.search = q
    const resp = await fetchTopicTypes(params)
    topicTypeValues.value = resp.data.data || []
  } catch { /* ignore */ }
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

// Reset and reload when filters/search change
let searchTimer: ReturnType<typeof setTimeout> | null = null
function resetAndReload() {
  newsPageData.value.clear()
  currentPage.value = 0
  loadPage(0)
}

watch(newsSearch, () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => resetAndReload(), 400)
})

watch(newsTopicFilters, () => resetAndReload(), { deep: true })
watch(newsDateRange, () => resetAndReload())

watch([newsDateRange, newsSearch], () => { loadTopicTypes() })
watch(() => route.query.search, () => loadStats())

onMounted(() => {
  // 响应式检测移动端
  const mq = window.matchMedia('(max-width: 475px)')
  isMobile.value = mq.matches
  mq.addEventListener('change', (e) => { isMobile.value = e.matches })
  loadStats()
  loadPage(0)
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
</script>
