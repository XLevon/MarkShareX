import axios from 'axios'
import type { AxiosInstance, InternalAxiosRequestConfig } from 'axios'
import {
  clearAuthSession,
  isAuthUser,
  readAuthSession,
  writeAuthSession,
  type AuthUser,
  type StoredAuthSession,
} from '@/utils/authStorage'

declare module 'axios' {
  interface InternalAxiosRequestConfig<D = any> {
    _retry?: boolean
    _authAccessToken?: string
    _authRefreshToken?: string
    _authReplayAccessToken?: string
  }
}

export interface ApiResponse<T> {
  data: T
  pagination?: {
    total: number
    page: number
    page_size: number
  }
}

const api: AxiosInstance = axios.create({
  baseURL: '/api/v1',
  timeout: 30000,
})

function sameSession(
  session: StoredAuthSession | null,
  accessToken: string,
  refreshToken: string,
): session is StoredAuthSession {
  return session?.accessToken === accessToken && session.refreshToken === refreshToken
}

function sessionKey(accessToken: string, refreshToken: string): string {
  return `${accessToken.length}:${accessToken}${refreshToken}`
}

// 请求发出时绑定身份。重放请求只能使用响应处理器明确指定、且仍属于
// 当前会话的 token，避免旧请求在新登录身份下执行。
api.interceptors.request.use((config) => {
  const current = readAuthSession()
  if (config._authReplayAccessToken) {
    if (!current
      || current.accessToken !== config._authReplayAccessToken
      || current.refreshToken !== config._authRefreshToken) {
      throw new axios.CanceledError('Authentication session changed before request replay')
    }
  }

  if (current) {
    config.headers.Authorization = `Bearer ${current.accessToken}`
    config._authAccessToken = current.accessToken
    config._authRefreshToken = current.refreshToken
  } else {
    delete config._authAccessToken
    delete config._authRefreshToken
  }
  return config
})

interface RefreshPayload {
  accessToken: string
  refreshToken: string
  user: AuthUser
}

interface RefreshContext extends RefreshPayload {
  originalAccessToken: string
  originalRefreshToken: string
  rememberMe: boolean
}

interface TokenRotation {
  nextKey: string
}

const refreshPromises = new Map<string, Promise<RefreshContext>>()
const tokenRotations = new Map<string, TokenRotation>()

function rememberRotation(context: RefreshContext): void {
  const originalKey = sessionKey(context.originalAccessToken, context.originalRefreshToken)
  tokenRotations.set(originalKey, {
    nextKey: sessionKey(context.accessToken, context.refreshToken),
  })
  while (tokenRotations.size > 32) {
    const oldest = tokenRotations.keys().next().value
    if (oldest === undefined) break
    tokenRotations.delete(oldest)
  }
}

function rotationReachesCurrent(
  originalAccessToken: string,
  originalRefreshToken: string,
  current: StoredAuthSession,
): boolean {
  let key = sessionKey(originalAccessToken, originalRefreshToken)
  const currentKey = sessionKey(current.accessToken, current.refreshToken)
  const visited = new Set<string>()
  while (!visited.has(key)) {
    if (key === currentKey) return true
    visited.add(key)
    const rotation = tokenRotations.get(key)
    if (!rotation) return false
    key = rotation.nextKey
  }
  return false
}

function refreshSession(session: StoredAuthSession): Promise<RefreshContext> {
  const key = sessionKey(session.accessToken, session.refreshToken)
  const existing = refreshPromises.get(key)
  if (existing) return existing

  const pending = axios.post('/api/v1/auth/refresh', { refresh_token: session.refreshToken })
    .then((response): RefreshContext => {
      const raw: unknown = response?.data?.data
      if (!raw || typeof raw !== 'object') throw new Error('Malformed refresh response')
      const candidate = raw as { access_token?: unknown; refresh_token?: unknown; user?: unknown }
      if (typeof candidate.access_token !== 'string' || !candidate.access_token
        || typeof candidate.refresh_token !== 'string' || !candidate.refresh_token) {
        throw new Error('Malformed refresh tokens')
      }
      const user = candidate.user === undefined ? session.user : candidate.user
      if (!isAuthUser(user)) throw new Error('Malformed refresh user')
      if (user.id !== session.user.id) throw new Error('Refresh identity mismatch')
      return {
        accessToken: candidate.access_token,
        refreshToken: candidate.refresh_token,
        user,
        originalAccessToken: session.accessToken,
        originalRefreshToken: session.refreshToken,
        rememberMe: session.storage === localStorage,
      }
    })
  const tracked = pending.finally(() => refreshPromises.delete(key))
  refreshPromises.set(key, tracked)
  return tracked
}

function expireIfCurrent(accessToken: string, refreshToken: string): void {
  if (!sameSession(readAuthSession(), accessToken, refreshToken)) return
  clearAuthSession()
  window.dispatchEvent(new CustomEvent('auth:expired'))
}

function replayRequest(
  config: InternalAxiosRequestConfig,
  session: StoredAuthSession,
): Promise<unknown> {
  config._retry = true
  config._authReplayAccessToken = session.accessToken
  config._authAccessToken = session.accessToken
  config._authRefreshToken = session.refreshToken
  config.headers.Authorization = `Bearer ${session.accessToken}`
  return api(config)
}

api.interceptors.response.use(
  response => response,
  async (error) => {
    if (error.response?.status !== 401 || !error.config) return Promise.reject(error)

    const config = error.config as InternalAxiosRequestConfig
    const originalAccessToken = config._authAccessToken
    const originalRefreshToken = config._authRefreshToken
    if (!originalAccessToken || !originalRefreshToken) return Promise.reject(error)

    if (config._retry) {
      expireIfCurrent(originalAccessToken, originalRefreshToken)
      return Promise.reject(error)
    }

    const current = readAuthSession()
    if (!current) return Promise.reject(error)

    // A delayed 401 from a token already rotated by this same session can be
    // replayed directly. An unrelated current login is never refreshed or used.
    if (!sameSession(current, originalAccessToken, originalRefreshToken)) {
      if (rotationReachesCurrent(originalAccessToken, originalRefreshToken, current)) {
        return replayRequest(config, current)
      }
      return Promise.reject(error)
    }

    try {
      const context = await refreshSession(current)
      const afterRefresh = readAuthSession()
      const alreadyApplied = sameSession(afterRefresh, context.accessToken, context.refreshToken)
      if (sameSession(afterRefresh, context.originalAccessToken, context.originalRefreshToken)) {
        writeAuthSession(
          {
            accessToken: context.accessToken,
            refreshToken: context.refreshToken,
            user: context.user,
          },
          context.rememberMe,
        )
        rememberRotation(context)
        window.dispatchEvent(new CustomEvent('auth:refreshed'))
      } else if (!alreadyApplied) {
        return Promise.reject(error)
      }

      const refreshed = readAuthSession()
      if (!sameSession(refreshed, context.accessToken, context.refreshToken)) {
        return Promise.reject(error)
      }
      rememberRotation(context)
      return replayRequest(config, refreshed)
    } catch {
      expireIfCurrent(originalAccessToken, originalRefreshToken)
      return Promise.reject(error)
    }
  },
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
  category_name?: string
  user_id?: number
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
  author_id?: number
  author_display_name?: string
  reading_time?: number
  is_pinned: boolean
  allow_comment?: boolean
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
  guest_copy_enabled?: string
  batch_load_size: string
  scroll_load_size: string
  'site-manager'?: string
  ip_whitelist_enabled?: string
  ip_whitelist?: string
  ip_blacklist_enabled?: string
  ip_blacklist?: string
  [key: string]: string | undefined
}
