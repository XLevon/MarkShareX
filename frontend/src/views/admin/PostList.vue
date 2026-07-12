<template>
  <div class="post-list-page">
    <!-- 页面标题 + 操作 -->
    <div class="page-header">
      <h1 class="page-title">📚 知识库</h1>
      <router-link v-if="activeMainTab === 'posts'" to="/admin/posts/new" class="btn-primary">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        写文章
      </router-link>
      <button v-if="activeMainTab === 'categories'" class="btn-primary" @click="categoriesRef?.openCreateModal()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        新建分类
      </button>
      <button v-if="activeMainTab === 'tags'" class="btn-primary" @click="tagsRef?.openCreateModal()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        新建标签
      </button>
      <button v-if="activeMainTab === 'types'" class="btn-primary" @click="typesRef?.openCreate()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        新建类型
      </button>
      <button v-if="activeMainTab === 'statuses'" class="btn-primary" @click="statusesRef?.openCreate()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        新建状态
      </button>
    </div>

    <!-- 主 Tab 切换：文章管理 | 分类管理 | 标签管理 -->
    <div class="main-tabs">
      <button
        v-for="tab in mainTabs"
        :key="tab.key"
        class="main-tab"
        :class="{ active: activeMainTab === tab.key }"
        @click="switchMainTab(tab.key)"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- ====== 文章管理 Tab ====== -->
    <template v-if="activeMainTab === 'posts'">
      <!-- 筛选条件行 -->
      <div v-if="showFilters" class="filter-bar">
        <n-select
          v-model:value="selectedCategoryIds"
          :options="categoryOptions"
          multiple
          placeholder="分类"
          clearable
          size="tiny"
          max-tag-count="responsive"
          class="filter-select"
          @update:value="resetAndLoad()"
        />
        <n-select
          v-model:value="selectedTypeCodes"
          :options="typeOptions"
          multiple
          placeholder="类型"
          clearable
          size="tiny"
          max-tag-count="responsive"
          class="filter-select"
          @update:value="resetAndLoad()"
        />
        <n-select
          v-model:value="selectedStatusCodes"
          :options="statusOptions"
          multiple
          placeholder="状态"
          clearable
          size="tiny"
          max-tag-count="responsive"
          class="filter-select"
          @update:value="resetAndLoad()"
        />
        <div class="search-box mini-tag-search">
          <input
            v-model="tagSearch"
            type="text"
            placeholder="标签..."
            class="search-input compact-input"
            @keyup.enter="resetAndLoad()"
          />
        </div>
        <div class="search-box">
          <svg class="search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索标题..."
            class="search-input compact-input"
            @keyup.enter="resetAndLoad()"
          />
        </div>
      </div>

      <!-- 状态页签 -->
      <div class="toolbar">
        <div class="filter-tabs">
          <button
            v-for="tab in filterTabs"
            :key="tab.key"
            class="filter-tab"
            :class="{ active: activeTab === tab.key }"
            @click="activeTab = tab.key; resetAndLoad()"
          >
            {{ tab.label }}
            <span v-if="tab.count !== undefined" class="tab-count">{{ tab.count }}</span>
          </button>
        </div>
        <span class="result-count">{{ totalPosts }} 篇</span>
        <button class="filter-toggle" @click="showFilters = !showFilters" :title="showFilters ? '收起筛选' : '展开筛选'">
          <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>
          </svg>
          <span>筛选</span>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline v-if="showFilters" points="6 9 12 15 18 9"/>
            <polyline v-else points="6 15 12 9 18 15"/>
          </svg>
        </button>
        <button v-if="activeTab === 'draft' && isAdmin && posts.length > 0"
          class="btn-danger btn-sm" style="margin-left: 8px" @click="batchDeleteDrafts">
          批量删除草稿
        </button>
      </div>

      <!-- 卡片列表 -->
      <div v-if="loading" class="loading-state">
        <div class="spinner"></div>
        <span>加载中...</span>
      </div>

      <div v-else-if="posts.length === 0" class="empty-state">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>
        <p>暂无文章</p>
        <router-link to="/admin/posts/new" class="btn-primary">写第一篇文章</router-link>
      </div>

      <div v-else class="post-cards">
        <div :id="`p${post.id}`" v-for="post in posts" :key="post.id" class="mb-3"
          :draggable="activeTab === 'pinned'"
          :class="{ 'drag-ghost': dragId === post.id }"
          @dragstart="activeTab === 'pinned' && onDragStart($event, post)"
          @dragover.prevent="activeTab === 'pinned' && onDragOver($event, post)"
          @drop="activeTab === 'pinned' && onDrop($event, post)"
          @dragend="dragId = null"
        >
          <div v-if="activeTab === 'pinned'" class="drag-handle" title="拖拽排序">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="9" cy="5" r="1"/><circle cx="15" cy="5" r="1"/><circle cx="9" cy="12" r="1"/><circle cx="15" cy="12" r="1"/><circle cx="9" cy="19" r="1"/><circle cx="15" cy="19" r="1"/></svg>
          </div>
          <PostCard
          :post="post"
          mode="admin"
          :max-tags="3"
          @preview="openPreview"
        >
          <template #actions="{ post: p }">
            <button class="action-btn edit" @click.stop="router.push(`/admin/posts/${p.id}`)" title="编辑文章">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>
            </button>
            <button class="action-btn quick-edit" @click.stop="openQuickEdit(p)" title="快速编辑">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/><line x1="1" y1="14" x2="7" y2="14"/><line x1="9" y1="8" x2="15" y2="8"/><line x1="17" y1="16" x2="23" y2="16"/></svg>
            </button>
            <button class="action-btn settings" @click.stop="openSettings(p)" title="文章设置">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>
            </button>
            <button v-if="activeTab === 'draft'" class="action-btn publish" @click.stop="publishDraft(p)" title="发布">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="22" y1="2" x2="11" y2="13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/></svg>
            </button>
            <button class="action-btn pin" v-if="showPinButtons" @click.stop="handlePin(p)"
              :title="p.is_pinned ? '取消置顶' : '置顶'"
              :style="p.is_pinned ? { color: 'var(--color-primary)' } : {}">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="17" x2="12" y2="22"/><path d="M5 17h14v-1.76a2 2 0 0 0-1.11-1.79l-1.78-.9A2 2 0 0 1 15 10.76V6h1a2 2 0 0 0 0-4H8a2 2 0 0 0 0 4h1v4.76a2 2 0 0 1-1.11 1.79l-1.78.9A2 2 0 0 0 5 15.24Z"/></svg>
            </button>
            <button v-if="isPrivileged" class="action-btn change-author" @click.stop="openChangeAuthor(p)" title="更换作者">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="8.5" cy="7" r="4"/><polyline points="17 11 19 13 23 9"/></svg>
            </button>
            <button v-if="isPrivileged || p.status !== 'published'" class="action-btn delete" @click.stop="confirmDelete(p)" title="删除">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
            </button>
          </template>
        </PostCard>
      </div>
    </div>

      <!-- 滚动加载 -->
      <div v-if="hasMore" ref="loadMoreRef" class="load-more">
        <div v-if="loadingMore" class="spinner"></div>
        <span v-else class="load-more-text">滚动加载更多...</span>
      </div>
      <div v-else-if="posts.length > batchSize" class="load-more-done">
        — 我是有底线的 —
      </div>

      <!-- 删除确认 -->
      <div v-if="deleteTarget" class="modal-overlay" @click.self="deleteTarget = null">
        <div class="modal-box">
          <h3>确认删除</h3>
          <p>确定要删除文章「{{ deleteTarget.title }}」吗？此操作不可恢复。</p>
          <div class="modal-actions">
            <button class="btn-secondary" @click="deleteTarget = null">取消</button>
            <button class="btn-danger" @click="handleDelete">删除</button>
          </div>
        </div>
      </div>

      <!-- 文章设置弹窗 -->
      <PostSettingsModal
        :visible="settingsVisible"
        :cover-image="settingsPost?.cover_image || ''"
        :summary="settingsPost?.summary || ''"
        :tags="settingsPost?.tags?.map((t: any) => typeof t === 'string' ? t : t.name) || []"
        :article-type="(settingsPost as any)?.article_type || ''"
        :article-status="(settingsPost as any)?.article_status || ''"
        @confirm="onSettingsConfirm"
        @cancel="settingsVisible = false"
      />

      <!-- 更换作者弹窗 -->
      <div v-if="changeAuthorVisible" class="modal-overlay" @click.self="changeAuthorVisible = false">
        <div class="modal-box" style="max-width: 440px;">
          <h3>更换作者</h3>
          <div class="modal-info">
            <div class="modal-info-row">
              <span class="modal-info-label">文章</span>
              <span class="modal-info-value">{{ changeAuthorTarget?.title }}</span>
            </div>
            <div class="modal-info-row">
              <span class="modal-info-label">当前作者</span>
              <span class="modal-info-value">{{ changeAuthorTarget?.author_name || changeAuthorTarget?.author || '无' }}</span>
            </div>
          </div>
          <n-select
            v-model:value="selectedAuthorId"
            :options="authorOptions"
            placeholder="选择新作者"
            filterable
            clearable
            style="width: 100%;"
          />
          <div class="modal-actions" style="margin-top: 16px;">
            <button class="btn-secondary" @click="changeAuthorVisible = false">取消</button>
            <button class="btn-primary" :disabled="!selectedAuthorId" @click="handleChangeAuthor">确认更换</button>
          </div>
        </div>
      </div>

      <!-- 快速编辑弹窗 -->
      <div v-if="quickEditVisible" class="modal-overlay">
        <div class="modal-box" style="max-width: 440px;">
          <h3>快速编辑</h3>
            <div>
              <label class="modal-field-label">标题</label>
              <textarea
                v-model="quickEditTitle"
                class="modal-text-input"
                placeholder="输入新标题"
                rows="2"
                style="width: 100%; resize: vertical;"
              ></textarea>
            </div>
            <div>
              <label class="modal-field-label">分类</label>
              <n-select
                v-model:value="quickEditCategoryId"
                :options="categoryTreeOptions"
                placeholder="选择分类"
                clearable
                filterable
                :consistent-menu-width="false"
                style="width: 100%;"
              />
            </div>
          <div class="modal-actions" style="margin-top: 16px;">
            <button class="btn-secondary" @click="quickEditVisible = false">取消</button>
            <button class="btn-primary" @click="saveQuickEdit">保存</button>
          </div>
        </div>
      </div>

      <!-- 预览弹窗 -->
      <div v-if="previewPost" class="preview-overlay" ref="previewPaneRef" @click.self="closePreview" @click="handlePreviewClick">
        <div class="preview-container">
          <div class="preview-header">
            <h2>{{ previewPost.title }}</h2>
            <div class="preview-meta">
              <span v-if="previewPost.category_name" class="preview-category">{{ previewPost.category_name }}</span>
              <span>{{ previewPost.published_at ? dayjs(previewPost.published_at).format('YYYY-MM-DD') : '' }}</span>
              <span>{{ previewPost.view_count || 0 }} 次阅读</span>
            </div>
            <button class="preview-close" @click="closePreview" title="关闭">✕</button>
          </div>
          <div class="preview-body">
            <img v-if="previewPost.cover_image" :src="previewPost.cover_image" class="preview-cover" />
            <div class="preview-content markdown-body" v-html="previewHtml"></div>
          </div>
        </div>
      </div>
    </template>

    <!-- ====== 分类管理 Tab ====== -->
    <CategoriesTab ref="categoriesRef" v-if="activeMainTab === 'categories'" />

    <!-- ====== 标签管理 Tab ====== -->
    <TagsTab ref="tagsRef" v-if="activeMainTab === 'tags'" />

    <!-- ====== 类型管理 Tab ====== -->
    <KnowledgeBaseTab ref="typesRef" v-if="activeMainTab === 'types'" mode="types" />

    <!-- ====== 状态管理 Tab ====== -->
    <KnowledgeBaseTab ref="statusesRef" v-if="activeMainTab === 'statuses'" mode="statuses" />
  </div>
