<template>
  <div class="knowledge-base-page">
    <!-- Main Content Area -->
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-12">
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-10">
        <!-- Article List -->
        <div class="lg:col-span-2">
          <div v-if="loading" class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>
          <div v-else-if="posts.length === 0" class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">
            <p class="text-lg mb-2">暂无文章</p>
            <p class="text-sm">开始写作你的第一篇文章吧</p>
          </div>

          <!-- Group by Category -->
          <template v-else>
            <section v-for="group in visibleCategorizedPosts" :key="group.category" class="mb-8 sm:mb-12">
              <!-- Category Header -->
              <div
                class="flex items-center gap-3 mb-6 cursor-pointer select-none group-header"
                @click="toggleGroup(group.category)"
              >
                <div class="flex items-center gap-2">
                  <div
                    class="flex items-center gap-2 px-3 py-1.5 rounded-lg"
                    :style="{ backgroundColor: getCategoryColor(group.category) + '20', color: getCategoryColor(group.category) }"
                  >
                    <span class="text-sm font-bold">{{ group.category }}</span>
                  </div>
                  <span
                    v-if="group.description"
                    class="text-xs truncate max-w-md hidden sm:inline"
                    :style="{ color: 'var(--color-text-secondary)' }"
                  >{{ group.description }}</span>
                </div>
                <div class="flex-1 h-px" :style="{ backgroundColor: 'var(--color-border-light)' }"></div>
                <router-link
                  v-if="group.slug"
                  :to="`/category/${group.slug}`"
                  class="text-xs font-medium no-underline hover:underline"
                  :style="{ color: 'var(--color-primary)' }"
                  @click.stop
                >{{ group.total }} 篇</router-link>
                <span
                  v-else
                  class="text-xs"
                  :style="{ color: 'var(--color-text-muted)' }"
                >{{ group.total }} 篇</span>
                <svg
                  class="w-4 h-4 transition-transform duration-300 flex-shrink-0"
                  :class="{ 'rotate-180': !collapsedGroups.has(group.category) }"
                  :style="{ color: 'var(--color-text-muted)' }"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
              </div>

              <!-- Article Cards -->
              <div
                class="transition-all duration-300"
                :class="collapsedGroups.has(group.category) ? 'overflow-hidden' : 'overflow-visible'"
                :style="collapsedGroups.has(group.category) ? { maxHeight: '0', opacity: '0', marginBottom: '0' } : { opacity: '1' }"
              >
              <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                  <div
                    v-for="post in group.posts.slice(0, scrollLoadSize)"
                    :key="post.id"
                    class="relative home-post-card"
                  >
                  <article
                    class="group rounded-xl border transition-all cursor-pointer flex flex-col overflow-hidden h-full"
                    :style="{
                      backgroundColor: 'var(--color-bg-card)',
                      borderColor: 'var(--color-border)',
                      boxShadow: 'var(--shadow-card)',
                    }"
                    @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.boxShadow = '0 8px 30px rgba(0,0,0,0.12)'; (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-primary)' }"
                    @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.boxShadow = 'var(--shadow-card)'; (e.currentTarget as HTMLElement).style.borderColor = 'var(--color-border)' }"
                  >
                    <router-link :to="`/post/${post.slug}`" class="block no-underline h-full flex flex-col overflow-hidden">
                      <!-- Cover: always 56.25% (16:9) reserved, placeholder if no image -->
                      <div class="w-full overflow-hidden" style="aspect-ratio: 16/9; flex-shrink: 0;">
                        <img
                          v-if="post.cover_image || post.category_cover_image"
                          :src="post.cover_image || post.category_cover_image"
                          class="w-full h-full object-cover transition-transform group-hover:scale-105"
                          alt=""
                          referrerpolicy="no-referrer"
                          loading="eager"
                        />
                        <div
                          v-else
                          class="w-full h-full flex items-center justify-center"
                          :style="{ backgroundColor: 'var(--color-bg-secondary)' }"
                        >
                          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" :style="{ color: 'var(--color-text-muted)', opacity: '0.35' }">
                            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
                            <polyline points="14 2 14 8 20 8"/>
                            <line x1="16" y1="13" x2="8" y2="13"/>
                            <line x1="16" y1="17" x2="8" y2="17"/>
                          </svg>
                        </div>
                      </div>

                      <!-- Text area: flex-1 fills remaining, flex-col for bottom anchoring -->
                      <div class="flex flex-col flex-1 p-3 min-w-0">
                        <!-- Title: always 2 lines max -->
                        <h3 class="text-sm font-bold group-hover:underline line-clamp-2" :style="{ color: 'var(--color-text)' }">
                          {{ post.title || '无标题' }}
                        </h3>

                        <!-- Summary: always 2 lines reserved -->
                        <p class="text-xs mt-1.5 line-clamp-2" :style="{ color: post.summary ? 'var(--color-text-secondary)' : 'var(--color-text-muted)' }">
                          {{ post.summary || '暂无摘要' }}
                        </p>

                        <!-- Spacer pushes footer to bottom -->
                        <div class="flex-1 min-h-0"></div>

                        <!-- Tags: always reserved row -->
                        <div class="flex flex-wrap gap-1 min-h-[20px] mt-1">
                          <span
                            v-for="tag in (post.tags || []).slice(0, 3)"
                            :key="tag"
                            class="px-1.5 py-0.5 text-xs rounded-full"
                            :style="{ backgroundColor: 'var(--color-primary-bg)', color: 'var(--color-primary)' }"
                          >
                            {{ tag }}
                          </span>
                        </div>

                        <!-- Stats: always visible row -->
                        <div class="flex items-center justify-between text-xs mt-1.5" :style="{ color: 'var(--color-text-muted)' }">
                          <div class="flex items-center gap-1">
                            <span>{{ post.author_name || '作者' }}</span>
                            <span>·</span>
                            <span>{{ dayjs(post.published_at || post.created_at).format('MM-DD') }}</span>
                            <span v-if="post.category_name">·</span>
                            <span v-if="post.category_name" class="text-[#f59e0b] text-xs px-1 py-0.5 rounded" :style="{ backgroundColor: 'rgba(245,158,11,0.08)' }">{{ post.category_name }}</span>
                          </div>
                          <div class="flex items-center gap-3">
                            <span class="flex items-center gap-1 transition-colors duration-200 group-hover:text-[var(--color-text)]">
                              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
                              {{ post.view_count || 0 }}
                            </span>
                            <span class="flex items-center gap-1 transition-colors duration-200 group-hover:text-[var(--color-text)]">
                              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"/></svg>
                              {{ post.like_count || 0 }}
                            </span>
                            <span class="flex items-center gap-1 transition-colors duration-200 group-hover:text-[var(--color-text)]">
                              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                              {{ post.comment_count || 0 }}
                            </span>
                          </div>
                        </div>
                      </div>
                    </router-link>
                  </article>

                    <!-- Brush-stroke highlights (outside article to avoid clipping) -->
                    <div v-if="post.is_pinned || getBadgeType(post) || getBadgeStatus(post)" class="home-edge-badges">
                      <div v-if="post.is_pinned" class="home-brush-badge home-brush-pinned">
                        <svg class="home-brush-stroke" viewBox="0 0 90 28" aria-hidden="true">
                          <path d="M5,2 C14,1 24,3 36,5 C50,7 64,2 78,2 C84,3 86,7 86,11 C84,21 70,25 56,24 C42,23 28,26 16,25 C8,24 3,20 3,14 C3,7 3,3 5,2 Z" />
                        </svg>
                        <span class="home-brush-text">📌 置顶</span>
                      </div>
                      <div v-if="getBadgeType(post)" class="home-brush-badge" :class="'home-brush-type-' + getBadgeType(post).key">
                        <svg class="home-brush-stroke" viewBox="0 0 90 28" aria-hidden="true">
                          <path d="M5,2 C14,1 24,3 36,5 C50,7 64,2 78,2 C84,3 86,7 86,11 C84,21 70,25 56,24 C42,23 28,26 16,25 C8,24 3,20 3,14 C3,7 3,3 5,2 Z" />
                        </svg>
                        <span class="home-brush-text">{{ getBadgeType(post).label }}</span>
                      </div>
                      <div v-if="getBadgeStatus(post)" class="home-brush-badge" :class="'home-brush-status-' + getBadgeStatus(post).key">
                        <svg class="home-brush-stroke" viewBox="0 0 90 28" aria-hidden="true">
                          <path d="M5,2 C14,1 24,3 36,5 C50,7 64,2 78,2 C84,3 86,7 86,11 C84,21 70,25 56,24 C42,23 28,26 16,25 C8,24 3,20 3,14 C3,7 3,3 5,2 Z" />
                        </svg>
                        <span class="home-brush-text">{{ getBadgeStatus(post).label }}</span>
                      </div>
                    </div>
                  </div>
              </div>
              </div>
            </section>
          </template>
        </div>

        <!-- Sidebar -->
        <aside class="space-y-6">
          <!-- Pinned Posts -->
          <SidebarCard v-if="pinnedPosts.length">
            <template #title>
              <router-link to="/pinned" class="no-underline flex items-center justify-between group" :style="{ color: 'var(--color-text)' }">
                <span>📌 置顶推荐</span>
                <span v-if="pinnedPosts.length > 5" class="text-xs flex items-center gap-0.5 flex-shrink-0" :style="{ color: 'var(--color-text-muted)' }">查看更多<svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/></svg></span>
              </router-link>
            </template>
            <div class="space-y-3 mt-3">
              <router-link
                v-for="post in pinnedPosts.slice(0, 5)"
                :key="post.id"
                :to="`/post/${post.slug}`"
                class="flex items-start gap-2.5 group no-underline"
              >
                <div class="w-10 h-10 rounded-lg overflow-hidden flex-shrink-0" :style="{ backgroundColor: 'var(--color-bg-hover)' }">
                  <img v-if="post.cover_image || post.category_cover_image" :src="post.cover_image || post.category_cover_image" class="w-full h-full object-cover" alt="" referrerpolicy="no-referrer" loading="eager" />
                  <div v-else class="w-full h-full flex items-center justify-center text-xs" :style="{ color: 'var(--color-text-muted)' }">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
                  </div>
                </div>
                <div class="min-w-0 flex-1">
                  <p class="text-sm font-medium truncate group-hover:underline" :style="{ color: 'var(--color-text)' }">
                    {{ post.title }}
                  </p>
                  <p class="text-xs mt-0.5" :style="{ color: 'var(--color-text-muted)' }">
                    {{ post.author_name || '作者' }}
                  </p>
                </div>
              </router-link>
            </div>
          </SidebarCard>

          <!-- Hot Articles -->
          <SidebarCard v-if="hotPosts.length">
            <template #title><span>🔥 热门文章</span></template>
            <div class="space-y-3 mt-3">
              <router-link
                v-for="(post, idx) in hotPosts.slice(0, 5)"
                :key="post.id"
                :to="`/post/${post.slug}`"
                class="flex items-start gap-3 group no-underline"
              >
                <span class="text-lg font-bold flex-shrink-0 w-6" :style="{ color: idx < 3 ? 'var(--color-primary)' : 'var(--color-text-muted)' }">
                  {{ idx + 1 }}
                </span>
                <div class="min-w-0">
                  <p class="text-sm font-medium truncate group-hover:underline" :style="{ color: 'var(--color-text)' }">
                    {{ post.title }}
                  </p>
                  <p class="text-xs mt-0.5" :style="{ color: 'var(--color-text-muted)' }">
                    {{ post.author_name || '作者' }}
                  </p>
                </div>
              </router-link>
            </div>
          </SidebarCard>

          <!-- Categories Tree -->
          <SidebarCard v-if="categories.length">
            <template #title>📂 专栏分类</template>
            <div class="space-y-0.5 mt-3">
              <template v-for="parent in sidebarCategories" :key="parent.id">
                <div
                  class="flex items-center justify-between px-2 py-1.5 rounded-lg text-sm cursor-pointer select-none transition-colors"
                  :style="{ color: 'var(--color-text)' }"
                  @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--color-bg-hover)' }"
                  @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent' }"
                >
                  <router-link
                    :to="`/category/${parent.slug}`"
                    class="flex items-center gap-1.5 flex-1 min-w-0 no-underline"
                    :style="{ color: 'var(--color-text)' }"
                    @click.stop
                  >
                    <span
                      v-if="parent._children?.length"
                      class="text-xs transition-transform duration-200 flex-shrink-0"
                      :style="{ transform: collapsedSidebar.has(parent.id) ? 'rotate(-90deg)' : '', color: 'var(--color-text-muted)' }"
                      @click.prevent.stop="toggleSidebar(parent.id)"
                    >▼</span>
                    <span v-else class="w-3 flex-shrink-0"></span>
                      <span class="font-medium">{{ parent.name }}</span>
                      <span v-if="parent.description" class="text-xs truncate" :style="{ color: 'var(--color-text-secondary)' }" :title="parent.description">· {{ parent.description }}</span>
                    </router-link>
                  <span class="text-xs px-1.5 py-0.5 rounded-full flex-shrink-0 ml-2" :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }">
                    {{ parent.post_count || 0 }}
                  </span>
                </div>
                <div v-if="parent._children?.length && !collapsedSidebar.has(parent.id)" class="pl-5 space-y-0.5">
                  <router-link
                    v-for="child in parent._children"
                    :key="child.id"
                    :to="`/category/${child.slug}`"
                    class="flex items-center justify-between px-2 py-1 rounded-lg text-sm transition-colors no-underline"
                    :style="{ color: 'var(--color-text)' }"
                    @mouseenter="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.backgroundColor = 'var(--color-bg-hover)' }"
                    @mouseleave="(e: MouseEvent) => { (e.currentTarget as HTMLElement).style.backgroundColor = 'transparent' }"
                  >
                    <span class="truncate">
                        <span class="mr-1" :style="{ color: 'var(--color-text-muted)' }">⤷</span>
                        {{ child.name }}
                        <span v-if="child.description" class="text-xs" :style="{ color: 'var(--color-text-secondary)' }" :title="child.description"> · {{ child.description }}</span>
                      </span>
                      <span class="text-xs px-1.5 py-0.5 rounded-full flex-shrink-0 ml-2" :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-muted)' }">
                        {{ child.post_count || 0 }}
                      </span>
                  </router-link>
                </div>
              </template>
            </div>
          </SidebarCard>

          <!-- Tag Cloud -->
          <SidebarCard v-if="tags.length">
            <template #title>🏷️ 热门标签</template>
            <div class="mt-3">
              <TagCloud :tags="tags" />
            </div>
          </SidebarCard>

          <!-- Friend Links -->
          <SidebarCard v-if="hasLinks">
            <template #title>🔗 友情链接</template>
            <div class="mt-3">
              <FriendLinks :data="settingsStore.settings.friend_links" />
            </div>
          </SidebarCard>

          <!-- Webmaster Info -->
          <SidebarCard v-if="siteManager?.bio">
            <template #title>👤 站长信息</template>
            <div class="mt-3">
              <WebmasterInfo :admin="siteManager" />
            </div>
          </SidebarCard>
        </aside>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, nextTick } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { fetchPosts, fetchPinnedPosts } from '@/api/posts'
