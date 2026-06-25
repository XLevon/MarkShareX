<template>
  <div class="post-detail-wrapper max-w-7xl mx-auto px-6">
    <!-- TOC Sidebar -->
    <aside ref="tocRef" class="toc-sidebar hidden lg:block">
      <nav class="toc-nav">
        <div class="toc-title">目录</div>
        <div v-if="tocItems.length === 0" class="toc-empty">暂无目录</div>
        <ul v-else class="toc-list">
          <li
            v-for="item in tocItems"
            :key="item.id"
            :ref="(el: any) => { if (el) tocItemRefs[item.id] = el }"
            :class="['toc-item', `toc-level-${item.level}`, { 'toc-active': activeId === item.id }]"
          >
            <a
              :href="`#${item.id}`"
              class="toc-link"
              @click.prevent="scrollToHeading(item.id)"
            >{{ item.text }}</a>
          </li>
        </ul>
      </nav>
    </aside>

    <!-- Main Content -->
    <div class="post-detail flex-1 min-w-0 max-w-3xl py-12">
    <div v-if="loading" class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">加载中...</div>

    <article v-else-if="post">
      <!-- Edit + Back -->
      <div class="flex items-center mb-8">
        <router-link
          v-if="canEdit"
          :to="`/admin/posts/${post.id}`"
          class="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg transition-colors no-underline font-medium"
          :style="{ backgroundColor: 'var(--color-primary-bg)', color: 'var(--color-primary)' }"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/></svg>
          编辑文章
        </router-link>
        <button class="back-btn ml-auto" @click="$router.back()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
          返回
        </button>
      </div>

      <!-- Header -->
      <header class="mb-6">
        <!-- ActionBar above title -->
        <ActionBar
          :prev="adjacent.prev"
          :next="adjacent.next"
          :views="post.view_count || 0"
          :like-count="likeStatus.like_count"
          :liked="likeStatus.liked"
          :like-loading="likeLoading"
          :is-logged-in="isLoggedIn"
          :share-title="post.title"
          :share-text="post.summary || post.title"
          @toggle-like="toggleLike"
        />
        <h1 class="text-3xl md:text-4xl font-extrabold mb-4 mt-4 leading-tight" :style="{ color: 'var(--color-text)' }">
          {{ post.title }}
        </h1>

        <!-- Article type & status badges -->
        <div v-if="articleBadges" class="flex flex-wrap gap-2 mb-3">
          <span v-if="typeBadge" class="badge badge-type-detail" :class="'badge-type-' + typeBadge.key">{{ typeBadge.label }}</span>
          <span v-if="statusBadge" class="badge badge-status-detail" :class="'badge-status-' + statusBadge.key">{{ statusBadge.label }}</span>
        </div>

        <div class="flex flex-wrap items-center gap-4 text-sm" :style="{ color: 'var(--color-text-muted)' }">
          <!-- Date -->
          <span v-if="post.published_at" class="flex items-center gap-1">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"/></svg>
            {{ dayjs(post.published_at).format('YYYY年MM月DD日') }}
          </span>
          <!-- Author -->
          <router-link v-if="post.author" :to="`/author/${post.author.id}`" class="flex items-center gap-1.5 no-underline transition-colors" :style="{ color: 'var(--color-text-muted)' }">
            <span class="avatar-circle">{{ (post.author.display_name || 'A')[0].toUpperCase() }}</span>
            {{ post.author.display_name || post.author.username || '匿名' }}
          </router-link>
          <!-- Category -->
          <span v-if="post.category_name">
            <router-link :to="`/category/${post.category_name}`" class="px-2 py-0.5 text-xs rounded-full transition-colors no-underline" :style="{ backgroundColor: 'var(--color-primary-bg)', color: 'var(--color-primary)' }">
              {{ post.category_name }}
            </router-link>
          </span>
          <!-- Tags -->
          <template v-if="post.tags?.length">
            <router-link
              v-for="tag in post.tags"
              :key="tag"
              :to="`/tag/${tag}`"
              class="px-2 py-0.5 text-xs rounded-full transition-colors no-underline"
              :style="{ backgroundColor: 'var(--color-bg-hover)', color: 'var(--color-text-secondary)' }"
            >{{ tag }}</router-link>
          </template>
        </div>
      </header>

      <!-- Cover Image -->
      <div v-if="post.cover_image || post.category_cover_image" class="mb-10">
        <img :src="post.cover_image || post.category_cover_image" :alt="post.title" class="w-full rounded-2xl shadow-lg" referrerpolicy="no-referrer" />
      </div>

      <!-- Content -->
      <div @contextmenu="handleContextMenu">
        <CodeCopyWrapper
          class="markdown-body"
          :html="post.content_html || ''"
          @need-login="showLoginDialog = true"
        />
      </div>

      <!-- 登录引导弹窗 -->
      <Teleport to="body">
        <div v-if="showLoginDialog" class="login-dialog-overlay" @click.self="showLoginDialog = false">
          <div class="login-dialog-box">
            <h3 class="login-dialog-title">登录后可复制代码</h3>
            <p class="login-dialog-desc">登录后即可使用代码复制、点赞等全部功能</p>
            <div class="login-dialog-actions">
              <button class="login-dialog-cancel" @click="showLoginDialog = false">稍后再说</button>
              <button class="login-dialog-login" @click="goToLogin">立即登录</button>
            </div>
          </div>
        </div>
      </Teleport>

      <!-- Bottom ActionBar -->
      <ActionBar
        :prev="adjacent.prev"
        :next="adjacent.next"
        :views="post.view_count || 0"
        :like-count="likeStatus.like_count"
        :liked="likeStatus.liked"
        :like-loading="likeLoading"
        :is-logged-in="isLoggedIn"
        :share-title="post.title"
        :share-text="post.summary || post.title"
        :dropdown-up="true"
        @toggle-like="toggleLike"
      />

      <!-- Bottom back row -->
      <div class="flex justify-center mt-8 pt-6 border-t" :style="{ borderColor: 'var(--color-border)' }">
        <button class="back-btn" @click="$router.back()">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><polyline points="15 18 9 12 15 6"/></svg>
          返回
        </button>
      </div>
    </article>

    <!-- ===================== COMMENTS SECTION ===================== -->
    <section v-if="post && post.allow_comment !== false" class="comments-section mt-12 pt-8 border-t" :style="{ borderColor: 'var(--color-border)' }">
      <!-- Comment count -->
      <h3 class="flex items-center gap-2 text-base font-semibold mb-4" :style="{ color: 'var(--color-text)' }">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
        </svg>
        评论 {{ comments.length > 0 ? `(${totalCommentCount})` : '' }}
      </h3>

      <!-- Comment form -->
      <form @submit.prevent="submitComment" class="mb-8 p-5 rounded-xl border" :style="{ borderColor: 'var(--color-border)', backgroundColor: 'var(--color-bg-card)' }">
        <div v-if="!isLoggedIn" class="flex flex-col sm:flex-row gap-3 mb-3">
          <input
            v-model="commentForm.author_name"
            type="text"
            placeholder="你的昵称 *"
            class="flex-1 px-2.5 py-2 rounded-lg text-xs outline-none border transition-colors"
            style="background: var(--color-bg); color: var(--color-text); border-color: var(--color-border)"
            required
          />
          <input
            v-model="commentForm.author_email"
            type="email"
            placeholder="邮箱 (可选)"
            class="flex-1 px-2.5 py-2 rounded-lg text-xs outline-none border transition-colors"
            style="background: var(--color-bg); color: var(--color-text); border-color: var(--color-border)"
          />
        </div>
        <textarea
          v-model="commentForm.content"
          rows="3"
          placeholder="写下你的评论，支持 Markdown..."
          class="w-full px-2.5 py-2 rounded-lg text-xs outline-none border transition-colors resize-y"
          style="background: var(--color-bg); color: var(--color-text); border-color: var(--color-border); min-height: 80px"
          required
        ></textarea>
        <div class="flex items-center justify-between mt-3">
          <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">支持 **粗体** *斜体* `代码` 等 Markdown 语法</span>
          <button
            type="submit"
            :disabled="commentSubmitting"
            class="px-4 py-1.5 rounded-lg text-xs font-medium text-white border-0 cursor-pointer transition-all"
            style="background: #4f46e5"
            :style="commentSubmitting ? { opacity: 0.5, cursor: 'not-allowed' } : {}"
          >
            {{ commentSubmitting ? '提交中...' : '发表评论' }}
          </button>
        </div>
        <div v-if="commentError" class="mt-2 text-xs px-3 py-2 rounded-lg" style="background: rgba(239,68,68,0.08); color: #f87171">{{ commentError }}</div>
      </form>

      <!-- Comment list -->
      <div v-if="commentLoading" class="text-center py-6" :style="{ color: 'var(--color-text-muted)' }">加载评论中...</div>
      <div v-else-if="comments.length === 0" class="text-center py-8" :style="{ color: 'var(--color-text-muted)' }">
        <p class="text-sm">还没有评论，来抢沙发吧～</p>
      </div>
      <div v-else class="space-y-5">
        <div v-for="c in comments" :key="c.id" :id="`comment-${c.id}`" class="comment-item scroll-mt-20">
          <!-- Top-level comment -->
          <div class="flex gap-3">
            <div class="w-9 h-9 rounded-full flex items-center justify-center text-white flex-shrink-0" style="background: #4f46e5; font-size: 13px; font-weight: 600">
              {{ (c.author_name || '匿')[0] }}
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1.5">
                <!-- Registered user: clickable, navigates to author page -->
                <span
                  v-if="c.user_id"
                  class="text-xs font-semibold cursor-pointer transition-colors hover:underline"
                  :style="{ color: 'var(--color-primary)' }"
                  @click="router.push(`/author/${c.user_id}`)"
                >{{ c.author_name }}</span>
                <!-- Visitor: plain text, non-clickable -->
                <span
                  v-else
                  class="text-xs font-semibold"
                  :style="{ color: 'var(--color-text)' }"
                >{{ c.author_name }}</span>
                <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">{{ dayjs(c.created_at).format('YYYY-MM-DD HH:mm') }}</span>
                <span v-if="isAdminOrSubAdmin && c.status !== 'approved'" class="text-xs px-1.5 py-0.5 rounded" :style="{ background: statusBg(c.status), color: statusFg(c.status) }">{{ statusLabel(c.status) }}</span>
              </div>
              <div class="comment-content text-sm leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }" v-html="c.content_html"></div>
              <div class="flex items-center gap-4 mt-2">
                <button @click="startReply(c)" class="inline-flex items-center gap-1 text-xs transition-colors border-0 bg-transparent cursor-pointer" :style="{ color: 'var(--color-text-muted)' }">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
                  回复
                </button>
              </div>

              <!-- Reply form -->
              <div v-if="replyTargetId === c.id" class="mt-3 p-4 rounded-lg border" :style="{ borderColor: 'var(--color-border)', backgroundColor: 'var(--color-bg)' }">
                <div v-if="!isLoggedIn" class="flex gap-2 mb-2">
                  <input v-model="replyForm.author_name" type="text" placeholder="昵称" class="flex-1 px-3 py-2 rounded-lg text-xs outline-none border" :style="{ borderColor: 'var(--color-border)' }" required />
                </div>
                <textarea v-model="replyForm.content" rows="2" :placeholder="`回复 @${c.author_name}...`" class="w-full px-3 py-2 rounded-lg text-sm outline-none border resize-y" :style="{ borderColor: 'var(--color-border)' }" required></textarea>
                <div class="flex justify-end gap-2 mt-2">
                  <button @click="cancelReply" class="px-3 py-1.5 rounded-lg text-xs border bg-transparent cursor-pointer" :style="{ borderColor: 'var(--color-border)', color: 'var(--color-text-secondary)' }">取消</button>
                  <button @click="submitReply(c.id)" :disabled="replySubmitting" class="px-3 py-1.5 rounded-lg text-xs text-white border-0 cursor-pointer" style="background: #4f46e5">
                    {{ replySubmitting ? '提交中...' : '回复' }}
                  </button>
                </div>
              </div>

              <!-- Nested replies -->
              <div v-if="c.replies && c.replies.length" class="mt-3 space-y-3 pl-2">
                <div v-for="r in c.replies" :key="r.id" class="flex gap-2.5">
                  <div class="w-7 h-7 rounded-full flex items-center justify-center text-white flex-shrink-0" style="background: #818cf8; font-size: 11px; font-weight: 600">
                    {{ (r.author_name || '匿')[0] }}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-1">
                      <!-- Registered user: clickable -->
                      <span
                        v-if="r.user_id"
                        class="text-xs font-semibold cursor-pointer transition-colors hover:underline"
                        :style="{ color: 'var(--color-primary)' }"
                        @click="router.push(`/author/${r.user_id}`)"
                      >{{ r.author_name }}</span>
                      <!-- Visitor: plain text -->
                      <span
                        v-else
                        class="text-xs font-semibold"
                        :style="{ color: 'var(--color-text)' }"
                      >{{ r.author_name }}</span>
                      <span class="text-xs" :style="{ color: 'var(--color-text-muted)' }">{{ dayjs(r.created_at).format('YYYY-MM-DD HH:mm') }}</span>
                    </div>
                    <div class="text-xs leading-relaxed" :style="{ color: 'var(--color-text-secondary)' }" v-html="r.content_html"></div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <div v-else class="text-center py-12" :style="{ color: 'var(--color-text-muted)' }">
      <p class="text-lg mb-2">文章未找到</p>
      <router-link to="/" class="text-sm" :style="{ color: 'var(--color-primary)' }">返回首页</router-link>
    </div>

  </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, watch, computed, onUnmounted, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import api from '@/api/index'
