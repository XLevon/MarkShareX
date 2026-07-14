import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const frontRoutes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('@/components/front/FrontLayout.vue'),
    children: [
      { path: '', name: 'home', component: () => import('@/views/front/Home.vue') },
      { path: 'knowledge-base', name: 'knowledge-base', component: () => import('@/views/front/KnowledgeBase.vue') },
      { path: 'post/:slug(.*)', name: 'post', component: () => import('@/views/front/PostDetail.vue') },
      { path: 'tags', name: 'tags', component: () => import('@/views/front/TagsAll.vue') },
      { path: 'tag/:slug', name: 'tag', component: () => import('@/views/front/TagPosts.vue') },
      { path: 'types', name: 'types', component: () => import('@/views/front/ArticleFilter.vue'), props: { filterType: 'type' } },
      { path: 'type/:code', name: 'type', component: () => import('@/views/front/ArticleFilter.vue'), props: { filterType: 'type' } },
      { path: 'statuses', name: 'statuses', component: () => import('@/views/front/ArticleFilter.vue'), props: { filterType: 'status' } },
      { path: 'status/:code', name: 'status', component: () => import('@/views/front/ArticleFilter.vue'), props: { filterType: 'status' } },
      { path: 'categories', name: 'categories', component: () => import('@/views/front/CategoriesAll.vue') },
      { path: 'category/:slug', name: 'category', component: () => import('@/views/front/CategoryPosts.vue') },
      { path: 'authors', name: 'authors', component: () => import('@/views/front/AuthorsList.vue') },
      { path: 'author/:id', name: 'author', component: () => import('@/views/front/AuthorPosts.vue') },
      { path: 'search', name: 'search', component: () => import('@/views/front/SearchResults.vue') },
      { path: 'login', name: 'login', component: () => import('@/views/front/Login.vue') },
      { path: 'register', name: 'register', component: () => import('@/views/front/Register.vue') },
      { path: 'apply', name: 'apply', component: () => import('@/views/front/ApplyAuthor.vue') },
      { path: 'changelog', name: 'changelog', component: () => import('@/views/front/Changelog.vue') },
      { path: 'pinned', name: 'pinned', component: () => import('@/views/front/PinnedPosts.vue') },
      { path: 'guestbook', name: 'guestbook', component: () => import('@/views/front/Guestbook.vue') },
    ],
  },
]

const adminRoutes: RouteRecordRaw[] = [
  {
    path: '/admin/setup',
    name: 'admin-setup',
    component: () => import('@/views/admin/Setup.vue'),
  },
  {
    path: '/admin',
    component: () => import('@/components/admin/AdminLayout.vue'),
    meta: { requiresAuth: true },
    children: [
      { path: '', redirect: '/admin/dashboard' },
      { path: 'dashboard', name: 'admin-dashboard', component: () => import('@/views/admin/Dashboard.vue') },
      { path: 'posts', name: 'admin-posts', component: () => import('@/views/admin/PostList.vue') },
      { path: 'posts/new', name: 'admin-post-new', component: () => import('@/views/admin/PostEdit.vue') },
      { path: 'posts/:id', name: 'admin-post-edit', component: () => import('@/views/admin/PostEdit.vue') },
      { path: 'categories', name: 'admin-categories', component: () => import('@/views/admin/Categories.vue') },
      { path: 'tags', name: 'admin-tags', component: () => import('@/views/admin/Tags.vue') },
      { path: 'files', name: 'admin-files', component: () => import('@/views/admin/Files.vue') },
      { path: 'analytics/views', name: 'admin-analytics-views', component: () => import('@/views/admin/ViewsAnalytics.vue') },
      { path: 'analytics/comments', name: 'admin-analytics-comments', component: () => import('@/views/admin/CommentsAdmin.vue') },
      { path: 'likes', name: 'admin-likes', component: () => import('@/views/admin/Likes.vue') },
      { path: 'import', name: 'admin-import', component: () => import('@/views/admin/Import.vue') },
      { path: 'users', name: 'admin-users', component: () => import('@/views/admin/AdminUsers.vue') },
      { path: 'settings', name: 'admin-settings', component: () => import('@/views/admin/Settings.vue') },
      { path: 'guestbook', name: 'admin-guestbook', component: () => import('@/views/admin/AdminGuestbook.vue') },
      { path: 'news', name: 'admin-news', component: () => import('@/views/admin/NewsManage.vue') },
      { path: 'ai', name: 'admin-ai', component: () => import('@/views/admin/AIManage.vue') },
    ],
  },
]

const routes: RouteRecordRaw[] = [
  ...frontRoutes,
  ...adminRoutes,
  {
    path: '/:pathMatch(.*)*',
    name: 'not-found',
    component: () => import('@/views/front/NotFound.vue'),
  },
]

// Scroll position cache — saves actual scrollY on navigation away, restores on back
const scrollCache: Record<string, number> = {}

const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior(to, _from, savedPosition) {
    // Hash navigation — let page component handle it
    if (to.hash) {
      return false
    }
    // Browser back/forward: delay restoration until async content renders
    if (savedPosition) {
      return new Promise((resolve) => {
        // Multiple animation frames + delay ensure DOM has settled after data loads
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            setTimeout(() => resolve(savedPosition), 100)
          })
        })
      })
    }
    // New navigation: scroll to top
    return { top: 0 }
  },
})

// Save scroll position before leaving a page
router.beforeEach(async (to, from) => {
  // Auth guard
  if (to.meta.requiresAuth) {
    const token = localStorage.getItem('marksharex_token') || sessionStorage.getItem('marksharex_token')
    if (!token) {
      return { name: 'login', query: { redirect: to.fullPath } }
    }
    // Role check: only admin/sub_admin/author can access admin routes
    const adminRoles = ['admin', 'sub_admin', 'author']
    const userStr = localStorage.getItem('marksharex_user') || sessionStorage.getItem('marksharex_user')
    if (userStr) {
      try {
        const user = JSON.parse(userStr)
        if (!adminRoles.includes(user.role)) {
          return { name: 'home' }
        }
        // only admin can access AI module
        if (user.role !== 'admin' && to.path.startsWith('/admin/ai')) {
          return { name: 'admin-dashboard' }
        }
      } catch { /* corrupted user data, let through */ }
    }
  }
  // Save current scroll position for the page we're leaving
  if (from.path && from.path !== to.path) {
    scrollCache[from.fullPath] = window.scrollY || document.documentElement.scrollTop
  }
})

// After navigation, if savedPosition was null (e.g., query param change), fallback to cache
router.afterEach((to) => {
  const cached = scrollCache[to.fullPath]
  if (cached !== undefined) {
    delete scrollCache[to.fullPath]
    // Only restore from cache if not already handled by scrollBehavior
    // (scrollBehavior runs before afterEach, so we check with a microtask)
    setTimeout(() => {
      if (cached > 0 && window.scrollY < 100) {
        window.scrollTo({ top: cached, behavior: 'instant' as ScrollBehavior })
      }
    }, 350)
  }
})

// Listen for auth expiration from axios interceptor (401 responses)
// Only redirect for admin pages; public pages should not be interrupted
window.addEventListener('auth:expired', () => {
  if (window.location.pathname === '/login') return
  // Only redirect if on an admin page that requires authentication
  if (!window.location.pathname.startsWith('/admin')) return
  const currentPath = window.location.pathname + window.location.search
  router.replace({ name: 'login', query: { redirect: currentPath } })
})

export default router
