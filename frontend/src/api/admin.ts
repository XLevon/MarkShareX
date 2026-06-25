import api from './index'

export interface AdminUser {
  id: number
  username: string
  email: string
  display_name: string | null
  role: string
  status: 'active' | 'muted' | 'banned'
  is_active: boolean
  last_login_at: string | null
  created_at: string
  application?: ApplicationInfo | null
}

export interface PaginatedUsers {
  data: AdminUser[]
  pagination: {
    total: number
    pages: number
    page: number
    page_size: number
  }
}

export function fetchUsers(params?: {
  page?: number
  page_size?: number
  status?: string
  search?: string
}) {
  return api.get<PaginatedUsers>('/admin/users', { params })
}

export function updateUserStatus(id: number, status: string) {
  return api.put<{ data: AdminUser }>(`/admin/users/${id}/status`, { status })
}

export function updateUserRole(id: number, role: string) {
  return api.put<{ data: AdminUser }>(`/admin/users/${id}/role`, { role })
}

export function deleteUser(id: number) {
  return api.delete(`/admin/users/${id}`)
}

export interface CreateUserPayload {
  username: string
  email: string
  password: string
  display_name?: string
  role?: string
  status?: string
}

export function createUser(payload: CreateUserPayload) {
  return api.post<{ data: AdminUser }>('/admin/users', payload)
}

export interface UpdateUserPayload {
  display_name?: string
  email?: string
  role?: string
  status?: string
}

export function updateUser(id: number, payload: UpdateUserPayload) {
  return api.put<{ data: AdminUser }>(`/admin/users/${id}`, payload)
}

// ── Profile ──

export interface UserProfile {
  id: number
  username: string
  email: string
  display_name: string | null
  role: string
  status: string
  avatar_url: string | null
  bio: string | null
  title: string | null
  created_at: string
  last_login_at: string | null
}

export type UpdateProfilePayload = Partial<Pick<UserProfile, 'display_name' | 'email' | 'bio' | 'title'>>

export function fetchProfile() {
  return api.get<{ data: UserProfile }>('/profile')
}

export function updateProfile(payload: UpdateProfilePayload) {
  return api.put<{ data: UserProfile }>('/profile', payload)
}

export interface ChangePasswordPayload {
  old_password: string
  new_password: string
  confirm_password: string
}

export function changePassword(payload: ChangePasswordPayload) {
  return api.put('/profile/password', payload)
}

// ── API Key ──

export interface ApiKeyData {
  api_key: string | null
}

export function fetchApiKey() {
  return api.get<{ data: ApiKeyData }>('/profile/api-key')
}

export function regenerateApiKey() {
  return api.put<{ data: ApiKeyData }>('/profile/api-key')
}

// ── Author Applications ──

export interface ApplicationInfo {
  id: number
  reason: string
  content_description: string
  status: string
  admin_remark: string | null
  created_at: string
}

export interface ApplicationResponse {
  id: number
  user_id: number
  username: string
  email: string
  display_name: string | null
  reason: string
  content_description: string
  status: string
  admin_remark: string | null
  created_at: string
}

export function submitApplication(reason: string, content: string) {
  return api.post<{ data: ApplicationResponse }>('/apply', { reason, content })
}

export function getApplicationStatus() {
  return api.get<{ data: ApplicationResponse | null }>('/apply/status')
}

export function approveApplication(id: number) {
  return api.post<{ data: ApplicationResponse }>(`/admin/applications/${id}/approve`)
}

export function rejectApplication(id: number, remark?: string) {
  return api.post<{ data: ApplicationResponse }>(`/admin/applications/${id}/reject`, { remark })
}

export function getPendingCount() {
  return api.get<{ data: number }>('/admin/applications/pending-count')
}

export function getCommentPendingCount(params?: { scope?: string }) {
  return api.get<{ data: number }>('/admin/comments/pending-count', { params })
}

// ── Network Resources ──

export interface NetworkResource {
  id: number
  url: string
  label: string | null
  source_type: string
  referenced: boolean
  created_at: string
  updated_at: string
}

export interface NetworkResourceListResponse {
  data: NetworkResource[]
  pagination?: {
    total: number
    pages: number
    page: number
    page_size: number
  }
}