import { recordReadLog } from '@/api/admin'
import dayjs from 'dayjs'
import { fetchComments, createComment } from '@/api/comments'
import { useAuthStore } from '@/stores/auth'
import ActionBar from '@/components/front/ActionBar.vue'
import CodeCopyWrapper from '@/components/shared/CodeCopyWrapper.vue'

// TOC state
const tocRef = ref<HTMLElement | null>(null)
const tocItemRefs = ref<Record<string, HTMLElement>>({})
const tocItems = ref<{ id: string; text: string; level: number }[]>([])
const activeId = ref('')
let observer: IntersectionObserver | null = null

// Auto-scroll TOC to keep active item visible
watch(activeId, (id) => {
  if (!id) return
  const el = tocItemRefs.value[id]
  if (!el) return
  el.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
})

// Generate heading ID from text (supports Chinese)
function makeHeadingId(text: string): string {
  return text
    .trim()
    .toLowerCase()
    .replace(/\s+/g, '-')
    .replace(/[^\w\u4e00-\u9fff-]/g, '')
    .replace(/^-+|-+$/g, '')
    .substring(0, 60)
}

// Parse headings from HTML and build TOC items
function parseToc(html: string) {
  const headingRegex = /<h([1-3])(?:\s[^>]*)?>(.+?)<\/h[1-3]>/gi
  const items: { id: string; text: string; level: number }[] = []
  const usedIds = new Set<string>()
  let match: RegExpExecArray | null
  while ((match = headingRegex.exec(html)) !== null) {
    const level = parseInt(match[1])
    const innerHtml = match[2]
    // Strip inner HTML tags to get plain text
    const text = innerHtml.replace(/<[^>]*>/g, '').trim()
    if (!text) continue
    let id = makeHeadingId(text)
    // Deduplicate
    if (usedIds.has(id)) {
      let i = 2
      while (usedIds.has(`${id}-${i}`)) i++
      id = `${id}-${i}`
    }
    usedIds.add(id)
    items.push({ id, text, level })
  }
  return items
}

