<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- 标题行：分类名 + 返回按钮 -->
    <div class="mb-8 flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold" :style="{ color: 'var(--color-text)' }">{{ categoryName }}</h1>
        <p v-if="categoryDescription" class="text-sm mt-1" :style="{ color: 'var(--color-text-secondary)' }">{{ categoryDescription }}</p>
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
import { fetchPosts } from '@/api/posts'
import { fetchCategories } from '@/api/categories'
import type { Post, Category } from '@/api/index'
import PostCard from '@/components/shared/PostCard.vue'
import { useDocumentTitle } from '@/composables/useDocumentTitle'


const router = useRouter()
const route = useRoute()
const settingsStore = useSettingsStore()

const loading = ref(false)
const loadingMore = ref(false)
const posts = ref<Post[]>([])
const categories = ref<Category[]>([])
const page = ref(1)
const total = ref(0)
const hasMore = ref(false)
const batchSize = computed(() => Number(settingsStore.settings.batch_load_size) || 5)
const scrollSize = computed(() => Number(settingsStore.settings.scroll_load_size) || 3)
const sentinelRef = ref<HTMLElement | null>(null)
let observer: IntersectionObserver | null = null

const categoryName = computed(() => {
  const slug = route.params.slug as string
  return categories.value.find((c) => c.slug === slug)?.name || slug
})
useDocumentTitle(categoryName)

const categoryDescription = computed(() => {
  const slug = route.params.slug as string
  return categories.value.find((c) => c.slug === slug)?.description || ''
})

async function loadFirst() {
  const slug = route.params.slug as string
  loading.value = true
  page.value = 1
  try {
    if (!categories.value.length) {
      const { data: catResp } = await fetchCategories()
      categories.value = catResp.data
    }
    const cat = categories.value.find((c) => c.slug === slug)
    const params: Record<string, any> = { page: 1, page_size: batchSize.value, status: 'published' }
    if (cat) params.category_id = cat.id
    const { data: resp } = await fetchPosts(params)
    posts.value = resp.data
    total.value = resp.pagination.total
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
    const slug = route.params.slug as string
    const cat = categories.value.find((c) => c.slug === slug)
    const params: Record<string, any> = { page: page.value, page_size: scrollSize.value, status: 'published' }
    if (cat) params.category_id = cat.id
    const { data: resp } = await fetchPosts(params)
    posts.value.push(...resp.data)
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
watch(() => route.params.slug, async () => { observer?.disconnect(); await loadFirst(); await nextTick(); setupObserver() })
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

/* ===== Load More ===== */
.load-more-wrap {
  display: flex; justify-content: center; margin-top: 28px;
}
.load-more-btn {
  padding: 10px 40px; border-radius: 10px; border: 1px solid var(--color-border);
  background: var(--color-bg-card); color: var(--color-text-secondary);
  font-size: 14px; cursor: pointer; transition: all 0.15s;
}
.load-more-btn:hover:not(:disabled) {
  border-color: var(--color-primary); color: var(--color-primary); background: var(--color-primary-bg);
}
.load-more-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