import { fetchCategories } from '@/api/categories'
import { fetchTags } from '@/api/tags'
import { fetchSiteManagerInfo, type SiteManagerInfo } from '@/api/index'
import type { Post, Category, Tag } from '@/api/index'
import dayjs from 'dayjs'
import SidebarCard from '@/components/front/SidebarCard.vue'
import TagCloud from '@/components/front/TagCloud.vue'
import FriendLinks from '@/components/front/FriendLinks.vue'
import WebmasterInfo from '@/components/front/WebmasterInfo.vue'

const settingsStore = useSettingsStore()

const loading = ref(false)
const posts = ref<Post[]>([])
const pinnedPosts = ref<Post[]>([])
const categories = ref<Category[]>([])
const tags = ref<Tag[]>([])
const collapsedGroups = reactive(new Set<string>())
const collapsedSidebar = ref<Set<number>>(new Set())
const siteManager = ref<SiteManagerInfo | null>(null)
const visibleGroupCount = ref(0)
const bottomSentinel = ref<HTMLElement | null>(null)
const scrollLoadSize = computed(() => Number(settingsStore.settings.scroll_load_size) || 3)
const categoryPostTotals = ref<Record<string, number>>({})
let loadedCategoryCount = 0

const visibleCategorizedPosts = computed(() => categorizedPosts.value.slice(0, visibleGroupCount.value))

