import api from './index'
import type { Post, PaginatedData } from './index'

export interface PostsParams {
  page?: number
  page_size?: number
  status?: string
  category_id?: number
  tag_id?: number
  search?: string
  article_type?: string
  article_status?: string
  is_pinned?: boolean
  include_subcategories?: boolean
}

export function fetchPosts(params?: PostsParams) {
  return api.get<PaginatedData<Post>>('/posts', { params })
}

// Admin-only: fetches posts filtered by user_id for non-privileged users
export function fetchAdminPosts(params?: {
  page?: number
  page_size?: number
  status?: string
  category_id?: number
  category_ids?: string  // comma-separated "1,2,3"
  tag_id?: number
  tag_search?: string   // fuzzy search by tag name
  search?: string
  article_type?: string
  article_types?: string  // comma-separated "tutorial,news"
  article_status?: string
  article_statuses?: string  // comma-separated "draft,reviewed"
  is_pinned?: boolean
}) {
  return api.get<PaginatedData<Post>>('/admin/posts', { params })
}

export function fetchPost(id: number) {
  return api.get<{ data: Post }>(`/posts/${id}`)
}

export function fetchPostBySlug(slug: string) {
  return api.get<{ data: Post }>(`/posts/slug/${slug}`)
}

export function createPost(data: Partial<Post> & { tags?: string[] }) {
  return api.post<{ data: Post }>('/posts', data)
}

export function updatePost(id: number, data: Partial<Post> & { tags?: string[] }) {
  return api.put<{ data: Post }>(`/posts/${id}`, data)
}

export function deletePost(id: number) {
  return api.delete(`/posts/${id}`)
}

// ── 置顶相关 ──
export function pinPost(id: number) {
  return api.post<{ data: Post }>(`/admin/posts/${id}/pin`)
}

export function unpinPost(id: number) {
  return api.post<{ data: Post }>(`/admin/posts/${id}/unpin`)
}

export function updatePinOrder(postIds: number[]) {
  return api.put<{ data: string }>('/admin/posts/pin-order', { post_ids: postIds })
}

export function fetchPinnedPosts() {
  return api.get<{ data: Post[] }>('/posts/pinned')
}
