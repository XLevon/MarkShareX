<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- Page Header -->
    <div class="mb-10">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--color-text)' }">标签</h1>
      <p class="text-sm" :style="{ color: 'var(--color-text-muted)' }">浏览所有标签，发现感兴趣的内容</p>
    </div>

    <!-- Search -->
    <div class="relative max-w-md mb-10">
      <input
        v-model="search"
        type="text"
        placeholder="搜索标签..."
        class="w-full px-4 py-2.5 pl-10 rounded-xl border outline-none text-sm transition-colors"
        :style="{
          backgroundColor: 'var(--color-bg-card)',
          borderColor: 'var(--color-border)',
          color: 'var(--color-text)',
        }"
        @focus="($event.target as HTMLElement).style.borderColor = 'var(--color-primary)'"
        @blur="($event.target as HTMLElement).style.borderColor = 'var(--color-border)'"
      />
      <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4" :style="{ color: 'var(--color-text-muted)' }" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
      </svg>
    </div>

    <!-- Tags Grid -->
    <h2 class="text-lg font-bold mb-4" :style="{ color: 'var(--color-text)' }">
      {{ search ? `搜索结果（${filteredTags.length}）` : '热门标签' }}
    </h2>
    <div v-if="loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
    <div v-else class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
      <router-link
        v-for="tag in displayTags"
        :key="tag.id"
        :to="`/tag/${tag.slug}`"
        class="flex items-center justify-between px-4 py-3 rounded-xl border transition-all no-underline"
        :style="{
          backgroundColor: 'var(--color-bg-card)',
          borderColor: 'var(--color-border)',
          boxShadow: 'var(--shadow-card)',
        }"
        @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-primary)'; (e.currentTarget as HTMLElement).style.boxShadow = 'var(--shadow-card-hover)' }"
        @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-border)'; (e.currentTarget as HTMLElement).style.boxShadow = 'var(--shadow-card)' }"
      >
        <span class="font-medium text-sm" :style="{ color: 'var(--color-text)' }">{{ tag.name }}</span>
        <span class="text-xs px-2 py-0.5 rounded-full" :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }">
          {{ tag.post_count || 0 }}
        </span>
      </router-link>
    </div>
    <div v-if="!loading && displayTags.length === 0" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">
      {{ search ? '未找到匹配的标签' : '暂无标签' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { fetchTags } from '@/api/tags'
import type { Tag } from '@/api/index'

const search = ref('')
const loading = ref(false)
const tags = ref<Tag[]>([])

// All tags matching the search (search across ALL tags)
const filteredTags = computed(() => {
  if (!search.value.trim()) return tags.value
  const q = search.value.toLowerCase()
  return tags.value.filter(t => t.name.toLowerCase().includes(q))
})

// Display: top 10 by post_count when no search; all matches when searching
const displayTags = computed(() => {
  if (search.value.trim()) {
    return filteredTags.value
  }
  // Top 10 tags with post_count > 0, sorted descending
  return tags.value
    .filter(t => (t.post_count || 0) > 0)
    .sort((a, b) => (b.post_count || 0) - (a.post_count || 0))
    .slice(0, 10)
})

onMounted(async () => {
  loading.value = true
  try {
    const resp = await fetchTags()
    tags.value = resp.data.data
  } catch { /* ignore */ }
  finally { loading.value = false }
})
</script>