const hasLinks = computed(() => {
  const raw = settingsStore.settings.friend_links
  if (!raw) return false
  try {
    const arr = JSON.parse(raw)
    return Array.isArray(arr) && arr.length > 0
  } catch { return false }
})

// ── Badge helpers ──
function getBadgeType(post: Post) {
  const t = (post as any).article_type_name
  return t ? { key: (post as any).article_type || '', label: t } : null
}
function getBadgeStatus(post: Post) {
  const s = (post as any).article_status_name
  return s ? { key: (post as any).article_status || '', label: s } : null
}

function toggleSidebar(id: number) {
  const next = new Set(collapsedSidebar.value)
  if (next.has(id)) {
    next.delete(id)
  } else {
    next.add(id)
  }
  collapsedSidebar.value = next
}

interface SidebarCat extends Category {
  _children?: Category[]
}
const sidebarCategories = computed<SidebarCat[]>(() => {
  const all = categories.value
  const parents = all.filter(c => !c.parent_id).sort((a, b) => a.sort_order - b.sort_order)
  const children = all.filter(c => c.parent_id)
  return parents.map(p => ({
    ...p,
    _children: children.filter(ch => ch.parent_id === p.id).sort((a, b) => a.sort_order - b.sort_order),
  }))
})

function toggleGroup(category: string) {
  if (collapsedGroups.has(category)) collapsedGroups.delete(category)
  else collapsedGroups.add(category)
}

