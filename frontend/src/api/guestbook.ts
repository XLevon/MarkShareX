import api from './index'
import type { PaginatedData } from './index'

export interface GuestbookEntry {
  id: number
  user_id: number | null
  username: string | null
  nickname: string
  email: string
  content: string
  content_html: string
  reply: string | null
  is_replied: boolean
  status: string
  created_at: string
  updated_at: string
}

export function fetchGuestbook(params?: {
  page?: number
  page_size?: number
  search?: string
}) {
  return api.get<PaginatedData<GuestbookEntry>>('/guestbook', { params })
}

export function createGuestbook(data: { nickname: string; email: string; content: string }) {
  return api.post<{ data: GuestbookEntry }>('/guestbook', data)
}

export function replyGuestbook(id: number, reply: string) {
  return api.put<{ data: GuestbookEntry }>(`/admin/guestbook/${id}/reply`, { reply })
}

export function deleteGuestbook(id: number) {
  return api.delete(`/admin/guestbook/${id}`)
}
