import api from './index'
import type { PaginatedData } from './index'

export interface NewsItem {
  id: number
  title: string
  summary: string
  content: string
  content_html: string
  status: string
  sort_order: number
  published_at: string | null
  user_id: number | null
  created_at: string
  updated_at: string
}

export function fetchNews(params?: { page?: number; page_size?: number }) {
  return api.get<PaginatedData<NewsItem>>('/news', { params })
}

export function fetchNewsItem(id: number) {
  return api.get<{ data: NewsItem }>(`/news/${id}`)
}

export function fetchAdminNews(params?: { page?: number; page_size?: number }) {
  return api.get<PaginatedData<NewsItem>>('/admin/news', { params })
}

export function createNews(data: {
  title: string
  summary?: string
  content?: string
  content_html?: string
  status?: string
  sort_order?: number
  published_at?: string | null
}) {
  return api.post<{ data: NewsItem }>('/admin/news', data)
}

export function updateNews(id: number, data: {
  title?: string
  summary?: string
  content?: string
  content_html?: string
  status?: string
  sort_order?: number
  published_at?: string | null
}) {
  return api.put<{ data: NewsItem }>(`/admin/news/${id}`, data)
}

export function deleteNews(id: number) {
  return api.delete<{ data: null }>(`/admin/news/${id}`)
}