</template>

<script setup lang="ts">
defineOptions({ name: 'PostList' })
import { ref, computed, onMounted, onActivated, watch, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { fetchAdminPosts, deletePost, pinPost, unpinPost, updatePinOrder, updatePost, fetchPost } from '@/api/posts'
import { fetchUsers } from '@/api/admin'
import type { Post } from '@/api/index'
import api from '@/api'
import { marked } from 'marked'
import dayjs from 'dayjs'
import { NSelect } from 'naive-ui'
import { fetchAdminCategories } from '@/api/categories'
import { fetchArticleTypes, fetchArticleStatuses } from '@/api/admin'
import type { Category } from '@/api/index'
import type { ArticleType, ArticleStatus } from '@/api/admin'
import PostCard from '@/components/shared/PostCard.vue'
import PostSettingsModal from '@/components/shared/PostSettingsModal.vue'
import CategoriesTab from './Categories.vue'
import TagsTab from './Tags.vue'
import KnowledgeBaseTab from './KnowledgeBase.vue'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'

const authStore = useAuthStore()
const settingsStore = useSettingsStore()
const isPrivileged = computed(() => {
  const role = authStore.user?.role
  return role === 'admin' || role === 'sub_admin'
})
const isAdmin = computed(() => authStore.user?.role === 'admin')

const categoriesRef = ref<InstanceType<typeof CategoriesTab> | null>(null)
const tagsRef = ref<InstanceType<typeof TagsTab> | null>(null)
const typesRef = ref<InstanceType<typeof KnowledgeBaseTab> | null>(null)
const statusesRef = ref<InstanceType<typeof KnowledgeBaseTab> | null>(null)

const router = useRouter()
const route = useRoute()
const loading = ref(false)
const posts = ref<Post[]>([])
const activeTab = ref((route.query.status as string) || 'all')
const searchQuery = ref('')

// ── 多选筛选器 ──
const selectedCategoryIds = ref<number[]>([])
const selectedTypeCodes = ref<string[]>([])
const selectedStatusCodes = ref<string[]>([])
const tagSearch = ref('')
const showFilters = ref(false)

// 可选选项
const categoryOptions = ref<{ label: string; value: number }[]>([])
const categories = ref<Category[]>([])  // 原始分类数据，用于树形选择器
const typeOptions = ref<{ label: string; value: string }[]>([])
const statusOptions = ref<{ label: string; value: string }[]>([])

const categoryTreeOptions = computed(() => {
  const opts: { label: string; value: number | null }[] = []
  const cats = categories.value || []
  const topLevel = cats.filter(c => !c.parent_id)
  const children = cats.filter(c => c.parent_id)

  topLevel.sort((a, b) => a.sort_order - b.sort_order).forEach(c => {
    opts.push({ label: c.name, value: c.id })
    children.filter(ch => ch.parent_id === c.id)
      .sort((a, b) => a.sort_order - b.sort_order)
      .forEach(ch => {
        opts.push({ label: `  ⤷ ${ch.name}`, value: ch.id })
      })
  })
  // 孤儿子分类
  const seenIds = new Set(opts.map(o => o.value))
  children.filter(ch => !seenIds.has(ch.id)).forEach(ch => {
    opts.push({ label: `  ⤷ ${ch.name}`, value: ch.id })
  })
  return opts
})

async function loadFilterOptions() {
  try {
    const [catRes, typeRes, statusRes] = await Promise.all([
      fetchAdminCategories(),
      fetchArticleTypes(),
      fetchArticleStatuses(),
    ])
    const rawCategories: Category[] = (catRes as any)?.data?.data || (catRes as any)?.data || []
    categories.value = rawCategories
    categoryOptions.value = rawCategories.map((c: Category) => ({
      label: c.name,
      value: c.id,
    })).sort((a, b) => a.label.localeCompare(b.label, 'zh'))
    typeOptions.value = (((typeRes as any)?.data?.data || (typeRes as any)?.data) || []).map((t: ArticleType) => ({
      label: t.display_name,
      value: t.code,
    }))
    statusOptions.value = (((statusRes as any)?.data?.data || (statusRes as any)?.data) || []).map((s: ArticleStatus) => ({
      label: s.display_name,
      value: s.code,
    }))
  } catch { /* use empty options */ }
}

const currentPage = ref(1)
const totalPosts = ref(0)

const batchSize = computed(() => Number(settingsStore.settings.batch_load_size) || 5)
const scrollSize = computed(() => Number(settingsStore.settings.scroll_load_size) || 3)
const hasMore = ref(false)
const loadingMore = ref(false)
const loadMoreRef = ref<HTMLElement | null>(null)
const deleteTarget = ref<Post | null>(null)
const previewPost = ref<Post | null>(null)
const previewLoading = ref(false)

// 跟 PostEdit 一样：marked 渲染时生成标题 id，支持锚点跳转
const headingRenderer = new marked.Renderer()
headingRenderer.heading = function ({ tokens, depth }: { tokens: any[]; depth: number }) {
  const text = this.parser.parseInline(tokens)
  const id = text
    .replace(/<[^>]*>/g, '')
    .toLowerCase()
    .replace(/\s+/g, '-')
    .replace(/[^\w\u4e00-\u9fff-]/g, '')
    .replace(/-+/g, '-')
    .replace(/^-+|-+$/g, '')
  return `<h${depth} id="${id}">${text}</h${depth}>\n`
}

const previewHtml = computed(() => {
  if (!previewPost.value) return ''
  // 用 marked 客户端渲染（带 heading ID），不用后端 content_html（comrak 不生成 ID）
  return previewPost.value.content ? marked.parse(previewPost.value.content, { renderer: headingRenderer }) : previewPost.value.content_html || ''
})

// ── 文章设置弹窗 ──
const settingsVisible = ref(false)
const settingsPost = ref<Post | null>(null)
// ── 更换作者 ──
const changeAuthorVisible = ref(false)
const changeAuthorTarget = ref<Post | null>(null)
const selectedAuthorId = ref<number | null>(null)
const authorOptions = ref<{ label: string; value: number }[]>([])

function openChangeAuthor(p: Post) {
  changeAuthorTarget.value = p
  selectedAuthorId.value = null
  changeAuthorVisible.value = true
  loadAuthorOptions()
}

async function loadAuthorOptions() {
  try {
    const resp = await fetchUsers({ page_size: 100 })
    const users = (resp.data as any)?.data?.data || (resp.data as any)?.data || []
    authorOptions.value = users
      .filter((u: any) => u.status === 'active')
      .map((u: any) => ({ label: u.display_name || u.username, value: u.id }))
      .sort((a: any, b: any) => a.label.localeCompare(b.label, 'zh'))
  } catch { /* ignore */ }
}

async function handleChangeAuthor() {
  if (!changeAuthorTarget.value || !selectedAuthorId.value) return
  try {
    await updatePost(changeAuthorTarget.value.id, { author_id: selectedAuthorId.value } as any)
    changeAuthorVisible.value = false
    changeAuthorTarget.value = null
    resetAndLoad()
  } catch (e: any) {
    alert(e?.response?.data?.error || '更换失败')
  }
}

// ── 快速编辑（标题 + 分类）──
const quickEditVisible = ref(false)
const quickEditTarget = ref<Post | null>(null)
const quickEditTitle = ref('')
const quickEditCategoryId = ref<number | null>(null)

function openQuickEdit(p: Post) {
  quickEditTarget.value = p
  quickEditTitle.value = p.title
  quickEditCategoryId.value = p.category_id
  quickEditVisible.value = true
}

async function saveQuickEdit() {
  if (!quickEditTarget.value) return
  try {
    const payload: Record<string, any> = {}
    if (quickEditTitle.value !== quickEditTarget.value.title) {
      payload.title = quickEditTitle.value
    }
    if (quickEditCategoryId.value !== quickEditTarget.value.category_id) {
      payload.category_id = quickEditCategoryId.value
    }
    if (Object.keys(payload).length === 0) {
      quickEditVisible.value = false
      return
    }
    await updatePost(quickEditTarget.value.id, payload as any)
    quickEditVisible.value = false
    quickEditTarget.value = null
    resetAndLoad()
  } catch (e: any) {
    alert(e?.response?.data?.error || '保存失败')
  }
}

const savingSettings = ref(false)

function openSettings(p: Post) {
  settingsPost.value = p
  settingsVisible.value = true
}

async function onSettingsConfirm(values: { coverImage: string; summary: string; tags: string[]; articleType: string; articleStatus: string }) {
  if (!settingsPost.value) return
  savingSettings.value = true
  try {
    await updatePost(settingsPost.value.id, {
      cover_image: values.coverImage,
      summary: values.summary,
      tags: values.tags as any,
      article_type: values.articleType || undefined,
      article_status: values.articleStatus || undefined,
    } as any)
    settingsVisible.value = false
    settingsPost.value = null
    resetAndLoad()
    loadCounts()
  } catch (e: any) {
    alert(e?.response?.data?.error || '保存失败')
  } finally {
    savingSettings.value = false
  }
}

async function publishDraft(p: Post) {
  try {
    await updatePost(p.id, { status: 'published' } as any)
    resetAndLoad()
    loadCounts()
  } catch { /* ignore */ }
}

async function openPreview(p: Post) {
  previewPost.value = p
  previewLoading.value = true
  // 禁用浏览器自动恢复滚动，由我们手动控制
  if ('scrollRestoration' in history) history.scrollRestoration = 'manual'
  try {
    const { data: resp } = await fetchPost(p.id)
    previewPost.value = { ...p, ...resp.data }
  } catch { /* keep list data */ }
  finally { previewLoading.value = false }
}

// ── 预览区内锚点跳转 ──
const previewPaneRef = ref<HTMLElement | null>(null)

function handlePreviewClick(e: MouseEvent) {
  const target = e.target as HTMLElement
  const anchor = target.closest('a[href^="#"]') as HTMLAnchorElement | null
  if (!anchor) return
  const href = anchor.getAttribute('href')
  if (!href || href === '#') return
  const id = href.slice(1)
  const container = previewPaneRef.value
  if (!container) return
  const heading = container.querySelector(`#${CSS.escape(id)}`) as HTMLElement | null
  if (!heading) return
  e.preventDefault()
  const offset = heading.getBoundingClientRect().top - container.getBoundingClientRect().top + container.scrollTop - 16
  container.scrollTo({ top: offset, behavior: 'smooth' })
}

function closePreview() {
  previewPost.value = null
  if ('scrollRestoration' in history) history.scrollRestoration = 'auto'
}

// ── 主 Tab（文章管理 / 分类管理 / 标签管理 / 类型管理 / 状态管理）──
const mainTabs = [
  { label: '文章管理', key: 'posts' },
  { label: '分类管理', key: 'categories' },
  { label: '标签管理', key: 'tags' },
  { label: '类型管理', key: 'types' },
  { label: '状态管理', key: 'statuses' },
]
const activeMainTab = ref((route.query.mtab as string) || 'posts')

function switchMainTab(key: string) {
  activeMainTab.value = key
  // 同步到 URL query（不触发导航，只更新地址栏）
  const query = { ...route.query, mtab: key }
  router.replace({ query })
}

// Sync main tab from URL（支持浏览器前进/后退）
watch(() => route.query.mtab, (val) => {
  if (val && ['posts', 'categories', 'tags', 'types', 'statuses'].includes(val as string)) {
    activeMainTab.value = val as string
  } else {
    activeMainTab.value = 'posts'
  }
})

// Counts for tabs
const tabCounts = ref({ all: 0, published: 0, draft: 0, pinned: 0 })

const filterTabs = [
  { label: '全部', key: 'all', count: undefined },
  { label: '已发布', key: 'published', count: undefined },
  { label: '草稿', key: 'draft', count: undefined },
  { label: '置顶', key: 'pinned', count: undefined },
]

const showPinButtons = computed(() => activeTab.value !== 'draft')

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD HH:mm')
}