// Inject IDs into heading tags in HTML
function injectHeadingIds(html: string, items: { id: string; text: string; level: number }[]): string {
  let idx = 0
  return html.replace(/<h([1-3])(\s[^>]*)?>/gi, (fullMatch, level: string, attrs: string) => {
    if (idx < items.length) {
      const item = items[idx++]
      const existingId = (attrs || '').match(/id="([^"]*)"/)
      if (existingId) {
        return fullMatch
      }
      // Insert id attribute right after the tag name
      return `<h${level} id="${item.id}"${attrs || ''}>`
    }
    return fullMatch
  })
}

function scrollToHeading(id: string) {
  const el = document.getElementById(id)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'start' })
    // Update URL hash without triggering scroll
    history.replaceState(null, '', `#${id}`)
    activeId.value = id
  }
}

function setupTocObserver() {
  cleanupObserver()
  const headingEls = tocItems.value
    .map(item => document.getElementById(item.id))
    .filter(Boolean) as HTMLElement[]
  
  if (headingEls.length === 0) return

  observer = new IntersectionObserver(
    (entries) => {
      // Find the first heading that's intersecting (visible)
      for (const entry of entries) {
        if (entry.isIntersecting) {
          activeId.value = entry.target.id
          return
        }
      }
      // If no heading is visible, find the last one above viewport
      for (let i = headingEls.length - 1; i >= 0; i--) {
        const rect = headingEls[i].getBoundingClientRect()
        if (rect.top < window.innerHeight) {
          activeId.value = headingEls[i].id
          return
        }
      }
    },
    { rootMargin: '-80px 0px -70% 0px' }
  )

  headingEls.forEach(el => observer!.observe(el))
}

