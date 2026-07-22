<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- ===== 列表视图（无 code 参数：显示所有类型/状态） ===== -->
    <template v-if="!filterValue">
      <!-- Page Header -->
      <div class="mb-10">
        <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--color-text)' }">
          {{ props.filterType === 'type' ? '📋 文章类型' : '🔄 文章状态' }}
        </h1>
        <p class="text-sm" :style="{ color: 'var(--color-text-muted)' }">
          {{ props.filterType === 'type' ? '按创作方式分类浏览文章' : '按内容时效性筛选文章' }}
        </p>
      </div>

      <!-- Grid -->
      <h2 class="text-lg font-bold mb-4" :style="{ color: 'var(--color-text)' }">
        {{ props.filterType === 'type' ? '全部类型' : '全部状态' }}
      </h2>
      <div v-if="loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
      <div v-else class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
        <router-link
          v-for="item in metaList"
          :key="item.code"
          :to="props.filterType === 'type' ? `/type/${item.code}` : `/status/${item.code}`"
          class="flex items-center justify-between px-4 py-3 rounded-xl border transition-all no-underline"
          :style="{
            backgroundColor: 'var(--color-bg-card)',
            borderColor: item.color + '60',
            boxShadow: 'var(--shadow-card)',
          }"
          @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-primary)'; (e.currentTarget as HTMLElement).style.boxShadow = 'var(--shadow-card-hover)' }"
          @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = item.color + '60'; (e.currentTarget as HTMLElement).style.boxShadow = 'var(--shadow-card)' }"
        >
          <span class="font-medium text-sm flex items-center gap-2" :style="{ color: 'var(--color-text)' }">
            <span class="w-2 h-2 rounded-full flex-shrink-0" :style="{ backgroundColor: item.color }"></span>
            {{ item.display_name }}
          </span>
          <span class="text-xs px-2 py-0.5 rounded-full" :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }">
            {{ item.post_count || 0 }}
          </span>
        </router-link>
      </div>
    </template>

    <!-- ===== 文章列表视图（有 code 参数：PostCard mode=front） ===== -->
    <template v-else>
      <div class="mb-8 flex items-center justify-between">
        <div>
          <h1 class="text-3xl font-bold" :style="{ color: 'var(--color-text)' }">
            {{ listTitle }}
          </h1>
          <p class="text-sm mt-2" :style="{ color: 'var(--color-text-muted)' }">共 {{ total }} 篇文章</p>
        </div>
        <button class="back-btn" @click="goBack">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
          返回
        </button>
      </div>

      <div v-if="loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
      <div v-else-if="posts.length === 0" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">
        <p class="text-lg mb-2">暂无文章</p>
        <p class="text-sm">该分类下还没有文章</p>
      </div>
      <div v-else class="post-cards">
        <PostCard
          v-for="post in posts"
          :key="post.id"
          :post="post"
          mode="front"
          class="mb-3"
        />
      </div>

      <!-- Load More -->
      <div ref="sentinelRef" class="load-more-sentinel">
        <div v-if="loadingMore" class="flex justify-center py-4"><div class="spinner"></div></div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { fetchPosts, type PostsParams } from '@/api/posts'
import { fetchArticleTypes, fetchArticleStatuses, type ArticleType, type ArticleStatus } from '@/api/admin'
import type { Post } from '@/api/index'
import PostCard from '@/components/shared/PostCard.vue'
import { useDocumentTitle } from '@/composables/useDocumentTitle'

const route = useRoute()
const router = useRouter()
const settingsStore = useSettingsStore()
const posts = ref<Post[]>([])
const loading = ref(true)
const loadingMore = ref(false)
const page = ref(1)
const total = ref(0)
const hasMore = ref(false)
const batchSize = computed(() => Number(settingsStore.settings.batch_load_size) || 5)
const sentinelRef = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

const props = defineProps<{ filterType: 'type' | 'status' }>()

const filterValue = computed(() => route.params.code as string || '')

const typeMap = ref<Record<string, string>>({})
const statusMap = ref<Record<string, string>>({})

const listTitle = computed(() => {
  if (!filterValue.value) return ''
  const label = props.filterType === 'type'
    ? (typeMap.value[filterValue.value] || filterValue.value)
    : (statusMap.value[filterValue.value] || filterValue.value)
  return label
})
const documentPageTitle = computed(() => listTitle.value || (props.filterType === 'type' ? '文章类型' : '文章状态'))
useDocumentTitle(documentPageTitle)

interface MetaItem { code: string; display_name: string; color: string; post_count: number }
const metaList = ref<MetaItem[]>([])

interface RequestContext {
  generation: number
  filterType: 'type' | 'status'
  code: string
  pageSize: number
}

let mounted = false
let requestGeneration = 0