async function loadMore() {
  if (loadingMore.value || !hasMore.value) return
  loadingMore.value = true
  try {
    currentPage.value++
    const params: Record<string, any> = { page: currentPage.value, page_size: batchSize.value }
    if (activeTab.value === 'pinned') {
      params.is_pinned = true
    } else if (activeTab.value !== 'all') {
      params.status = activeTab.value
    }
    if (searchQuery.value.trim()) {
      params.search = searchQuery.value.trim()
    }
    if (tagSearch.value.trim()) {
      params.tag_search = tagSearch.value.trim()
    }
    if (selectedCategoryIds.value.length) {
      params.category_ids = selectedCategoryIds.value.join(',')
    }
    if (selectedTypeCodes.value.length) {
      params.article_types = selectedTypeCodes.value.join(',')
    }
    if (selectedStatusCodes.value.length) {
      params.article_statuses = selectedStatusCodes.value.join(',')
    }
    const { data: resp } = await fetchAdminPosts(params)
    posts.value.push(...resp.data)
    hasMore.value = posts.value.length < resp.pagination.total
    totalPosts.value = resp.pagination.total
  } catch { /* ignore */ }
  finally {
    loadingMore.value = false
    nextTick(() => setupScrollObserver())
  }
}

async function resetAndLoad() {
  loading.value = true
  currentPage.value = 1
  try {
    const params: Record<string, any> = { page: 1, page_size: batchSize.value }
    if (activeTab.value === 'pinned') {
      params.is_pinned = true
    } else if (activeTab.value !== 'all') {
      params.status = activeTab.value
    }
    if (searchQuery.value.trim()) {
      params.search = searchQuery.value.trim()
    }
    if (tagSearch.value.trim()) {
      params.tag_search = tagSearch.value.trim()
    }
    if (selectedCategoryIds.value.length) {
      params.category_ids = selectedCategoryIds.value.join(',')
    }
    if (selectedTypeCodes.value.length) {
      params.article_types = selectedTypeCodes.value.join(',')
    }
    if (selectedStatusCodes.value.length) {
      params.article_statuses = selectedStatusCodes.value.join(',')
    }
    const { data: resp } = await fetchAdminPosts(params)
    posts.value = resp.data
    totalPosts.value = resp.pagination.total
    hasMore.value = posts.value.length < resp.pagination.total
  } catch {
    posts.value = []
  } finally {
    loading.value = false
    setupScrollObserver()
  }
}