function cleanupObserver() {
  if (observer) {
    observer.disconnect()
    observer = null
  }
}

// ── Code block enhancement ──
function enhanceCodeBlocks() {
  const container = document.querySelector('.markdown-body')
  if (!container) return
  const pres = container.querySelectorAll('pre')
  pres.forEach(pre => {
    // Skip if already enhanced
    if (pre.parentElement?.classList.contains('code-block-wrapper')) return
    const code = pre.querySelector('code')
    const lang = code?.className.match(/language-(\w+)/)?.[1] || ''
    
    const wrapper = document.createElement('div')
    wrapper.className = 'code-block-wrapper'
    
    // Header bar with language label + copy button
    const header = document.createElement('div')
    header.className = 'code-block-header'
    header.innerHTML = `<span class="code-lang">${lang}</span><button class="code-copy-btn" onclick="navigator.clipboard.writeText(this.closest('.code-block-wrapper')!.querySelector('code')!.textContent!).then(()=>{this.textContent='已复制';setTimeout(()=>{this.textContent='复制'},1500)})">复制</button>`
    
    pre.parentNode!.insertBefore(wrapper, pre)
    wrapper.appendChild(header)
    wrapper.appendChild(pre)
  })
}

const route = useRoute()
const router = useRouter()
const loading = ref(false)
const post = ref<Post | null>(null)
const readStartTime = ref(0)
const adjacent = ref<{ prev: {id:number,title:string,slug:string}|null, next: {id:number,title:string,slug:string}|null }>({ prev: null, next: null })
const likeStatus = ref({ liked: false, like_count: 0 })
const likeLoading = ref(false)