const hotPosts = computed(() => [...posts.value].sort((a, b) => (b.view_count || 0) - (a.view_count || 0)))

const categoryColors: Record<string, string> = {}
const colorPalette = ['#f59e0b', '#10b981', '#3b82f6', '#8b5cf6', '#ec4899', '#06b6d4', '#f97316', '#6366f1', '#14b8a6', '#e11d48']

const categorizedPosts = computed(() => {
  const childToParent = new Map<string, string>()
  for (const cat of categories.value) {
    if (cat.parent_id) {
      const parent = categories.value.find(c => c.id === cat.parent_id)
      if (parent) childToParent.set(cat.name, parent.name)
    }
  }
  const groups: Record<string, Post[]> = {}
  for (const post of posts.value) {
    const cat = post.category_name || '未分类'
    const displayCat = childToParent.get(cat) || cat
    if (!groups[displayCat]) groups[displayCat] = []
    groups[displayCat].push(post)
  }
  let colorIdx = 0
  return Object.entries(groups)
    .sort(([a], [b]) => new Intl.Collator('zh-Hans-CN', { sensitivity: 'base' }).compare(a, b))
    .map(([category, posts]) => {
    if (!categoryColors[category]) {
      categoryColors[category] = colorPalette[colorIdx % colorPalette.length]
      colorIdx++
    }
    const catInfo = categories.value.find(c => c.name === category)
    return { category, slug: catInfo?.slug || '', description: catInfo?.description || '', posts, total: categoryPostTotals.value[category] ?? posts.length, color: categoryColors[category] }
  })
})

