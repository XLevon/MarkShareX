<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- ===== 列表视图（无 code 参数：显示所有类型/状态） ===== -->
    <template v-if="!filterValue">
      <!-- Page Header -->
      <div class="mb-10">
        <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--color-text)' }">
          {{ props.filterType === 'type' ? '文章类型' : '文章状态' }}
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
const scrollSize = computed(() => Number(settingsStore.settings.scroll_load_size) || 3)
const sentinelRef = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

const props = defineProps<{ filterType: 'type' | 'status' }>()

const filterValue = computed(() => route.params.code as string || '')

const listTitle = computed(() => {
  if (!filterValue.value) return ''
  const label = props.filterType === 'type'
    ? (typeMap.value[filterValue.value] || filterValue.value)
    : (statusMap.value[filterValue.value] || filterValue.value)
  return label
})

const typeMap = ref<Record<string, string>>({})
const statusMap = ref<Record<string, string>>({})

interface MetaItem { code: string; display_name: string; color: string; post_count: number }
const metaList = ref<MetaItem[]>([])

async function loadMeta() {
  loading.value = true
  try {
    if (props.filterType === 'type') {
      const res = await fetchArticleTypes()
      metaList.value = (res.data?.data ?? []).map((t: any) => ({
        code: t.code, display_name: t.display_name, color: t.color, post_count: t.post_count
      }))
      for (const t of metaList.value) typeMap.value[t.code] = t.display_name
    } else {
      const res = await fetchArticleStatuses()
      metaList.value = (res.data?.data ?? []).map((s: any) => ({
        code: s.code, display_name: s.display_name, color: s.color, post_count: s.post_count
      }))
      for (const s of metaList.value) statusMap.value[s.code] = s.display_name
    }
  } catch { metaList.value = [] }
  loading.value = false
}

async function loadPosts() {
  if (!filterValue.value) return
  page.value = 1
  loading.value = true
  try {
    const params: PostsParams = { page: 1, page_size: batchSize.value }
    if (props.filterType === 'type') params.article_type = filterValue.value
    else params.article_status = filterValue.value

    const res = await fetchPosts(params)
    posts.value = res.data?.data ?? []
    total.value = res.data?.pagination?.total ?? 0
    hasMore.value = posts.value.length < total.value
  } catch { posts.value = [] }
  loading.value = false
}

async function loadMorePosts() {
  if (!hasMore.value || loadingMore.value) return
  loadingMore.value = true
  page.value++
  try {
    const params: PostsParams = { page: page.value, page_size: scrollSize.value }
    if (props.filterType === 'type') params.article_type = filterValue.value
    else params.article_status = filterValue.value

    const res = await fetchPosts(params)
    posts.value.push(...(res.data?.data ?? []))
    hasMore.value = posts.value.length < total.value
  } catch { /* ignore */ }
  loadingMore.value = false
}
function goBack() {
  const path = props.filterType === 'type' ? '/types' : '/statuses'
  router.push(path)
}

function setupObserver() {
  if (!sentinelRef.value) return
  observer = new IntersectionObserver(([entry]) => {
    if (entry.isIntersecting && hasMore.value && !loadingMore.value) {
      loadMorePosts()
    }
  }, { threshold: 0.1 })
  observer.observe(sentinelRef.value)
}

onMounted(async () => {
  await loadMeta()
  if (filterValue.value) await loadPosts()
  await nextTick()
  setupObserver()
})
onUnmounted(() => { observer?.disconnect() })

// 监听路由切换
watch(() => route.path, async () => {
  observer?.disconnect()
  page.value = 1
  await loadMeta()
  if (filterValue.value) {
    await loadPosts()
    await nextTick()
    setupObserver()
  } else { posts.value = []; total.value = 0; hasMore.value = false }
})

watch(() => route.params.code, async (newVal) => {
  observer?.disconnect()
  if (newVal) {
    await loadPosts()
    await nextTick()
    setupObserver()
  } else { posts.value = []; total.value = 0; hasMore.value = false }
})
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