const authStore = useAuthStore()
const isLoggedIn = computed(() => authStore.isAuthenticated)
const showLoginDialog = ref(false)

// ── Article type & status badges ──
const TYPE_MAP_DETAIL: Record<string, string> = {
  original: '📝 原创实践',
  ai_organized: '🤖 AI 整理',
  knowledge_summary: '📚 知识汇总',
  reprint_translation: '🔗 转载翻译',
  opinion_essay: '💡 观点随笔',
}
const STATUS_MAP_DETAIL: Record<string, string> = {
  latest: '✅ 最新',
  partially_outdated: '⚠️ 部分过时',
  outdated: '❌ 已过时',
  continuously_updated: '🚧 持续更新',
  classic_archive: '📌 经典存档',
  experimental: '🧪 实验性',
}

const typeBadge = computed(() => {
  const t = (post.value as any)?.article_type
  return t && TYPE_MAP_DETAIL[t] ? { key: t, label: TYPE_MAP_DETAIL[t] } : null
})
const statusBadge = computed(() => {
  const s = (post.value as any)?.article_status
  return s && STATUS_MAP_DETAIL[s] ? { key: s, label: STATUS_MAP_DETAIL[s] } : null
})
const articleBadges = computed(() => !!(typeBadge.value || statusBadge.value))

function goToLogin() {
  showLoginDialog.value = false
  router.push({ name: 'login', query: { redirect: route.fullPath } })
}

function handleContextMenu(e: MouseEvent) {
  if (!isLoggedIn.value) {
    e.preventDefault()
    showLoginDialog.value = true
  }
}

function handleKeyDown(e: KeyboardEvent) {
  if (isLoggedIn.value) return
  // Only intercept Ctrl+C / Cmd+C
  if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
    // Don't intercept when focused on input/textarea (user might be copying from form)
    const tag = (e.target as HTMLElement)?.tagName
    if (tag === 'INPUT' || tag === 'TEXTAREA') return
    e.preventDefault()
    showLoginDialog.value = true
  }
}
const isAdminOrSubAdmin = computed(() => {
  const u = authStore.user
  if (u) return u.role === 'admin' || u.role === 'sub_admin'
  return false
})

function getUserInfo(): { id: number; role: string } | null {
  const u = authStore.user
  return u ? { id: u.id, role: u.role } : null
}

const canEdit = computed(() => {
  if (!isLoggedIn.value || !post.value) return false
  const user = getUserInfo()
  if (!user) return false
  // Admin / sub_admin can edit all posts
  if (user.role === 'admin' || user.role === 'sub_admin') return true
  // Author can only edit their own posts
  if (user.role === 'author') {
    const authorId = (post.value as any).user_id ?? post.value.author?.id
    return user.id === authorId
  }
  return false
})