function getCategoryColor(cat: string): string {
  return categoryColors[cat] || colorPalette[0]
}

async function loadSiteManagerInfo() {
  try {
    const resp = await fetchSiteManagerInfo()
    siteManager.value = resp.data.data
  } catch { siteManager.value = null }
}

async function loadData() {
  loading.value = true
  posts.value = []
  loadedCategoryCount = 0
  categoryPostTotals.value = {}
  try {
    const [catResp, tagResp] = await Promise.all([
      fetchCategories(),
      fetchTags(),
    ])
    categories.value = catResp.data.data
    categories.value.sort((a, b) => a.sort_order - b.sort_order)
    if (settingsStore.settings.sidebar_collapse === 'true') {
      const childParentIds = new Set(
        categories.value.filter(c => c.parent_id).map(c => c.parent_id)
      )
      collapsedSidebar.value = new Set(childParentIds)
    } else {
      collapsedSidebar.value = new Set()
    }
    tags.value = tagResp.data.data
    tags.value.sort((a, b) => (b.post_count || 0) - (a.post_count || 0))

    const allParentCategories = categories.value.filter(c => !c.parent_id)
    await fetchCategoryBatch(0, allParentCategories.length)
  } finally {
    loading.value = false
  }
}

async function loadPinnedPosts() {
  try {
    const resp = await fetchPinnedPosts()
    pinnedPosts.value = resp.data.data
  } catch { /* ignore */ }
}

