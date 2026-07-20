<template>
  <div class="max-w-4xl mx-auto px-4 py-12">
    <!-- Page Header -->
    <div class="mb-10">
      <h1 class="text-3xl font-bold mb-2" :style="{ color: 'var(--color-text)' }">分类</h1>
      <p class="text-sm" :style="{ color: 'var(--color-text-muted)' }">浏览所有分类，发现感兴趣的内容</p>
    </div>

    <!-- Search -->
    <div class="relative max-w-md mb-10">
      <input
        v-model="search"
        type="text"
        placeholder="搜索分类..."
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

    <!-- Loading -->
    <div v-if="loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>

    <!-- Category List -->
    <div v-else-if="displayList.length" class="cat-list">
      <router-link
        v-for="cat in displayList"
        :key="cat.id"
        :to="`/category/${cat.slug}`"
        class="cat-row group"
        :class="{ 'is-child': cat._indent }"
      >
        <!-- Thumbnail -->
        <div class="cat-thumb" :class="{ 'img-placeholder': !cat.image_url }">
          <img v-if="cat.image_url" :src="cat.image_url" :alt="cat.name" referrerpolicy="no-referrer" @error="($event.target as HTMLImageElement).style.display='none'" />
          <span v-else class="placeholder-icon">📂</span>
        </div>

        <!-- Info -->
        <div class="cat-info">
          <span class="cat-name" :style="{ color: 'var(--color-text)' }">
            <span v-if="cat._indent" class="indent-mark">⤷</span>
            {{ cat.name }}
          </span>
          <span v-if="cat.description && !search" class="cat-desc">{{ cat.description }}</span>
        </div>

        <!-- Count -->
        <span class="cat-count" :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }">
          {{ cat.post_count || 0 }} 篇
        </span>

        <!-- Arrow -->
        <svg class="cat-arrow" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"><polyline points="9 18 15 12 9 6"/></svg>
      </router-link>
    </div>

    <div v-else-if="!loading" class="py-12 text-center" :style="{ color: 'var(--color-text-muted)' }">
      {{ search ? '没有匹配的分类' : '暂无分类' }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { fetchCategories } from '@/api/categories'
import type { Category } from '@/api/index'

interface TreeCategory extends Category {
  _indent?: number
  _children?: TreeCategory[]
}

const search = ref('')
const loading = ref(false)
const categories = ref<TreeCategory[]>([])

const treeNodes = computed<TreeCategory[]>(() => {
  const all = categories.value
  const parents = all.filter(c => !c.parent_id).sort((a, b) => a.sort_order - b.sort_order)
  const children = all.filter(c => c.parent_id)

  return parents.map(p => {
    const kids = children
      .filter(ch => ch.parent_id === p.id)
      .sort((a, b) => a.sort_order - b.sort_order)
      .map(ch => ({ ...ch, _indent: 1 }))
    return { ...p, _children: kids }
  })
})

// Flatten tree for list display: top 10 hot by post_count, or search results
const displayList = computed(() => {
  if (search.value.trim()) {
    // Search: match against ALL categories
    const q = search.value.toLowerCase()
    const result: TreeCategory[] = []
    for (const node of treeNodes.value) {
      if (node.name.toLowerCase().includes(q)) result.push(node)
      for (const child of node._children || []) {
        if (child.name.toLowerCase().includes(q)) result.push(child)
      }
    }
    return result
  }
  // No search: top 10 by post_count (flat, no hierarchy)
  const flat: TreeCategory[] = []
  for (const node of treeNodes.value) {
    flat.push({ ...node, _indent: undefined })
    for (const child of node._children || []) {
      flat.push({ ...child, _indent: undefined })
    }
  }
  return flat
    .sort((a, b) => (b.post_count || 0) - (a.post_count || 0))
    .slice(0, 10)
})

onMounted(async () => {
  loading.value = true
  try {
    const resp = await fetchCategories()
    categories.value = resp.data.data
  } catch { /* ignore */ }
  finally { loading.value = false }
})
</script>

<style scoped>
/* ── List ── */
.cat-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

/* ── Row ── */
.cat-row {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px 16px;
  border-radius: 12px;
  text-decoration: none;
  transition: background 0.15s;
}
.cat-row:hover {
  background: var(--color-bg-hover);
}
.cat-row.is-child {
  padding-left: 48px;
}

/* ── Thumbnail ── */
.cat-thumb {
  width: 52px;
  height: 52px;
  border-radius: 10px;
  overflow: hidden;
  flex-shrink: 0;
  background: var(--color-bg-hover);
}
.cat-thumb img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.cat-thumb.img-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
}
.placeholder-icon {
  font-size: 24px;
  opacity: 0.4;
}

/* ── Info ── */
.cat-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.cat-name {
  font-size: 15px;
  font-weight: 500;
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.indent-mark {
  margin-right: 2px;
  color: var(--color-text-muted);
  font-size: 12px;
}
.cat-desc {
  font-size: 12px;
  color: var(--color-text-secondary);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ── Count ── */
.cat-count {
  font-size: 12px;
  padding: 3px 12px;
  border-radius: 999px;
  flex-shrink: 0;
}

/* ── Arrow ── */
.cat-arrow {
  flex-shrink: 0;
  color: var(--color-text-muted);
  opacity: 0;
  transition: opacity 0.15s, transform 0.15s;
}
.cat-row:hover .cat-arrow {
  opacity: 1;
  transform: translateX(2px);
}
</style>