async function loadCounts() {
  try {
    const [allRes, pubRes, draftRes, pinnedRes] = await Promise.all([
      fetchAdminPosts({ page: 1, page_size: 1 }),
      fetchAdminPosts({ page: 1, page_size: 1, status: 'published' }),
      fetchAdminPosts({ page: 1, page_size: 1, status: 'draft' }),
      fetchAdminPosts({ page: 1, page_size: 1, is_pinned: true }),
    ])
    tabCounts.value.all = allRes.data.pagination.total
    tabCounts.value.published = pubRes.data.pagination.total
    tabCounts.value.draft = draftRes.data.pagination.total
    tabCounts.value.pinned = pinnedRes.data.pagination.total
  } catch { /* ignore */ }
}

function confirmDelete(post: Post) {
  deleteTarget.value = post
}

async function handlePin(post: Post) {
  try {
    if (post.is_pinned) {
      await unpinPost(post.id)
    } else {
      await pinPost(post.id)
    }
    resetAndLoad()
    loadCounts()
  } catch { /* ignore */ }
}

async function handleDelete() {
  if (!deleteTarget.value) return
  try {
    await deletePost(deleteTarget.value.id)
    deleteTarget.value = null
    resetAndLoad()
    loadCounts()
  } catch { /* ignore */ }
}

