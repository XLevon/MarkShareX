<template>
  <div class="home-page">
    <!-- Hero Section -->
    <section class="relative overflow-hidden py-16 md:py-24 px-4" :style="{ background: isDark ? 'linear-gradient(135deg, #1e1b4b 0%, #0f172a 50%, #1e293b 100%)' : 'linear-gradient(135deg, #eef2ff 0%, #f8fafc 50%, #ede9fe 100%)' }">
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

    <GuestbookFormModal :visible="showGuestbookForm" @close="showGuestbookForm = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useAuthStore } from '@/stores/auth'
import { useDarkMode } from '@/composables/useDarkMode'
import { fetchPosts } from '@/api/posts'
import { fetchCategories } from '@/api/categories'
import { fetchTags } from '@/api/tags'
import GuestbookFormModal from '@/components/shared/GuestbookFormModal.vue'
import { navSearchVisible } from '@/composables/useSearchVisibility'
import { useTitleParts } from '@/composables/useTitleParts'

const route = useRoute()
const router = useRouter()
const settingsStore = useSettingsStore()
const authStore = useAuthStore()
const { isDark } = useDarkMode()

const titleParts = useTitleParts(
  () => settingsStore.settings.site_title || 'MarkShareX',
  () => isDark.value
)

const heroSearch = ref('')
const heroSearchRef = ref<HTMLElement | null>(null)
const showGuestbookForm = ref(false)

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

watch(() => route.query.search, () => loadStats())
</script>