async function loadPost() {
  const slug = route.params.slug as string
  if (!slug) return
  // Backward compat: if param is numeric ID, fetch by ID → redirect to slug-based URL
  if (/^\d+$/.test(slug)) {
    try {
      const { data: resp } = await api.get<{ data: Post }>(`/posts/${slug}`)
      if (resp.data?.slug) {
        router.replace(`/post/${resp.data.slug}`)
        return
      }
    } catch { /* fall through to 404 */ }
  }
  loading.value = true
  try {
    const { data: resp } = await api.get<{ data: Post }>(`/posts/slug/${slug}`)
    post.value = resp.data
    // Record read log entry — use post.id from loaded data
    readStartTime.value = Date.now()
    recordReadLog({ post_id: resp.data.id, referrer: document.referrer }).catch(() => {})

    // Process content_html: inject heading IDs, build TOC
    if (post.value?.content_html) {
      const items = parseToc(post.value.content_html)
      tocItems.value = items
      if (items.length > 0) {
        post.value.content_html = injectHeadingIds(post.value.content_html, items)
      }
      // Add referrerpolicy to external images to prevent CDN hotlink blocking
      post.value.content_html = post.value.content_html.replace(
        /<img\s/gi,
        '<img referrerpolicy="no-referrer" '
      )
      await nextTick()
      enhanceCodeBlocks()
      if (items.length > 0) {
        setupTocObserver()
      }
    }

    // Init like count from post
    likeStatus.value.like_count = post.value?.like_count || 0

    // Fetch adjacent posts
    if (post.value?.id) {
      try {
        const { data: adjResp } = await api.get<{ data: { prev: any, next: any } }>(`/posts/${post.value.id}/adjacent`)
        adjacent.value = adjResp.data
      } catch { /* ignore */ }

      // Fetch like status if logged in
      if (isLoggedIn.value) {
        try {
          const { data: likeResp } = await api.get<{ data: { liked: boolean, like_count: number } }>(
            `/posts/${post.value.id}/like-status`
          )
          likeStatus.value = likeResp.data
        } catch { /* ignore */ }
      }
    }

    // Load comments
    loadComments()
  } catch {
    post.value = null
  } finally {
    loading.value = false
  }
}

async function toggleLike() {
  if (!post.value?.id || !isLoggedIn.value) {
    router.replace('/login')
    return
  }
  likeLoading.value = true
  try {
    const { data: resp } = await api.post<{ data: { liked: boolean, like_count: number } }>(
      `/posts/${post.value.id}/like`
    )
    likeStatus.value = resp.data
  } catch {
    // If 401, redirect to login
    router.replace('/login')
  }
  likeLoading.value = false
}

// ── Comments ──
const comments = ref<Comment[]>([])
const commentLoading = ref(false)
const commentSubmitting = ref(false)
const commentError = ref('')
const replyTargetId = ref<number | null>(null)
const replySubmitting = ref(false)
const commentForm = reactive({ content: '', author_name: localStorage.getItem('marksharex_visitor_name') || '', author_email: '' })
const replyForm = reactive({ content: '', author_name: localStorage.getItem('marksharex_visitor_name') || '' })

const totalCommentCount = computed(() => {
  let n = comments.value.length
  for (const c of comments.value) {
    if (c.replies) n += c.replies.length
  }
  return n
})

async function loadComments() {
  if (!post.value?.id) return
  commentLoading.value = true
  try {
    const { data: resp } = await fetchComments(post.value.id, isAdminOrSubAdmin.value)
    comments.value = resp.data || []
    // After loading comments, scroll to anchor if present
    await nextTick()
    scrollToCommentHash()
  } catch { /* ignore */ }
  commentLoading.value = false
}

function statusLabel(s: string): string {
  const map: Record<string, string> = { pending: '待审', deleted: '已删' }
  return map[s] || s
}
function statusBg(s: string): string {
  const map: Record<string, string> = { pending: 'rgba(245,158,11,0.15)', deleted: 'rgba(239,68,68,0.15)' }
  return map[s] || 'transparent'
}
function statusFg(s: string): string {
  const map: Record<string, string> = { pending: '#f59e0b', deleted: '#ef4444' }
  return map[s] || '#9ca3af'
}

function scrollToCommentHash() {
  const hash = window.location.hash
  if (!hash || !hash.startsWith('#comment-')) return

  // Retry: dynamic content may not be rendered yet
  let attempts = 0
  const maxAttempts = 10
  const tryScroll = () => {
    const el = document.getElementById(hash.slice(1))
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' })
      // Highlight the comment briefly
      el.style.transition = 'background 0.3s'
      const origBg = el.style.background
      el.style.background = 'rgba(79,70,229,0.12)'
      setTimeout(() => { el.style.background = origBg }, 1500)
      return
    }
    attempts++
    if (attempts < maxAttempts) {
      setTimeout(tryScroll, 200)
    }
  }
  tryScroll()
}