async function batchDeleteDrafts() {
  if (!posts.value.length) return
  if (!confirm(`确定删除当前列表中已加载的 ${posts.value.length} 篇草稿吗？此操作不可恢复。`)) return
  const ids = posts.value.map(p => p.id)
  try {
    await api.post('/admin/posts/batch-delete', { ids })
    resetAndLoad()
    loadCounts()
  } catch (e: any) {
    alert(e?.response?.data?.error || '批量删除失败')
  }
}

// ── 拖拽排序 ──
const dragId = ref<number | null>(null)

function onDragStart(e: DragEvent, post: Post) {
  dragId.value = post.id
  e.dataTransfer!.effectAllowed = 'move'
  e.dataTransfer!.setData('text/plain', String(post.id))
}

function onDragOver(e: DragEvent, _post: Post) {
  e.dataTransfer!.dropEffect = 'move'
}

async function onDrop(e: DragEvent, targetPost: Post) {
  e.preventDefault()
  const draggedId = dragId.value
  if (!draggedId || draggedId === targetPost.id) return

  const newPosts = [...posts.value]
  const draggedIdx = newPosts.findIndex(p => p.id === draggedId)
  const targetIdx = newPosts.findIndex(p => p.id === targetPost.id)
  if (draggedIdx === -1 || targetIdx === -1) return

  // Reorder
  const [moved] = newPosts.splice(draggedIdx, 1)
  newPosts.splice(targetIdx, 0, moved)
  posts.value = newPosts
  dragId.value = null

  // Save to backend
  try {
    await updatePinOrder(newPosts.map(p => p.id))
  } catch { resetAndLoad() }
}