async function fetchCategoryBatch(startIdx: number, count: number) {
  const parentCategories = categories.value.filter(c => !c.parent_id)
  const effectiveCount = Math.min(count, parentCategories.length - startIdx)
  if (effectiveCount <= 0) return

  const cats = parentCategories.slice(startIdx, startIdx + effectiveCount)
  const results = await Promise.all(
    cats.map(cat => fetchPosts({ category_id: cat.id, page_size: scrollLoadSize.value, status: 'published', include_subcategories: true }))
  )
  for (let i = 0; i < cats.length; i++) {
    const res = results[i]
    const cat = cats[i]
    categoryPostTotals.value[cat.name] = res.data?.pagination?.total ?? 0
    posts.value.push(...(res.data?.data ?? []))
  }
  loadedCategoryCount = startIdx + effectiveCount
  visibleGroupCount.value = loadedCategoryCount

  await nextTick()
  setupGroupObserver()
}

let groupObserver: IntersectionObserver | null = null

function setupGroupObserver() {
  groupObserver?.disconnect()
  const parentCategories = categories.value.filter(c => !c.parent_id)
  if (bottomSentinel.value && loadedCategoryCount < parentCategories.length) {
    groupObserver = new IntersectionObserver(([entry]) => {
      if (entry.isIntersecting) loadMoreGroups()
    }, { rootMargin: '500px' })
    groupObserver.observe(bottomSentinel.value)
  }
}

function loadMoreGroups() {
  const parentCategories = categories.value.filter(c => !c.parent_id)
  const remaining = parentCategories.length - loadedCategoryCount
  if (remaining <= 0) return
  fetchCategoryBatch(loadedCategoryCount, Math.min(scrollLoadSize.value, remaining))
}

onMounted(() => {
  loadData()
  loadPinnedPosts()
  loadSiteManagerInfo()
})

onUnmounted(() => {
  groupObserver?.disconnect()
})
</script>

<style>
/* ── Brush-stroke highlight badges ── */
.home-edge-badges {
  position: absolute;
  right: 0;
  top: 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
  z-index: 5;
  pointer-events: none;
}

.home-brush-badge {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 2px 10px;
  transform: translateX(25%) rotate(-6deg);
}
.home-brush-stroke {
  position: absolute;
  bottom: -2px;
  left: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
  pointer-events: none;
}
.home-brush-stroke path {
  fill: var(--home-brush-color);
  opacity: 0.15;
  filter: url(#brush-edge);
}
.home-brush-text {
  position: relative;
  z-index: 1;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.2px;
  white-space: nowrap;
  color: var(--color-text);
  text-shadow: 0 0 4px var(--color-bg);
}

.home-brush-type-original            { --home-brush-color: #3b82f6; }
.home-brush-type-ai_organized        { --home-brush-color: #a855f7; }
.home-brush-type-knowledge_summary   { --home-brush-color: #22c55e; }
.home-brush-type-reprint_translation { --home-brush-color: #fb923c; }
.home-brush-type-opinion_essay       { --home-brush-color: #ec4899; }
.home-brush-status-latest             { --home-brush-color: #22c55e; }
.home-brush-status-partially_outdated { --home-brush-color: #eab308; }
.home-brush-status-outdated           { --home-brush-color: #ef4444; }
.home-brush-status-continuously_updated { --home-brush-color: #3b82f6; }
.home-brush-status-classic_archive    { --home-brush-color: #8b5cf6; }
.home-brush-status-experimental       { --home-brush-color: #06b6d4; }
.home-brush-pinned                   { --home-brush-color: #f59e0b; }

.home-post-card:hover .home-brush-stroke path {
  opacity: 0.75;
  filter: url(#brush-edge) brightness(1.1);
}
</style>