async function submitComment() {
  if (!commentForm.content.trim() || !post.value?.id) return
  commentSubmitting.value = true
  commentError.value = ''
  try {
    const { data: resp } = await createComment(post.value.id, {
      content: commentForm.content,
      author_name: commentForm.author_name || undefined,
      author_email: commentForm.author_email || undefined,
    })
    const newId = resp.data.id
    // Save visitor name
    if (!isLoggedIn.value && commentForm.author_name) {
      localStorage.setItem('marksharex_visitor_name', commentForm.author_name)
    }
    commentForm.content = ''
    commentForm.author_name = localStorage.getItem('marksharex_visitor_name') || ''
    commentForm.author_email = ''
    await loadComments()
    // Scroll to new comment
    await nextTick()
    const el = document.getElementById(`comment-${newId}`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' })
      el.style.transition = 'background 0.3s'
      const origBg = el.style.background
      el.style.background = 'rgba(79,70,229,0.12)'
      setTimeout(() => { el.style.background = origBg }, 2000)
    }
  } catch (e: any) {
    commentError.value = e?.response?.data?.error || '评论失败，请稍后重试'
  }
  commentSubmitting.value = false
}

function startReply(c: Comment) {
  replyTargetId.value = c.id
  replyForm.content = ''
  replyForm.author_name = localStorage.getItem('marksharex_visitor_name') || ''
}

function cancelReply() {
  replyTargetId.value = null
  replyForm.content = ''
}

async function submitReply(parentId: number) {
  if (!replyForm.content.trim() || !post.value?.id) return
  replySubmitting.value = true
  try {
    const { data: resp } = await createComment(post.value.id, {
      content: replyForm.content,
      parent_id: parentId,
      author_name: replyForm.author_name || undefined,
    })
    const newId = resp.data.id
    // Save visitor name
    if (!isLoggedIn.value && replyForm.author_name) {
      localStorage.setItem('marksharex_visitor_name', replyForm.author_name)
    }
    replyTargetId.value = null
    replyForm.content = ''
    replyForm.author_name = ''
    await loadComments()
    // Scroll to new reply
    await nextTick()
    const el = document.getElementById(`comment-${newId}`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' })
      el.style.transition = 'background 0.3s'
      const origBg = el.style.background
      el.style.background = 'rgba(79,70,229,0.12)'
      setTimeout(() => { el.style.background = origBg }, 2000)
    }
  } catch {
    // ignore
  }
  replySubmitting.value = false
}

onMounted(() => {
  loadPost()
  document.addEventListener('keydown', handleKeyDown)
})
onUnmounted(() => {
  document.removeEventListener('keydown', handleKeyDown)
  // Record read duration on leave
  if (readStartTime.value && post.value?.id) {
    const duration = Math.round((Date.now() - readStartTime.value) / 1000)
    recordReadLog({ post_id: post.value.id, duration_seconds: duration }).catch(() => {})
  }
  cleanupObserver()
})
watch(() => route.params.slug, () => {
  // Record exit duration for previous post
  if (readStartTime.value && post.value?.id) {
    const duration = Math.round((Date.now() - readStartTime.value) / 1000)
    recordReadLog({ post_id: post.value.id, duration_seconds: duration }).catch(() => {})
  }
  cleanupObserver()
  tocItems.value = []
  activeId.value = ''
  loadPost()
})
</script>

<style scoped>
/* Scroll offset for anchor links — prevent headings hiding behind sticky navbar */
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  scroll-margin-top: 90px;
}

/* Scroll margin for comment anchors */
.comment-item {
  scroll-margin-top: 100px;
}

