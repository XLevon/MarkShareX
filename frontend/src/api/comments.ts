import api from './index'
import type { Comment, PaginatedData } from './index'

export function fetchComments(postId: number, admin = false) {
  return api.get<{ data: Comment[] }>(`/posts/${postId}/comments`, { params: admin ? { admin: '1' } : {} })
}

export function createComment(
  postId: number,
  data: { content: string; parent_id?: number; author_name?: string; author_email?: string }
) {
  return api.post<{ data: Comment }>(`/posts/${postId}/comments`, data)
}

export function fetchAdminComments(params?: { page?: number; page_size?: number; status?: string }) {
  return api.get<PaginatedData<Comment>>('/admin/comments', { params })
}

export function updateCommentStatus(id: number, status: string) {
  return api.put<{ data: Comment }>(`/admin/comments/${id}`, { status })
}
