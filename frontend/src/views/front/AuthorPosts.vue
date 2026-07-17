<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- 标题行：作者名 + 返回按钮 -->
    <div class="mb-8 flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold" :style="{ color: 'var(--color-text)' }">{{ authorName }}</h1>
        <p class="text-sm mt-2" :style="{ color: 'var(--color-text-muted)' }">共 {{ total }} 篇文章</p>
      </div>
      <button class="back-btn" @click="$router.back()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
        返回
      </button>
    </div>

    <div v-if="loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
    <div v-else-if="posts.length === 0" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">
      <p class="text-lg mb-2">暂无文章</p>
      <p class="text-sm">该作者还没有文章</p>
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

    <!-- 触底加载 -->
    <div ref="sentinelRef" class="load-more-sentinel">
      <div v-if="loadingMore" class="flex justify-center py-4">
        <div class="spinner"></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import api from '@/api/index'
import type { Post, PaginatedData } from '@/api/index'
import PostCard from '@/components/shared/PostCard.vue'
import { useDocumentTitle } from '@/composables/useDocumentTitle'

interface AuthorItem {
  id: number
  username: string
  display_name: string | null
  avatar_url: string | null
  bio: string | null
  post_count: number
}

const router = useRouter()
const route = useRoute()
const settingsStore = useSettingsStore()
const loading = ref(false)
const loadingMore = ref(false)
const posts = ref<Post[]>([])
const authors = ref<AuthorItem[]>([])
const page = ref(1)
const total = ref(0)
const hasMore = ref(false)
const batchSize = computed(() => Number(settingsStore.settings.batch_load_size) || 5)
const scrollSize = computed(() => Number(settingsStore.settings.scroll_load_size) || 3)
const sentinelRef = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

const authorId = computed(() => route.params.id as string)

const authorName = computed(() => {
  const a = authors.value.find(x => x.id === parseInt(authorId.value))
  return a?.display_name || a?.username || '作者'
})
useDocumentTitle(authorName)

async function loadFirst() {
  loading.value = true
  page.value = 1
  try {
    if (!authors.value.length) {
      const { data: authorResp } = await api.get<{ data: AuthorItem[] }>('/authors')
      authors.value = authorResp.data || []
    }
    const params: Record<string, any> = { page: 1, page_size: batchSize.value, status: 'published', author_id: authorId.value }
    const { data: resp } = await api.get<PaginatedData<Post>>('/posts', { params })
    posts.value = resp.data || []
    total.value = resp.pagination?.total || 0
    hasMore.value = posts.value.length < total.value
  } finally {
    loading.value = false
  }
}

async function loadMore() {
  if (!hasMore.value || loadingMore.value) return
  loadingMore.value = true
  page.value++
  try {
    const params: Record<string, any> = { page: page.value, page_size: scrollSize.value, status: 'published', author_id: authorId.value }
    const { data: resp } = await api.get<PaginatedData<Post>>('/posts', { params })
    posts.value.push(...(resp.data || []))
    hasMore.value = posts.value.length < total.value
  } finally {
    loadingMore.value = false
  }
}

function setupObserver() {
  if (!sentinelRef.value) return
  observer = new IntersectionObserver(([entry]) => {
    if (entry.isIntersecting && hasMore.value && !loadingMore.value) {
      loadMore()
    }
  }, { threshold: 0.1 })
  observer.observe(sentinelRef.value)
}

onMounted(async () => {
  await loadFirst()
  await nextTick()
  setupObserver()
})
onUnmounted(() => { observer?.disconnect() })
watch(() => route.params.id, async () => { observer?.disconnect(); await loadFirst(); await nextTick(); setupObserver() })
</script>

<style scoped>
/* ===== Back button ===== */
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
.dark .back-btn:hover {
  background: rgba(79, 70, 229, 0.1);
}

/* ===== Post Cards container ===== */
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