/* Avatar circles */
.avatar-circle {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  background: linear-gradient(135deg, #4f46e5, #818cf8);
  color: #fff;
  flex-shrink: 0;
}
.avatar-large {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  font-weight: 700;
  background: linear-gradient(135deg, #4f46e5, #818cf8);
  color: #fff;
  flex-shrink: 0;
}

/* Like button */
button.liked {
  animation: heartPulse 0.3s ease;
}
@keyframes heartPulse {
  0% { transform: scale(1); }
  50% { transform: scale(1.2); }
  100% { transform: scale(1); }
}


/* ===== TOC Sidebar ===== */
.post-detail-wrapper {
  display: flex;
  align-items: flex-start;
  gap: 24px;
}

.toc-sidebar {
  position: sticky;
  top: 90px;
  width: 200px;
  max-height: calc(100vh - 140px);
  overflow-y: auto;
  flex-shrink: 0;
  padding-right: 12px;
  border-right: 1px solid var(--color-border);
}

.toc-nav {
  font-size: 13px;
}

.toc-title {
  font-weight: 700;
  font-size: 14px;
  margin-bottom: 12px;
  color: var(--color-text);
  letter-spacing: 0.5px;
}
.toc-empty {
  font-size: 12px;
  color: var(--color-text-muted);
  padding: 8px 0;
}

.toc-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.toc-item {
  margin-bottom: 2px;
}

.toc-link {
  display: block;
  padding: 5px 8px;
  border-radius: 6px;
  color: var(--color-text-muted);
  text-decoration: none;
  line-height: 1.5;
  transition: all 0.15s ease;
  border-left: 2px solid transparent;
}

.toc-link:hover {
  color: var(--color-text);
  background: var(--color-bg-hover);
}

.toc-level-1 .toc-link {
  font-weight: 600;
  padding-left: 8px;
}

.toc-level-2 .toc-link {
  padding-left: 20px;
}

.toc-level-3 .toc-link {
  padding-left: 32px;
  font-size: 12px;
}

.toc-active .toc-link {
  color: var(--color-primary);
  background: var(--color-primary-bg);
  border-left-color: var(--color-primary);
  font-weight: 600;
}

/* Scrollbar for TOC */
.toc-sidebar::-webkit-scrollbar {
  width: 4px;
}
.toc-sidebar::-webkit-scrollbar-thumb {
  background: var(--color-border);
  border-radius: 2px;
}

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

/* ========== Code Block Styles ========== */
.code-block-wrapper {
  margin: 16px 0;
  border-radius: 10px;
  overflow: hidden;
  border: 1px solid rgba(255,255,255,0.08);
  background: #1a1a2e;
}
.dark .code-block-wrapper {
  background: #0f0f18;
  border-color: rgba(255,255,255,0.06);
}

.code-block-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 14px;
  background: rgba(255,255,255,0.03);
  border-bottom: 1px solid rgba(255,255,255,0.05);
}

.code-lang {
  font-size: 11px;
  color: #818cf8;
  font-family: monospace;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* 登录引导弹窗 */
.login-dialog-overlay {
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.5);
  display: flex; align-items: center; justify-content: center;
  z-index: 10000;
}
.login-dialog-box {
  background: var(--color-bg-card, #1e293b);
  border: 1px solid var(--color-border, #334155);
  border-radius: 12px;
  padding: 28px 32px;
  max-width: 380px;
  width: 90%;
  text-align: center;
}
.login-dialog-title {
  font-size: 18px;
  color: var(--color-text, #f1f5f9);
  margin: 0 0 8px;
}
.login-dialog-desc {
  font-size: 14px;
  color: var(--color-text-secondary, #94a3b8);
  margin: 0 0 24px;
  line-height: 1.5;
}
.login-dialog-actions {
  display: flex; gap: 12px; justify-content: center;
}
.login-dialog-cancel {
  padding: 8px 20px;
  border-radius: 8px;
  border: 1px solid var(--color-border, #334155);
  background: transparent;
  color: var(--color-text-secondary, #94a3b8);
  font-size: 14px; cursor: pointer;
}
.login-dialog-login {
  padding: 8px 20px;
  border-radius: 8px;
  border: none;
  background: var(--color-primary, #818cf8);
  color: #fff;
  font-size: 14px; cursor: pointer;
  font-weight: 500;
}

.code-block-wrapper pre {
  margin: 0 !important;
  padding: 14px 16px;
  background: transparent !important;
  border: none !important;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.65;
}

.code-block-wrapper code {
  background: transparent !important;
  padding: 0 !important;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', Consolas, monospace;
  font-size: 13px;
  color: #e2e8f0;
}

/* Inline code (not in a code block) */
.markdown-body :not(pre) > code {
  background: rgba(79,70,229,0.08);
  color: #818cf8;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.88em;
  font-family: 'JetBrains Mono', 'Fira Code', Consolas, monospace;
}

/* ===== Article type & status badges ===== */
.badge {
  font-size: 12px;
  padding: 2px 10px;
  border-radius: 6px;
  font-weight: 500;
  white-space: nowrap;
}
.badge-type-original { background: rgba(59,130,246,0.12); color: #60a5fa; }
.badge-type-ai_organized { background: rgba(168,85,247,0.12); color: #c084fc; }
.badge-type-knowledge_summary { background: rgba(34,197,94,0.12); color: #4ade80; }
.badge-type-reprint_translation { background: rgba(251,146,60,0.12); color: #fb923c; }
.badge-type-opinion_essay { background: rgba(236,72,153,0.12); color: #f472b6; }
.badge-status-latest { background: rgba(34,197,94,0.12); color: #4ade80; }
.badge-status-partially_outdated { background: rgba(250,204,21,0.12); color: #facc15; }
.badge-status-outdated { background: rgba(239,68,68,0.12); color: #f87171; }
.badge-status-continuously_updated { background: rgba(59,130,246,0.12); color: #60a5fa; }
.badge-status-classic_archive { background: rgba(139,92,246,0.12); color: #a78bfa; }
.badge-status-experimental { background: rgba(34,211,238,0.12); color: #22d3ee; }
</style>
