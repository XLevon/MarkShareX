import axios from 'axios'
import type { AxiosInstance } from 'axios'

const api: AxiosInstance = axios.create({
  baseURL: '/api/v1',
  timeout: 30000,
})

// 请求拦截器：自动附加 JWT（支持 localStorage 和 sessionStorage）
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('marksharex_token') || sessionStorage.getItem('marksharex_token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

// Token 刷新锁 — 防止并发 401 触发多个 refresh 导致的竞态条件
// 第一个 401 发起刷新，后续 401 共享同一个 Promise
let refreshPromise: Promise<{ access_token: string; refresh_token: string; user: any }> | null = null

function getTokenStorage(): Storage {
  return localStorage.getItem('marksharex_refresh_token') ? localStorage : sessionStorage
}

function clearAllTokens() {
  for (const key of ['marksharex_token', 'marksharex_refresh_token', 'marksharex_user']) {
    localStorage.removeItem(key)
    sessionStorage.removeItem(key)
  }
}

// 响应拦截器：Token 过期自动刷新或跳转登录
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      const refreshToken = localStorage.getItem('marksharex_refresh_token') || sessionStorage.getItem('marksharex_refresh_token')

      if (!refreshToken) {
        clearAllTokens()
        window.dispatchEvent(new CustomEvent('auth:expired'))
        return Promise.reject(error)
      }

      if (!error.config._retry) {
        error.config._retry = true
        try {
          // 并发 401 共享同一个刷新 Promise，避免第一个成功后第二个用旧 token 导致失败
          if (!refreshPromise) {
            refreshPromise = axios.post('/api/v1/auth/refresh', { refresh_token: refreshToken })
              .then(r => r.data.data)
              .finally(() => { refreshPromise = null })
          }

          const refreshed = await refreshPromise
          const { access_token, refresh_token: newRefresh, user } = refreshed

          // 写回原来的 storage（以首次 401 时的 storage 为准）
          const store = getTokenStorage()
          store.setItem('marksharex_token', access_token)
          store.setItem('marksharex_refresh_token', newRefresh)
          if (user) {
            store.setItem('marksharex_user', JSON.stringify(user))
          }

          // 通知 authStore 同步响应式 ref
          window.dispatchEvent(new CustomEvent('auth:refreshed', {
            detail: { token: access_token, refreshToken: newRefresh, user }
          }))

          error.config.headers.Authorization = `Bearer ${access_token}`
          return api(error.config)
        } catch {
          clearAllTokens()
          window.dispatchEvent(new CustomEvent('auth:expired'))
          return Promise.reject(error)
        }
      } else {
        // 已重试过但仍 401 → 刷新失败
        clearAllTokens()
        window.dispatchEvent(new CustomEvent('auth:expired'))
        return Promise.reject(error)
      }
    }
    return Promise.reject(error)
  }
)

export default api

// ========== 类型定义 ==========
export interface PaginatedData<T> {
  data: T[]
  pagination: {
    total: number
    pages: number
    page: number
    page_size: number
  }
}

export interface Post {
  id: number
  title: string
  slug: string
  content: string
  content_html: string
  summary: string
  cover_image: string
  category_cover_image?: string
  status: 'draft' | 'published'
  category_id: number | null
  category: Category | null
  tags: Tag[]
  view_count: number
  like_count: number
  comment_count: number
  article_type?: string   // code: original | ai_organized | ...
  article_type_name?: string  // display name from article_types table
  article_status?: string // code: latest | partially_outdated | ...
  article_status_name?: string  // display name from article_statuses table
  author?: string
  author_name?: string
  reading_time?: number
  is_pinned: boolean
  published_at: string
  created_at: string
  updated_at: string
}

export interface Category {
  id: number
  name: string
  slug: string
  description: string
  image_url?: string
  image_filename?: string
  is_visible: boolean
  parent_id?: number
  sort_order: number
  user_id?: number
  post_count: number
}

export interface Tag {
  id: number
  name: string
  slug: string
  user_id?: number
  post_count: number
}

export interface Comment {
  id: number
  post_id: number
  post_title?: string
  user_id: number | null
  parent_id: number | null
  author_name: string
  author_email?: string
  content: string
  content_html: string
  status: string
  like_count: number
  created_at: string
  updated_at: string
  replies: Comment[]
}

export interface FileInfo {
  id: number
  filename: string
  url: string
  mime_type: string
  size: number
  created_at: string
}

export interface SiteManagerInfo {
  display_name: string | null
  avatar_url: string | null
  bio: string | null
  title: string | null
  email: string | null
}

export function fetchSiteManagerInfo(): Promise<{ data: ApiResponse<SiteManagerInfo> }> {
  return api.get('/site/admin-info')
}

export interface Settings {
  site_title: string
  site_subtitle: string
  site_description: string
  site_logo: string
  friend_links: string
  comment_moderation: string
  sidebar_collapse: string
  guestbook_enabled: string
  batch_load_size: string
  scroll_load_size: string
  'site-manager'?: string
}