function isCurrentRequest(context: RequestContext) {
  return mounted
    && context.generation === requestGeneration
    && context.filterType === props.filterType
    && context.code === filterValue.value
}

function currentRequestContext(): RequestContext {
  return {
    generation: ++requestGeneration,
    filterType: props.filterType,
    code: filterValue.value,
    // The backend uses page-number pagination, so every page in one list
    // generation must use the same size or offsets will overlap/skip rows.
    pageSize: batchSize.value,
  }
}

async function reloadForCurrentRoute() {
  const context = currentRequestContext()
  observer?.disconnect()
  observer = null
  page.value = 1
  loading.value = true
  loadingMore.value = false

  try {
    if (context.filterType === 'type') {
      const res = await fetchArticleTypes()
      if (!isCurrentRequest(context)) return
      const items = (res.data?.data ?? []).map((item: ArticleType) => ({
        code: item.code,
        display_name: item.display_name,
        color: item.color,
        post_count: item.post_count,
      }))
      metaList.value = items
      typeMap.value = Object.fromEntries(items.map((item) => [item.code, item.display_name]))
    } else {
      const res = await fetchArticleStatuses()
      if (!isCurrentRequest(context)) return
      const items = (res.data?.data ?? []).map((item: ArticleStatus) => ({
        code: item.code,
        display_name: item.display_name,
        color: item.color,
        post_count: item.post_count,
      }))
      metaList.value = items
      statusMap.value = Object.fromEntries(items.map((item) => [item.code, item.display_name]))
    }
  } catch {
    if (!isCurrentRequest(context)) return
    metaList.value = []
    if (context.filterType === 'type') typeMap.value = {}
    else statusMap.value = {}
  }

  if (!isCurrentRequest(context)) return

  if (!context.code) {
    posts.value = []
    total.value = 0
    hasMore.value = false
  } else {
    try {
      const params: PostsParams = { page: 1, page_size: context.pageSize }
      if (context.filterType === 'type') params.article_type = context.code
      else params.article_status = context.code

      const res = await fetchPosts(params)
      if (!isCurrentRequest(context)) return
      posts.value = res.data?.data ?? []
      total.value = res.data?.pagination?.total ?? 0
      hasMore.value = posts.value.length < total.value
    } catch {
      if (!isCurrentRequest(context)) return
      posts.value = []
      total.value = 0
      hasMore.value = false
    }
  }

  if (!isCurrentRequest(context)) return
  loading.value = false
  await nextTick()
  if (context.code && isCurrentRequest(context)) setupObserver(context)
}

async function loadMorePosts(context: RequestContext) {
  if (!isCurrentRequest(context) || !hasMore.value || loadingMore.value) return
  loadingMore.value = true
  const nextPage = page.value + 1
  try {
    const params: PostsParams = { page: nextPage, page_size: context.pageSize }
    if (context.filterType === 'type') params.article_type = context.code
    else params.article_status = context.code

    const res = await fetchPosts(params)
    if (!isCurrentRequest(context)) return
    posts.value.push(...(res.data?.data ?? []))
    page.value = nextPage
    hasMore.value = posts.value.length < total.value
  } catch {
    // Keep the current page so a later observer event can retry it.
  } finally {
    if (isCurrentRequest(context)) loadingMore.value = false
  }
}
function goBack() {
  const path = props.filterType === 'type' ? '/types' : '/statuses'
  router.push(path)
}

function setupObserver(context: RequestContext) {
  if (!sentinelRef.value || !isCurrentRequest(context)) return
  observer?.disconnect()
  observer = new IntersectionObserver(([entry]) => {
    if (entry.isIntersecting && isCurrentRequest(context) && hasMore.value && !loadingMore.value) {
      loadMorePosts(context)
    }
  }, { threshold: 0.1 })
  observer.observe(sentinelRef.value)
}

onMounted(() => {
  mounted = true
  reloadForCurrentRoute()
})
onUnmounted(() => {
  mounted = false
  requestGeneration++
  observer?.disconnect()
  observer = null
})

// 路由或筛选类型变化时，生成新的请求代次；旧响应不得提交状态。
watch(
  () => [route.path, route.params.code, props.filterType] as const,
  () => {
    reloadForCurrentRoute()
  },
)
</script>

<style scoped>
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  border-radius: 10px;
  border: 1px solid var(--color-border);
  background: var(--color-bg-card);
  color: var(--color-text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}
.back-btn:hover {
  border-color: var(--color-primary);
  color: var(--color-primary);
  background: var(--color-primary-bg);
}

.post-cards {
  display: flex;
  flex-direction: column;
}

/* ===== Sentinel + Spinner ===== */
.load-more-sentinel { min-height: 1px; }
.spinner {
  width: 24px; height: 24px; border: 2px solid var(--color-border);
  border-top-color: var(--color-primary); border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