onMounted(() => {
  loadFilterOptions()
  // 从分类管理跳转过来时（/admin/posts?category_id=X），预填分类筛选
  const cid = route.query.category_id
  if (cid) {
    selectedCategoryIds.value = [Number(cid)]
  }
  resetAndLoad()
  loadCounts()
  scrollToTarget()
})

// keep-alive 缓存后切回来时刷新数据
onActivated(() => {
  loadFilterOptions()
  const cid = route.query.category_id
  if (cid) {
    selectedCategoryIds.value = [Number(cid)]
  }
  resetAndLoad()
  loadCounts()
  scrollToTarget()
})

let scrollObserver: IntersectionObserver | null = null

function setupScrollObserver() {
  scrollObserver?.disconnect()
  // Wait for DOM to render loadMoreRef
  nextTick(() => {
    if (loadMoreRef.value) {
      scrollObserver = new IntersectionObserver(([entry]) => {
        if (entry.isIntersecting && hasMore.value && !loadingMore.value) {
          loadMore()
        }
      }, { rootMargin: '100px' })
      scrollObserver.observe(loadMoreRef.value)
    }
  })
}

function scrollToTarget() {
  const hash = route.hash
  if (hash && hash.startsWith('#p')) {
    nextTick(() => {
      const el = document.getElementById(hash.slice(1))
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' })
        el.classList.add('post-highlight')
        setTimeout(() => el.classList.remove('post-highlight'), 2000)
      }
    })
  }
}

// Sync with route query param (for dashboard links)
watch(() => route.query.status, (newVal) => {
  activeTab.value = (newVal as string) || 'all'
  resetAndLoad()
})
</script>

