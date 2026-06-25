<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <div class="mb-10">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--color-text)' }">📌 置顶推荐</h1>
      <p class="text-sm" :style="{ color: 'var(--color-text-muted)' }">共 {{ total }} 篇</p>
    </div>

    <div v-if="loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
    <div v-else-if="posts.length === 0" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">
      <p class="text-lg mb-2">暂无置顶文章</p>
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
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import { fetchPosts } from '@/api/posts'
import type { Post } from '@/api/index'
import PostCard from '@/components/shared/PostCard.vue'
import { useSettingsStore } from '@/stores/settings'

const posts = ref<Post[]>([])
const settingsStore = useSettingsStore()
const loading = ref(true)
const loadingMore = ref(false)
const page = ref(1)
const total = ref(0)
const hasMore = ref(false)
const batchSize = computed(() => Number(settingsStore.settings.batch_load_size) || 5)
const scrollSize = computed(() => Number(settingsStore.settings.scroll_load_size) || 3)
const sentinelRef = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

async function loadFirst() {
  loading.value = true
  page.value = 1
  try {
    const res = await fetchPosts({ is_pinned: true, page: 1, page_size: batchSize.value })
    posts.value = res.data?.data ?? []
    total.value = res.data?.pagination?.total ?? 0
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
    const res = await fetchPosts({ is_pinned: true, page: page.value, page_size: scrollSize.value })
    posts.value.push(...(res.data?.data ?? []))
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
</script>

<style scoped>
.post-cards { display: flex; flex-direction: column; }
.load-more-sentinel { min-height: 1px; }
.spinner {
  width: 24px; height: 24px; border: 2px solid var(--color-border);
  border-top-color: var(--color-primary); border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