export function fetchNetworkResources(params?: {
  page?: number
  page_size?: number
  search?: string
  source_type?: string
}) {
  return api.get<NetworkResourceListResponse>('/network-resources', { params })
}

export function createNetworkResource(data: { url: string; label?: string; source_type?: string }) {
  return api.post<{ data: NetworkResource }>('/network-resources', data)
}

export function updateNetworkResource(id: number, data: { url?: string; label?: string }) {
  return api.put<{ data: NetworkResource }>(`/network-resources/${id}`, data)
}

export function deleteNetworkResource(id: number) {
  return api.delete(`/network-resources/${id}`)
}

export interface ReferenceItem {
  target_type: string
  target_id: number
  target_name: string
  target_slug: string
  target_description: string | null
}

export function fetchNetworkResourceReferences(id: number) {
  return api.get<{ data: ReferenceItem[] }>(`/network-resources/${id}/references`)
}

// ── Login Logs ──

export interface LoginLog {
  id: number
  user_id: number | null
  username: string
  ip_address: string | null
  device_type: string | null
  login_method: string
  success: boolean
  created_at: string
}

export interface PaginatedLoginLogs {
  data: LoginLog[]
  pagination: { total: number; pages: number; page: number; page_size: number }
}

export function fetchLoginLogs(params?: {
  page?: number; page_size?: number; user_id?: number; success?: boolean
}) {
  return api.get<PaginatedLoginLogs>('/admin/login-logs', { params })
}

// ── Read Logs ──

export interface ReadLog {
  id: number
  post_id: number
  post_title: string | null
  user_id: number | null
  username: string | null
  ip_address: string | null
  device_type: string | null
  referrer: string | null
  duration_seconds: number
  created_at: string
}

export interface PaginatedReadLogs {
  data: ReadLog[]
  pagination: { total: number; pages: number; page: number; page_size: number }
}

export function fetchReadLogs(params?: {
  page?: number; page_size?: number; post_id?: number; user_id?: number
}) {
  return api.get<PaginatedReadLogs>('/admin/read-logs', { params })
}

export function recordReadLog(data: { post_id: number; duration_seconds?: number; referrer?: string }) {
  return api.post('/read-logs', data)
}

// ── Article Types & Statuses (Knowledge Base) ──

export interface ArticleType {
  id: number
  code: string
  display_name: string
  color: string
  sort_order: number
  is_active: boolean
  post_count: number
  created_at?: string
  updated_at?: string
}

export interface ArticleStatus {
  id: number
  code: string
  display_name: string
  color: string
  sort_order: number
  is_active: boolean
  post_count: number
  created_at?: string
  updated_at?: string
}

// Public
export function fetchArticleTypes() {
  return api.get<ArticleType[]>('/article-types')
}
export function fetchArticleStatuses() {
  return api.get<ArticleStatus[]>('/article-statuses')
}

// Admin
export function fetchAdminArticleTypes() {
  return api.get<ArticleType[]>('/admin/article-types')
}
export function createArticleType(data: { code: string; display_name: string; color?: string; sort_order?: number }) {
  return api.post<ArticleType>('/admin/article-types', data)
}
export function updateArticleType(id: number, data: Partial<ArticleType>) {
  return api.put<ArticleType>(`/admin/article-types/${id}`, data)
}
export function deleteArticleType(id: number) {
  return api.delete(`/admin/article-types/${id}`)
}
export function reorderArticleTypes(ids: number[]) {
  return api.post('/admin/article-types/reorder', { ids })
}
export function fetchAdminArticleStatuses() {
  return api.get<ArticleStatus[]>('/admin/article-statuses')
}
export function createArticleStatus(data: { code: string; display_name: string; color?: string; sort_order?: number }) {
  return api.post<ArticleStatus>('/admin/article-statuses', data)
}
export function updateArticleStatus(id: number, data: Partial<ArticleStatus>) {
  return api.put<ArticleStatus>(`/admin/article-statuses/${id}`, data)
}
export function deleteArticleStatus(id: number) {
  return api.delete(`/admin/article-statuses/${id}`)
}
export function reorderArticleStatuses(ids: number[]) {
  return api.post('/admin/article-statuses/reorder', { ids })
}