<style scoped>
.post-list-page {
  animation: fadeIn 0.3s ease;
}
@keyframes fadeIn {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

/* ===== Header ===== */
.page-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
}
.page-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--input-color);
}
.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 10px 20px;
  background: #4f46e5;
  color: #fff;
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  text-decoration: none;
  border: none;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-primary:hover { background: #4338ca; }

/* ===== Toolbar ===== */
/* ===== Main Tabs (文章/分类/标签) ===== */
.main-tabs {
  display: flex;
  gap: 4px;
  background: var(--card-bg);
  border-radius: 10px;
  padding: 4px;
  margin-bottom: 20px;
  width: fit-content;
}
.main-tab {
  padding: 9px 20px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
}
.main-tab:hover { color: var(--text-primary); }
.main-tab.active {
  background: rgba(79, 70, 229, 0.15);
  color: #818cf8;
}

/* ===== 筛选条件行 ===== */
.filter-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  flex-wrap: nowrap;
}
.filter-bar > * {
  flex: 1;
  min-width: 0;
}
.filter-select {
}
.filter-select :deep(.n-base-selection) {
  font-size: 12px;
}
.filter-select :deep(.n-base-selection .n-tag) {
  font-size: 11px;
  padding: 0 4px;
  height: 18px;
  line-height: 18px;
  max-width: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.mini-tag-search {
}
.compact-input {
  font-size: 12px !important;
  padding: 5px 8px !important;
}

/* ===== Toolbar（状态页签行） ===== */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
  flex-wrap: wrap;
  gap: 12px;
}
.filter-tabs {
  display: flex;
  gap: 4px;
  background: var(--card-bg);
  border-radius: 10px;
  padding: 4px;
}
.filter-tab {
  padding: 8px 16px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s;
  display: flex;
  align-items: center;
  gap: 6px;
}
.filter-tab:hover { color: var(--text-primary); }
.filter-tab.active {
  background: rgba(79, 70, 229, 0.15);
  color: #818cf8;
}
.tab-count {
  font-size: 11px;
  padding: 1px 7px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-dim);
}
.filter-tab.active .tab-count {
  background: rgba(79, 70, 229, 0.2);
  color: #a5b4fc;
}
.result-count {
  font-size: 12px;
  color: var(--text-dim);
  margin-left: auto;
  white-space: nowrap;
}
.filter-toggle {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: var(--card-bg);
  border: 1px solid var(--card-border-color);
  color: var(--text-secondary);
  cursor: pointer;
  padding: 5px 10px;
  border-radius: 8px;
  font-size: 12px;
  transition: all 0.15s;
}
.filter-toggle:hover {
  color: var(--text-primary);
  border-color: rgba(79, 70, 229, 0.3);
  background: rgba(79, 70, 229, 0.06);
}
.search-icon {
  color: var(--text-dim);
  flex-shrink: 0;
}
.search-box {
  display: flex;
  align-items: center;
  background: var(--card-bg);
  border: 1px solid var(--card-border-color);
  border-radius: 10px;
  padding: 0 14px;
  transition: border-color 0.2s;
}
.search-box:focus-within {
  border-color: rgba(79, 70, 229, 0.4);
}
.search-icon {
  color: var(--text-dim);
  flex-shrink: 0;
}
.search-input {
  background: none;
  border: none;
  color: var(--text-primary);
  padding: 9px 10px;
  font-size: 13px;
  outline: none;
  width: 400px;
}
.search-input::placeholder { color: var(--text-dim); }

/* ===== 卡片网格 ===== */
/* ===== Loading ===== */
.loading-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 60px 0;
  color: var(--text-dim);
  font-size: 14px;
}
.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(79, 70, 229, 0.15);
  border-top-color: #4f46e5;
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }

/* ===== Empty ===== */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 80px 0;
  color: #4b5563;
}
.empty-state p { font-size: 15px; color: var(--text-dim); }

/* ===== Post Cards container ===== */
.post-cards {
  display: flex;
  flex-direction: column;
}

/* Button styles used in card actions slot */
.action-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: 1px solid var(--card-border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.15s;
}
.action-btn.edit {
  background: rgba(79, 70, 229, 0.12);
  color: #818cf8;
}
.action-btn.edit:hover { background: rgba(79, 70, 229, 0.25); }
.action-btn.settings {
  background: rgba(20, 184, 166, 0.12);
  color: #2dd4bf;
}
.action-btn.settings:hover { background: rgba(20, 184, 166, 0.25); }
.action-btn.publish {
  background: rgba(16, 185, 129, 0.12);
  color: #34d399;
}
.action-btn.publish:hover { background: rgba(16, 185, 129, 0.25); }
.action-btn.delete {
  background: rgba(239, 68, 68, 0.1);
  color: #f87171;
}
.action-btn.delete:hover { background: rgba(239, 68, 68, 0.2); }

/* ===== Pagination ===== */
.pagination {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-top: 24px;
}
.pagination button {
  padding: 8px 16px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  background: var(--card-bg);
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.15s;
}
.pagination button:hover:not(:disabled) {
  border-color: rgba(79, 70, 229, 0.3);
  color: var(--text-primary);
}
.pagination button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}
.pagination span {
  font-size: 13px;
  color: var(--text-dim);
}

