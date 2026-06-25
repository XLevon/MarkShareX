<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- Search input + back -->
    <div class="mb-8">
      <div class="flex items-center gap-3 mb-4">
        <button class="back-btn" @click="$router.back()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
          返回
        </button>
        <h1 class="text-2xl font-bold" :style="{ color: 'var(--color-text)' }">搜索结果</h1>
      </div>
      <div class="relative max-w-lg">
        <input
          v-model="query"
          type="text"
          placeholder="搜索文章、标签或作者..."
          class="w-full px-4 py-3 pl-11 rounded-xl border outline-none text-base transition-colors"
          :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
          @keyup.enter="doSearch"
          @focus="$event.target.style.borderColor = 'var(--color-primary)'"
          @blur="$event.target.style.borderColor = 'var(--color-border)'"
        />
        <svg class="absolute left-3.5 top-1/2 -translate-y-1/2 w-5 h-5" :style="{ color: 'var(--color-text-muted)' }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
        </svg>
      </div>
    </div>

    <div v-if="loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">搜索中...</div>

    <template v-else-if="searched">
      <!-- Articles -->
      <section v-if="articles.length" class="mb-10">
        <h2 class="text-lg font-bold mb-4 flex items-center gap-2" :style="{ color: 'var(--color-text)' }">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z"/></svg>
          文章（{{ articles.length }}）
        </h2>
        <div class="space-y-2">
          <router-link
            v-for="a in articles"
            :key="a.id"
            :to="`/post/${a.slug}`"
            class="flex items-center gap-3 px-4 py-3 rounded-xl border transition-all no-underline"
            :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)' }"
            @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-primary)' }"
            @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-border)' }"
          >
            <svg class="w-4 h-4 flex-shrink-0" :style="{ color: 'var(--color-text-muted)' }" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
            <span class="font-medium text-sm" :style="{ color: 'var(--color-text)' }">{{ a.title }}</span>
          </router-link>
        </div>
      </section>

      <!-- Tags -->
      <section v-if="tags.length" class="mb-10">
        <h2 class="text-lg font-bold mb-4 flex items-center gap-2" :style="{ color: 'var(--color-text)' }">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"/></svg>
          标签（{{ tags.length }}）
        </h2>
        <div class="flex flex-wrap gap-2">
          <router-link
            v-for="t in tags"
            :key="t.id"
            :to="`/tag/${t.slug}`"
            class="flex items-center gap-1.5 px-3 py-2 rounded-lg text-sm border transition-all no-underline"
            :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
            @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-primary)' }"
            @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-border)' }"
          >
            {{ t.name }}
            <span class="text-xs px-1.5 py-0.5 rounded" :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }">{{ t.post_count }}</span>
          </router-link>
        </div>
      </section>

      <!-- Authors -->
      <section v-if="authors.length" class="mb-10">
        <h2 class="text-lg font-bold mb-4 flex items-center gap-2" :style="{ color: 'var(--color-text)' }">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/></svg>
          作者（{{ authors.length }}）
        </h2>
        <div class="flex flex-wrap gap-2">
          <router-link
            v-for="a in authors"
            :key="a.id"
            :to="`/author/${a.id}`"
            class="flex items-center gap-2 px-3 py-2 rounded-lg text-sm border transition-all no-underline"
            :style="{ backgroundColor: 'var(--color-bg-card)', borderColor: 'var(--color-border)', color: 'var(--color-text)' }"
            @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-primary)' }"
            @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-border)' }"
          >
            <span class="w-6 h-6 rounded-full flex items-center justify-center text-white text-xs font-semibold" style="background: #4f46e5">{{ (a.display_name || a.username)[0].toUpperCase() }}</span>
            {{ a.display_name || a.username }}
          </router-link>
        </div>
      </section>

      <!-- No results -->
      <div v-if="!articles.length && !tags.length && !authors.length" class="py-16 text-center" :style="{ color: 'var(--color-text-muted)' }">
        <svg class="w-12 h-12 mx-auto mb-4 opacity-40" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></svg>
        <p class="text-lg">未找到与「{{ routeQuery }}」相关的结果</p>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import api from '@/api/index'

interface SearchArticle { id: number; title: string; slug: string }
interface SearchTag { id: number; name: string; slug: string; post_count: number }
interface SearchAuthor { id: number; username: string; display_name: string | null }
interface SearchData { articles: SearchArticle[]; tags: SearchTag[]; authors: SearchAuthor[] }

const route = useRoute()
const router = useRouter()
const query = ref((route.query.q as string) || '')
const routeQuery = ref((route.query.q as string) || '')
const loading = ref(false)
const searched = ref(false)
const articles = ref<SearchArticle[]>([])
const tags = ref<SearchTag[]>([])
const authors = ref<SearchAuthor[]>([])

function doSearch() {
  const q = query.value.trim()
  if (q) {
    router.push({ path: '/search', query: { q } })
  } else {
    // 清空搜索参数，显示空白/提示页面
    router.replace({ path: '/search', query: {} })
  }
}

async function loadResults() {
  const q = (route.query.q as string) || ''
  if (!q) return
  routeQuery.value = q
  query.value = q
  loading.value = true
  searched.value = false
  try {
    const { data: resp } = await api.get<{ data: SearchData }>('/search', { params: { q } })
    const d = resp.data
    articles.value = d?.articles || []
    tags.value = d?.tags || []
    authors.value = d?.authors || []
    searched.value = true
  } finally { loading.value = false }
}

onMounted(loadResults)
watch(() => route.query.q, loadResults)
</script>

<style scoped>
.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
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
</style>