/* ===== Modal ===== */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 300;
}
.modal-box {
  background: var(--modal-bg);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  padding: 28px;
  max-width: 400px;
  width: 90%;
}
.modal-box h3 {
  margin: 0 0 12px;
  font-size: 17px;
  color: var(--input-color);
}
.modal-box p {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.6;
  margin: 0 0 20px;
}
.modal-info {
  margin-bottom: 16px;
  padding: 12px 14px;
  border-radius: 8px;
  background: var(--color-bg-secondary);
  border: 1px solid var(--color-border-light, var(--color-border));
}
.modal-info-row {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 13px;
  line-height: 1.6;
}
.modal-info-row + .modal-info-row {
  margin-top: 4px;
}
.modal-info-label {
  color: var(--color-text-muted);
  flex-shrink: 0;
  width: 56px;
}
.modal-info-value {
  color: var(--color-text);
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
.modal-field-label {
  display: block;
  font-size: 13px;
  color: var(--color-text-muted);
  margin-bottom: 6px;
  font-weight: 500;
}
.modal-text-input {
  padding: 8px 12px;
  border-radius: 8px;
  border: 1px solid var(--color-border);
  background: var(--color-bg);
  color: var(--color-text);
  font-size: 14px;
  outline: none;
  transition: border-color 0.15s;
  box-sizing: border-box;
}
.modal-text-input:focus {
  border-color: var(--color-primary);
}
.btn-secondary {
  padding: 8px 18px;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  background: transparent;
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
}
.btn-secondary:hover { background: rgba(255, 255, 255, 0.04); }
.btn-danger {
  padding: 8px 18px;
  border-radius: 8px;
  border: none;
  background: #dc2626;
  color: #fff;
  font-size: 13px;
  cursor: pointer;
}
.btn-danger:hover { background: #b91c1c; }

/* ===== 预览弹窗 ===== */
.preview-overlay {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.85);
  display: flex;
  justify-content: center;
  align-items: flex-start;
  overflow-y: auto;
  padding: 0;
}
.preview-container {
  width: 100%;
  min-height: 100vh;
  background: var(--color-bg);
  padding: 0;
}
.preview-header {
  position: sticky;
  top: 0;
  z-index: 10;
  background: var(--color-bg);
  border-bottom: 1px solid var(--color-border);
  padding: 20px 32px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 860px;
  margin: 0 auto;
  width: 100%;
}
.preview-header h2 {
  font-size: 22px;
  font-weight: 700;
  color: var(--color-text);
  margin: 0;
  padding-right: 40px;
}
.preview-meta {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: var(--color-text-muted);
}
.preview-category {
  color: #f59e0b;
  font-weight: 500;
}
.preview-close {
  position: absolute;
  top: 16px;
  right: 20px;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: 1px solid var(--color-border);
  background: var(--color-bg-card);
  color: var(--color-text-secondary);
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}
.preview-close:hover {
  background: #ef4444;
  color: #fff;
  border-color: #ef4444;
}
.preview-body {
  padding: 24px 32px 80px;
  max-width: 860px;
  margin: 0 auto;
}
.preview-cover {
  width: 100%;
  max-height: 400px;
  object-fit: cover;
  border-radius: 10px;
  margin-bottom: 32px;
}
.preview-content {
  font-size: 15px;
  line-height: 1.8;
  color: var(--color-text);
}
.preview-content h1 { font-size: 24px; margin: 1.5em 0 0.5em; }
.preview-content h2 { font-size: 20px; margin: 1.5em 0 0.5em; padding-bottom: 6px; border-bottom: 1px solid var(--color-border); }
.preview-content h3 { font-size: 17px; margin: 1.2em 0 0.4em; }
.preview-content p { margin: 0.6em 0; }
.preview-content img { max-width: 100%; border-radius: 6px; margin: 1em 0; }
.preview-content pre {
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  padding: 16px;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.5;
  margin: 1em 0;
}
.preview-content code { font-family: 'JetBrains Mono', monospace; font-size: 0.9em; }
.preview-content pre code { background: none; padding: 0; }
.preview-content blockquote {
  border-left: 3px solid var(--color-primary);
  padding-left: 16px;
  color: var(--color-text-secondary);
  margin: 1em 0;
}
.preview-content table {
  border-collapse: collapse;
  width: 100%;
  margin: 1em 0;
}
.preview-content th, .preview-content td {
  border: 1px solid var(--color-border);
  padding: 8px 12px;
  text-align: left;
}
.preview-content th { background: var(--color-bg-card); font-weight: 600; }

/* ===== Responsive ===== */
@media (max-width: 768px) {
  .post-card { flex-direction: column; }
  .card-cover { width: 100%; height: 160px; }
  .toolbar { flex-direction: column; align-items: stretch; }
  .search-input { width: 100%; }
}

/* ===== 拖拽排序 ===== */
.drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  cursor: grab;
  color: var(--color-text-muted);
  opacity: 0.6;
  transition: opacity 0.15s;
  flex-shrink: 0;
  margin-right: -4px;
}
.drag-handle:hover { opacity: 1; color: var(--color-primary); }
.drag-handle:active { cursor: grabbing; }
.drag-ghost { opacity: 0.4; }

/* ===== 滚动加载 ===== */
.load-more {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px 0;
  min-height: 48px;
}
.load-more-text {
  color: var(--color-text-muted);
  font-size: 13px;
}
.load-more-done {
  text-align: center;
  padding: 20px 0;
  color: var(--color-text-muted);
  font-size: 12px;
}

/* ===== 移动端适配 ===== */
@media (max-width: 640px) {
  .page-title { font-size: 22px; }
  .filter-bar {
    flex-wrap: wrap;
  }
  .filter-bar > * {
    min-width: 100%;
  }
  .toolbar {
    flex-wrap: wrap;
    gap: 8px;
  }
  .filter-tabs {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
    flex-shrink: 0;
  }
  .filter-tab {
    white-space: nowrap;
    font-size: 12px;
    padding: 4px 10px;
  }
  .main-tabs {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }
  .main-tab {
    white-space: nowrap;
    font-size: 13px;
    padding: 6px 14px;
  }
}

</style>